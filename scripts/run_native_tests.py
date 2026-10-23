#!/usr/bin/env python3
"""Build the library test binary once, then time each bounded rules scenario."""
import argparse
import json
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


def stop_process(process):
    """Reap the command and stop its descendants before outputs may be cleaned."""
    children = descendants(process.pid)
    for sig in (signal.SIGTERM, signal.SIGKILL):
        children.update(descendants(process.pid))
        for pid in [*children, process.pid]:
            try:
                os.kill(pid, sig)
            except ProcessLookupError:
                pass
        try:
            process.wait(timeout=2 if sig == signal.SIGTERM else None)
        except subprocess.TimeoutExpired:
            pass
        # A parent can exit on TERM before its children; always send the KILL
        # pass to the captured descendants before returning.
    deadline = time.monotonic() + 2
    while any(process_running(pid) for pid in children) and time.monotonic() < deadline:
        time.sleep(0.01)
    if any(process_running(pid) for pid in children):
        raise RuntimeError('command descendants are still active; preserve their outputs')


def run_command(command, *, env, log, timeout):
    started = time.monotonic()
    with Path(log).open('w') as output:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = process.wait(timeout=max(0.01, timeout))
            timed_out = False
        except subprocess.TimeoutExpired:
            stop_process(process)
            code, timed_out = 124, True
        except BaseException:
            stop_process(process)
            raise
    return dict(command=[str(arg) for arg in command], exit=code, timed_out=timed_out,
                seconds=time.monotonic()-started, log=str(log))


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
    if min(args.compile_timeout, args.test_timeout, args.suite_timeout) <= 0:
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
        suite_start = time.monotonic()
        with args.report.with_suffix('.jsonl').open('w') as events:
            for index, name in enumerate(names):
                remaining = args.suite_timeout - (time.monotonic()-suite_start)
                if remaining <= 0:
                    result['budget_exceeded'] = True
                    break
                events.write(json.dumps(dict(event='start', test=name))+'\n')
                events.flush()
                test = run_command([binary, '--exact', name, '--nocapture', '--test-threads=1'],
                                   env=env, log=args.report.with_name(f'native-{index:03}.log'),
                                   timeout=min(args.test_timeout, remaining))
                test['test'] = name
                result['tests'].append(test)
                events.write(json.dumps(dict(event='finish', **test))+'\n')
                events.flush()
                print(f"{name}: exit={test['exit']} {test['seconds']:.3f}s", flush=True)
                if test['exit']:
                    break
        result['execution_seconds'] = time.monotonic()-suite_start
        result['expected_tests'] = len(names)
        result['success'] = (len(result['tests']) == len(names)
                             and all(test['exit'] == 0 for test in result['tests']))
        return 0 if result['success'] else 1
    except KeyboardInterrupt:
        result['interrupted'] = True
        return 130
    finally:
        result['terminal'] = True
        result['seconds'] = time.monotonic()-started
        args.report.write_text(json.dumps(result, indent=2)+'\n')
        print(f'Native test report: {args.report}', flush=True)


if __name__ == '__main__':
    sys.exit(main())
