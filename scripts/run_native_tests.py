#!/usr/bin/env python3
"""Link integration tests against cached engine/set libraries and run serially."""
import argparse
import json
import math
import re
import os
from pathlib import Path
import signal
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
        except (FileNotFoundError, ProcessLookupError):
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
    except (FileNotFoundError, ProcessLookupError):
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
        except (FileNotFoundError, ProcessLookupError):
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


def interrupted(signum, frame):
    raise KeyboardInterrupt


def suite_results(names, output):
    tests = [dict(test=name, status=status, exit=0 if status == 'ok' else 1)
             for name, status in re.findall(
                 r'^test (\S+) \.\.\. (ok|FAILED|ignored)$', output, re.M)]
    complete = (bool(names) and len(tests) == len(names)
                and {test['test'] for test in tests} == set(names)
                and all(test['status'] == 'ok' for test in tests)
                and f'test result: ok. {len(names)} passed; 0 failed; 0 ignored;' in output)
    return tests, complete


def run_suite(command, *, env, log, timeout, test_timeout):
    """Bound the suite and the unfinished case reported by serial libtest.

    libtest flushes `test NAME ... ` before entering each case. A single process
    avoids per-case startup overhead; the outer guard still owns descendants.
    """
    started = time.monotonic()
    members_before = group_members()
    active = None
    active_since = started
    timed_out_case = None
    timed_out = False
    with Path(log).open('w') as output:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        try:
            while process.poll() is None:
                now = time.monotonic()
                text = Path(log).read_text(errors='replace')
                matches = list(re.finditer(r'^test (\S+) \.\.\. ', text, re.M))
                current = None
                if matches:
                    last = matches[-1]
                    if not re.match(r'(ok|FAILED|ignored)\n', text[last.end():]):
                        current = last[1]
                if current != active:
                    active, active_since = current, now
                if now-started >= timeout or (active and now-active_since >= test_timeout):
                    timed_out = True
                    timed_out_case = active
                    stop_process(process, members_before)
                    break
                try:
                    process.wait(timeout=0.025)
                except subprocess.TimeoutExpired:
                    pass
            code = process.wait()
            orphaned = bool(group_members() - members_before)
            if orphaned:
                stop_process(process, members_before)
            if timed_out:
                code = 124
            elif orphaned:
                code = code or 1
        except BaseException:
            stop_process(process, members_before)
            raise
    return dict(command=[str(arg) for arg in command], exit=code,
                timed_out=timed_out, timed_out_case=timed_out_case,
                orphaned_processes=orphaned, seconds=time.monotonic()-started,
                log=str(log))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--compile-timeout', type=float, default=360)
    parser.add_argument('--test-timeout', type=float, default=45)
    parser.add_argument('--suite-timeout', type=float, default=240)
    parser.add_argument('--test-target', help='compile only this integration target')
    parser.add_argument('--filter', default='', help='execute matching cases in selected targets')
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
            ['cargo', 'test', '-p', 'rustmage', '--test', args.test_target or '*',
             '--no-run', '--message-format=json'],
            env=env, log=compile_log, timeout=args.compile_timeout)
        if result['compile']['exit']:
            print(compile_log.read_text()[-12000:], flush=True)
            return 1
        binaries = []
        for line in compile_log.read_text().splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if (message.get('reason') == 'compiler-artifact' and message.get('executable')
                    and message.get('profile', {}).get('test')
                    and 'test' in message.get('target', {}).get('kind', [])):
                binaries.append(Path(message['executable']).resolve())
        if not binaries or any(target not in binary.parents for binary in binaries):
            raise RuntimeError(f'expected integration test binaries under {target}, got {binaries}')
        result['binaries'] = []
        expected = 0
        suite_start = time.monotonic()
        complete = True
        for index, binary in enumerate(binaries):
            listing = args.report.with_name(f'native-list-{index}.log')
            listed = run_command([str(binary), args.filter, '--list', '--format=terse'],
                                 env=env, log=listing, timeout=5)
            if listed['exit']:
                return 1
            names = [line.removesuffix(': test') for line in listing.read_text().splitlines()
                     if line.endswith(': test')]
            if not names:
                if args.test_target:
                    raise RuntimeError('selected target/filter contains no tests')
                continue
            expected += len(names)
            remaining = args.suite_timeout - (time.monotonic()-suite_start)
            if remaining <= 0:
                result['budget_exceeded'] = True
                complete = False
                break
            execution = run_suite(
                [str(binary), args.filter, '--include-ignored', '--test-threads=1'],
                env=env, log=args.report.with_name(f'native-execution-{index}.log'),
                timeout=remaining, test_timeout=args.test_timeout)
            output = Path(execution['log']).read_text()
            tests, accounted = suite_results(names, output)
            for test in tests:
                test['binary'] = str(binary)
            result['tests'].extend(tests)
            result['binaries'].append(dict(binary=str(binary), expected=len(names), execution=execution))
            if execution['exit'] or not accounted:
                complete = False
                print(output[-12000:], flush=True)
                break
        result['execution_seconds'] = time.monotonic()-suite_start
        result['expected_tests'] = expected
        result['success'] = complete and expected > 0 and len(result['tests']) == expected
        print(f"Native assurance: {len(result['tests'])}/{expected} cases in {result['execution_seconds']:.3f}s", flush=True)
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
