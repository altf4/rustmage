#!/usr/bin/env python3
"""Build once and run the card-coverage suite in one bounded native process."""
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
    """Account for every listed test; a zero-exit or skipped suite is not a pass."""
    tests = [dict(test=name, status=status) for name, status in
             re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$', output, re.M)]
    success = (bool(names) and {test['test'] for test in tests} == set(names)
               and len(tests) == len(names)
               and all(test['status'] == 'ok' for test in tests)
               and f'test result: ok. {len(names)} passed; 0 failed; 0 ignored;' in output)
    return tests, success


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--compile-timeout', type=float, default=360)
    parser.add_argument('--suite-timeout', type=float, default=30)
    parser.add_argument('--filter', default='', help='run matching Rust test names for local iteration')
    args = parser.parse_args()
    if any(not math.isfinite(v) or v <= 0 for v in (args.compile_timeout, args.suite_timeout)):
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
        result['filter'] = args.filter
        result['list'] = run_command([binary, args.filter, '--list', '--format=terse'], env=env,
                                     log=listing, timeout=5)
        if result['list']['exit']:
            return 1
        names = [line.removesuffix(': test') for line in listing.read_text().splitlines()
                 if line.endswith(': test')]
        if not names:
            raise RuntimeError('the native library suite contains no tests')
        # Starting a process for every scenario makes test count itself expensive.
        # libtest already executes serially and identifies the currently running
        # case in the flushed log. The outer runner bounds the entire process tree.
        execution = run_command(
            [binary, args.filter, '--include-ignored', '--test-threads=1'], env=env,
            log=args.report.with_name('native-execution.log'), timeout=args.suite_timeout)
        output = Path(execution['log']).read_text()
        result['execution'] = execution
        result['tests'], accounted = suite_results(names, output)
        result['execution_seconds'] = execution['seconds']
        result['expected_tests'] = len(names)
        result['success'] = execution['exit'] == 0 and accounted
        print(f"Native card coverage: {len(result['tests'])}/{len(names)} cases in {execution['seconds']:.3f}s", flush=True)
        if not result['success']:
            print(output[-12000:], flush=True)
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
