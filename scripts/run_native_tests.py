#!/usr/bin/env python3
"""Build the library test binary once, then time each bounded rules scenario."""
import argparse
import json
import math
import re
import os
from pathlib import Path
import signal
import selectors
import subprocess
import sys
import time

if __package__:
    from .resource_limits import bounded_group
else:
    from resource_limits import bounded_group

ROOT = Path(__file__).resolve().parents[1]


def descendants(pid):
    """Find nested sessions too (Cargo and nested test runners may create them)."""
    found = set()
    pending = [pid]
    while pending:
        parent = pending.pop()
        try:
            children = Path(f'/proc/{parent}/task/{parent}/children').read_text().split()
        except FileNotFoundError:
            continue
        for raw in children:
            child = int(raw)
            if child not in found:
                found.add(child)
                pending.append(child)
    return found


def process_running(pid):
    try:
        # The command name may contain spaces or parentheses.
        state = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()[0]
        return state not in ('Z', 'X')
    except FileNotFoundError:
        return False


def group_members():
    group = bounded_group()
    if group is None:
        return set()
    return {int(pid) for pid in (group/'cgroup.procs').read_text().split()
            if process_running(int(pid))}


def active_peers():
    # A normally exiting parent can orphan children outside our /proc subtree.
    # The guarded service cgroup still owns those processes.
    ancestors = {os.getpid()}
    pid = os.getpid()
    while pid > 1:
        try:
            pid = int(Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()[1])
        except FileNotFoundError:
            break
        ancestors.add(pid)
    return group_members() - ancestors


def stop_process(process, members_before=None):
    children = descendants(process.pid)
    deadline = time.monotonic() + 3
    for sig in (signal.SIGTERM, signal.SIGKILL):
        while True:
            children.update(descendants(process.pid))
            if members_before is not None:
                children.update(group_members() - members_before)
            for pid in children | {process.pid}:
                try:
                    os.kill(pid, sig)
                except ProcessLookupError:
                    pass
            try:
                process.wait(timeout=0.1)
            except subprocess.TimeoutExpired:
                pass
            if sig == signal.SIGTERM or not any(process_running(pid) for pid in children):
                break
            if time.monotonic() >= deadline:
                raise RuntimeError('command descendants remain active; preserve outputs')
            time.sleep(0.01)
    process.wait()


def run_command(command, *, env, log, timeout):
    started = time.monotonic()
    members_before = group_members()
    orphaned = False
    with Path(log).open('w') as output:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = process.wait(timeout=max(0.01, timeout))
            timed_out = False
            if group_members() - members_before:
                orphaned = True
                stop_process(process, members_before)
                code = code or 1
        except subprocess.TimeoutExpired:
            stop_process(process, members_before)
            code, timed_out = 124, True
        except BaseException:
            stop_process(process, members_before)
            raise
    return dict(command=[str(arg) for arg in command], exit=code, timed_out=timed_out,
                orphaned_processes=orphaned, seconds=time.monotonic()-started, log=str(log))


class NativeProgress:
    """Parse flushed, serial libtest progress with ordinary stdout capture.

    Start prefixes have no newline. Consume chunks so a hanging test cannot
    hide its start from the deadline watcher. Captured body output is emitted
    after the progress stream, where it cannot impersonate test results.
    """
    def __init__(self, names, events, log):
        if not names or len(set(names)) != len(names):
            raise ValueError('expected unique, nonempty native test listing')
        self.names = names
        self.events = events
        self.log = str(log)
        self.buffer = b''
        self.active = None
        self.tests = []
        self.tail = b''

    def feed(self, chunk, now):
        if len(self.tests) == len(self.names):
            self.tail += chunk
            return
        self.buffer += chunk
        while len(self.tests) < len(self.names):
            if self.active is None:
                match = re.search(rb'(?m)^test ([^\r\n]+?) \.\.\. ', self.buffer)
                if match is None:
                    return
                name = match.group(1).decode('utf-8')
                if name.endswith(' - should panic'):
                    name = name.removesuffix(' - should panic')
                expected = self.names[len(self.tests)]
                if name != expected:
                    raise ValueError(f'expected native test {expected!r}, got {name!r}')
                self.active = (name, now)
                self.events.write(json.dumps(dict(event='start', test=name))+'\n')
                self.events.flush()
                self.buffer = self.buffer[match.end():]
            end = self.buffer.find(b'\n')
            if end < 0:
                return
            status = self.buffer[:end].rstrip(b'\r')
            if status not in (b'ok', b'FAILED'):
                raise ValueError(f'unexpected native status {status!r}')
            name, began = self.active
            row = dict(test=name, exit=0 if status == b'ok' else 1,
                       timed_out=False, orphaned_processes=False,
                       seconds=now-began, log=self.log)
            self.tests.append(row)
            self.events.write(json.dumps(dict(event='finish', **row))+'\n')
            self.events.flush()
            self.active = None
            self.buffer = self.buffer[end+1:]
        self.tail = self.buffer
        self.buffer = b''

    def complete(self):
        count = len(self.names)
        summary = (rf'\s*test result: ok\. {count} passed; 0 failed; 0 ignored; '
                   rf'0 measured; 0 filtered out; finished in [0-9.]+s\s*').encode()
        return (len(self.tests) == count and all(t['exit'] == 0 for t in self.tests)
                and re.fullmatch(summary, self.tail) is not None)


def run_test_suite(command, *, names, env, log, events, test_timeout, suite_timeout):
    """Run every serial test with unchanged per-test and global wall deadlines.

    One libtest process avoids spawning thousands of copies of a large native
    binary. The complete listing and flushed progress protocol remain mandatory;
    missing, reordered, ignored or failed tests cannot produce success.
    """
    started = time.monotonic()
    members_before = group_members()
    progress = NativeProgress(names, events, log)
    result = dict(success=False, tests=progress.tests, expected_tests=len(names),
                  command=[str(x) for x in command], log=str(log))
    process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, start_new_session=True)
    assert process.stdout is not None
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    between_deadline = started + test_timeout
    eof = False
    try:
        with Path(log).open('wb') as output:
            while not eof:
                now = time.monotonic()
                per_test_deadline = (progress.active[1] + test_timeout
                                     if progress.active else between_deadline)
                deadline = min(started + suite_timeout, per_test_deadline)
                if now >= deadline:
                    result['timed_out'] = True
                    result['budget_exceeded'] = now >= started + suite_timeout
                    if progress.active:
                        name, began = progress.active
                        row = dict(test=name, exit=124, timed_out=True,
                                   orphaned_processes=False, seconds=now-began,
                                   log=str(log))
                        progress.tests.append(row)
                        events.write(json.dumps(dict(event='finish', **row))+'\n')
                        events.flush()
                    stop_process(process, members_before)
                    break
                ready = selector.select(timeout=deadline-now)
                if not ready:
                    continue
                chunk = os.read(process.stdout.fileno(), 65536)
                if not chunk:
                    eof = True
                    break
                output.write(chunk)
                output.flush()
                try:
                    progress.feed(chunk, time.monotonic())
                except (ValueError, UnicodeError) as error:
                    result['error'] = str(error)
                    stop_process(process, members_before)
                    break
                if progress.active is None:
                    between_deadline = time.monotonic() + test_timeout
            if process.poll() is None:
                remaining = min(started + suite_timeout, between_deadline)-time.monotonic()
                try:
                    process.wait(timeout=max(0.001, remaining))
                except subprocess.TimeoutExpired:
                    result['timed_out'] = True
                    result['budget_exceeded'] = time.monotonic() >= started+suite_timeout
                    stop_process(process, members_before)
            result['exit'] = process.returncode
            peers = group_members() - members_before
            result['orphaned_processes'] = bool(peers)
            if peers:
                stop_process(process, members_before)
            result['success'] = (process.returncode == 0 and progress.complete()
                                 and not result.get('timed_out') and not peers
                                 and not result.get('error'))
            result['execution_seconds'] = time.monotonic()-started
            return result
    except BaseException:
        stop_process(process, members_before)
        raise
    finally:
        selector.close()
        process.stdout.close()


def interrupted(signum, frame):
    raise KeyboardInterrupt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--compile-timeout', type=float, default=360)
    parser.add_argument('--test-timeout', type=float, default=10)
    parser.add_argument('--suite-timeout', type=float, default=30)
    args = parser.parse_args()
    if any(not math.isfinite(v) or v <= 0 for v in (args.compile_timeout, args.test_timeout, args.suite_timeout)):
        parser.error('timeouts must be positive')
    if bounded_group() is None:
        os.execv(sys.executable, [sys.executable, str(ROOT/'scripts/resource_guard.py'), '--',
                                 sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    signal.signal(signal.SIGTERM, interrupted)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    target = args.target_dir.resolve()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), RUST_TEST_THREADS='1')
    result = dict(success=False, terminal=False, tests=[])
    started = time.monotonic()
    try:
        compile_log = args.report.with_name('native-compile.log')
        result['compile'] = run_command(
            ['cargo', 'test', '--lib', '--no-run', '--message-format=json'],
            env=env, log=compile_log, timeout=args.compile_timeout)
        if result['compile']['exit']:
            return 1
        binaries = []
        for line in compile_log.read_text().splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if (message.get('reason') == 'compiler-artifact' and message.get('executable')
                    and message.get('profile', {}).get('test')):
                binaries.append(Path(message['executable']).resolve())
        if len(binaries) != 1 or target not in binaries[0].parents:
            raise RuntimeError(f'expected one library test binary under {target}, got {binaries}')
        binary = str(binaries[0])
        listing = args.report.with_name('native-list.log')
        result['list'] = run_command([binary, '--list', '--format=terse'], env=env,
                                     log=listing, timeout=5)
        if result['list']['exit']:
            return 1
        names = [line.removesuffix(': test') for line in listing.read_text().splitlines()
                 if line.endswith(': test')]
        if not names:
            raise RuntimeError('the native library suite contains no tests')
        with args.report.with_suffix('.jsonl').open('w') as events:
            # Capture ordinary test output so only libtest writes progress.
            # Failed-test output is retained in the shared suite transcript.
            suite = run_test_suite(
                [binary, '--format=pretty', '--color=never', '--test-threads=1',
                 '--include-ignored'], names=names, env=env,
                log=args.report.with_name('native-suite.log'), events=events,
                test_timeout=args.test_timeout, suite_timeout=args.suite_timeout)
        result.update(suite)
        for test in result['tests']:
            print(f"{test['test']}: exit={test['exit']} {test['seconds']:.3f}s", flush=True)
        return 0 if result['success'] else 1
    except KeyboardInterrupt:
        result['interrupted'] = True
        return 130
    finally:
        result['active_processes'] = sorted(active_peers())
        result['terminal'] = not result['active_processes']
        result['success'] = result['success'] and result['terminal']
        result['seconds'] = time.monotonic()-started
        args.report.write_text(json.dumps(result, indent=2)+'\n')
        print(f'Native test report: {args.report}', flush=True)
        if not result['terminal']:
            raise RuntimeError('native runner left active processes; outputs preserved')


if __name__ == '__main__':
    sys.exit(main())
