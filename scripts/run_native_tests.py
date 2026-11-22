#!/usr/bin/env python3
"""Link integration tests against cached engine/set libraries and run serially."""
import argparse
import errno
import fnmatch
import json
import math
import re
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import tempfile
import time

if __package__:
    from .resource_limits import bounded_group, managed_context
    from .build_platform import allocated_bytes, argument_batches, exec_process
else:
    from resource_limits import bounded_group, managed_context
    from build_platform import allocated_bytes, argument_batches, exec_process

ROOT = Path(__file__).resolve().parents[1]


def descendants(pid):
    """Find nested sessions too (Cargo and nested test runners may create them)."""
    if sys.platform != 'linux':
        import psutil
        try:
            return {p.pid for p in psutil.Process(pid).children(recursive=True)}
        except psutil.NoSuchProcess:
            return set()
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
    if sys.platform != 'linux':
        import psutil
        try:
            process = psutil.Process(pid)
            return process.is_running() and process.status() not in (psutil.STATUS_ZOMBIE, psutil.STATUS_DEAD)
        except psutil.NoSuchProcess:
            return False
    try:
        # The command name may contain spaces or parentheses.
        state = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()[0]
        return state not in ('Z', 'X')
    except (FileNotFoundError, ProcessLookupError):
        return False


def group_members():
    group = bounded_group()
    if group is None:
        context = managed_context()
        if context is None:
            raise RuntimeError('managed process membership unavailable; preserve outputs')
        return set(context['members'])
    return {int(pid) for pid in (group/'cgroup.procs').read_text().split()
            if process_running(int(pid))}


def active_peers():
    # A normally exiting parent can orphan children outside our /proc subtree.
    # The guarded service cgroup still owns those processes.
    ancestors = {os.getpid()}
    if sys.platform != 'linux':
        import psutil
        ancestors.update(process.pid for process in psutil.Process().parents())
        return group_members() - ancestors
    pid = os.getpid()
    while pid > 1:
        try:
            pid = int(Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()[1])
        except (FileNotFoundError, ProcessLookupError):
            break
        ancestors.add(pid)
    return group_members() - ancestors


def process_details(pids):
    """Retain safe identity evidence without command arguments or environments."""
    try:
        import psutil
    except ImportError:
        # The Linux cgroup backend does not require the portable dependency.
        return [dict(pid=pid) for pid in sorted(pids)]
    details = []
    for pid in sorted(pids):
        try:
            details.append(psutil.Process(pid).as_dict(
                attrs=['pid', 'ppid', 'name', 'exe', 'create_time'], ad_value=None))
        except psutil.NoSuchProcess:
            details.append(dict(pid=pid, exited=True))
    return details


def stop_process(process, members_before=None):
    if bounded_group() is None:
        import psutil
        if __package__:
            from .portable_guard import stop_processes
        else:
            from portable_guard import stop_processes
        deadline = time.monotonic() + 5
        while True:
            context = managed_context()
            if context is None:
                raise RuntimeError('build supervisor disappeared; preserve outputs')
            processes = []
            for pid, birth in context['identities']:
                if members_before is not None and pid in members_before:
                    continue
                if members_before is None and pid not in descendants(process.pid) | {process.pid}:
                    continue
                try:
                    candidate = psutil.Process(pid)
                    if candidate.create_time() == birth:
                        processes.append(candidate)
                except psutil.NoSuchProcess:
                    pass
            if not processes:
                break
            stop_processes(processes)
            if time.monotonic() > deadline:
                raise RuntimeError('command descendants remain active; preserve outputs')
        process.wait()
        return
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


def list_compiled_tests(binary, *, target, filter_name, env, log):
    command = [str(binary), filter_name, '--list', '--format=terse']
    try:
        return run_command(command, env=env, log=log, timeout=5)
    except PermissionError as error:
        resolved_binary = binary.resolve()
        if (sys.platform == 'win32' or error.errno != errno.EACCES
                or not error.filename or Path(error.filename).resolve() != resolved_binary
                or target.resolve() not in resolved_binary.parents
                or not stat.S_ISREG(binary.stat().st_mode)):
            raise
        # A newly built macOS Cargo artifact can arrive without its owner execute bit.
        # Repair only that compiled test file, then make one ordinary bounded retry.
        binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
        print(f'Retrying compiled test executable after EACCES: {binary}', flush=True)
        return run_command(command, env=env, log=log, timeout=5)


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
    timeout_scope = None
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
                    timeout_scope = 'suite' if now-started >= timeout else 'case'
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
                timeout_scope=timeout_scope,
                orphaned_processes=orphaned, seconds=time.monotonic()-started,
                log=str(log))


def retire_test_binaries(binaries, target):
    """Only remove completed Cargo test executables, after their processes exit."""
    parent = target/'debug'/'deps'
    if parent.resolve() != target.resolve()/'debug'/'deps':
        raise RuntimeError('native test directory escapes the Cargo target; preserve outputs')
    if len(set(binaries)) != len(binaries):
        raise RuntimeError('duplicate native test binary record; preserve outputs')
    for path in binaries:
        if (not path.is_absolute() or path.is_symlink() or not path.is_file()
                or path.resolve().parent != parent
                or path.suffix != ('.exe' if sys.platform == 'win32' else '')):
            raise RuntimeError('native test binary is outside the Cargo test outputs; preserve outputs')
    if active_peers():
        raise RuntimeError('native test processes remain active; preserve outputs')
    reclaimed = sum(allocated_bytes(path) for path in binaries)
    for path in binaries:
        path.unlink()
    return len(binaries), reclaimed


def compiled_tests(log, target):
    binaries = []
    for line in log.read_text().splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (message.get('reason') == 'compiler-artifact' and message.get('executable')
                and message.get('profile', {}).get('test')
                and 'test' in message.get('target', {}).get('kind', [])):
            binaries.append((message['target'].get('name'), Path(message['executable']).resolve()))
    if not binaries or any(target not in binary.parents for _, binary in binaries):
        raise RuntimeError(f'expected integration test binaries under {target}, got {binaries}')
    return binaries


def grouped_modules(tests, selector):
    """Globs match former target names. An exact suite name selects the Cargo target."""
    result = {}
    for main in tests.glob('*/main.rs'):
        if main.parent.name != selector:
            result[main.parent.name] = {path.stem for path in main.parent.glob('*.rs')
                                        if path != main and fnmatch.fnmatchcase(path.stem, selector)}
    return result


def selected_case_batches(listed_names, modules, case_filter):
    if not listed_names:
        return
    if modules is None:
        # The listing was already filtered by libtest for this ordinary Cargo target.
        yield listed_names, [case_filter]
        return
    selected = []
    for name in listed_names:
        module, separator, original_case = name.partition('::')
        if separator and module in modules and case_filter in original_case:
            selected.append(name)
    if len(selected) == len(listed_names):
        yield selected, []  # Every listed case belongs to the selection.
    else:
        for batch in argument_batches(selected):
            yield batch, ['--exact', *batch]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--compile-timeout', type=float, default=1800)
    parser.add_argument('--test-timeout', type=float, default=45)
    parser.add_argument('--suite-timeout', type=float, default=240)
    parser.add_argument('--test-target', help='select original test targets by name or glob, or name a complete suite exactly')
    parser.add_argument('--filter', default='', help='execute matching cases in selected targets')
    parser.add_argument('--retire-binaries', action='store_true',
                        help='compile, execute and remove test executables in bounded batches')
    parser.add_argument('--batch-size', type=int, default=32,
                        help='targets per compile when retiring binaries (default: 32)')
    args = parser.parse_args()
    if any(not math.isfinite(v) or v <= 0 for v in (args.compile_timeout, args.test_timeout, args.suite_timeout)):
        parser.error('timeouts must be positive')
    if args.batch_size < 1:
        parser.error('--batch-size must be positive')
    if managed_context() is None:
        exec_process([sys.executable, str(ROOT/'scripts/resource_guard.py'), '--',
                      sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    signal.signal(signal.SIGTERM, interrupted)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    target = args.target_dir.resolve()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), RUST_TEST_THREADS='1')
    if 'RUSTMAGE_TDM_EVIDENCE_DIR' not in env:
        env['RUSTMAGE_TDM_EVIDENCE_DIR'] = tempfile.mkdtemp(
            prefix='tdm-games-', dir=args.report.resolve().parent)
    result = dict(success=False, terminal=False, tests=[], binaries=[], compile_batches=[],
                  execution_seconds=0, expected_tests=0,
                  retired_native_test_binaries=0, retired_native_test_bytes=0,
                  game_evidence_directory=env['RUSTMAGE_TDM_EVIDENCE_DIR'])
    started = time.monotonic()
    try:
        compile_log = args.report.with_name('native-compile.log')
        selector = args.test_target or '*'
        batches = [[selector]]
        groups = grouped_modules(ROOT/'tests', selector) if args.test_target or args.filter else {}
        compile_seconds = 0
        if args.retire_binaries or args.test_target:
            metadata_log = args.report.with_name('native-metadata.log')
            result['metadata'] = run_command(
                ['cargo', 'metadata', '--no-deps', '--format-version=1'],
                env=env, log=metadata_log, timeout=min(30, args.compile_timeout))
            compile_seconds += result['metadata']['seconds']
            if result['metadata']['exit']:
                return 1
            package = next(p for p in json.loads(metadata_log.read_text())['packages']
                           if Path(p['manifest_path']).resolve() == ROOT/'Cargo.toml')
            available = {t['name'] for t in package['targets'] if 'test' in t['kind']}
            direct = {name for name in available if fnmatch.fnmatchcase(name, selector)}
            selected = (direct - groups.keys()) | {name for name in available if groups.get(name)}
            if not selected:
                raise RuntimeError('no integration test targets selected')
            if args.retire_binaries or selected != direct:
                names = sorted(selected)
                batches = [names[i:i+args.batch_size] for i in range(0, len(names), args.batch_size)]
        separate_compile_logs = args.retire_binaries or len(batches) > 1
        expected = 0
        complete = True
        index = 0
        for batch_index, batch in enumerate(batches):
            remaining_compile = args.compile_timeout - compile_seconds
            if remaining_compile <= 0:
                result['budget_exceeded'] = True
                complete = False
                break
            command = ['cargo', 'test', '-p', 'rustmage']
            for name in batch:
                command.extend(['--test', name])
            command.extend(['--no-run', '--message-format=json'])
            log = (args.report.with_name(f'native-compile-{batch_index}.log')
                   if separate_compile_logs else compile_log)
            compiled = run_command(command, env=env, log=log, timeout=remaining_compile)
            result['compile_batches'].append(compiled)
            compile_seconds += compiled['seconds']
            result['compile'] = dict(compiled, seconds=compile_seconds)
            if separate_compile_logs:
                with compile_log.open('a' if batch_index else 'w') as output:
                    output.write(log.read_text())
                result['compile']['log'] = str(compile_log)
            if compiled['exit']:
                print(log.read_text()[-12000:], flush=True)
                return 1
            binaries = compiled_tests(log, target)
            if args.retire_binaries and (len(binaries) != len(batch)
                                        or {name for name, _ in binaries} != set(batch)):
                raise RuntimeError('compiled test targets do not match the selected batch')
            suite_start = time.monotonic()
            for target_name, binary in binaries:
                modules = groups.get(target_name)
                listing = args.report.with_name(f'native-list-{index}.log')
                index += 1
                listed = list_compiled_tests(binary, target=target, env=env, log=listing,
                                             filter_name=args.filter if modules is None else '')
                if listed['exit']:
                    return 1
                listed_names = [line.removesuffix(': test') for line in listing.read_text().splitlines()
                                if line.endswith(': test')]
                for names, case_args in selected_case_batches(listed_names, modules, args.filter):
                    expected += len(names)
                    remaining = (args.suite_timeout - result['execution_seconds']
                                 - (time.monotonic()-suite_start))
                    if remaining <= 0:
                        result['budget_exceeded'] = True
                        print(f'Native suite execution budget exhausted ({args.suite_timeout:g}s)', flush=True)
                        complete = False
                        break
                    execution = run_suite(
                        [str(binary), *case_args, '--include-ignored', '--test-threads=1'],
                        env=env, log=args.report.with_name(f'native-execution-{len(result["binaries"])}.log'),
                        timeout=remaining, test_timeout=args.test_timeout)
                    output = Path(execution['log']).read_text()
                    tests, accounted = suite_results(names, output)
                    for test in tests:
                        test['binary'] = str(binary)
                    result['tests'].extend(tests)
                    result['binaries'].append(dict(binary=str(binary), expected=len(names), execution=execution))
                    if execution['exit'] or not accounted:
                        complete = False
                        if execution.get('timeout_scope') == 'suite':
                            result['budget_exceeded'] = True
                            print(f'Native suite execution budget exhausted ({args.suite_timeout:g}s)', flush=True)
                        elif execution.get('timeout_scope') == 'case':
                            print(f"Native case deadline exceeded ({args.test_timeout:g}s): "
                                  f"{execution['timed_out_case']}", flush=True)
                        print(output[-12000:], flush=True)
                        break
                if not complete:
                    break
            result['execution_seconds'] += time.monotonic()-suite_start
            if not complete:
                break
            if args.retire_binaries:
                count, reclaimed = retire_test_binaries([binary for _, binary in binaries], target)
                result['retired_native_test_binaries'] += count
                result['retired_native_test_bytes'] += reclaimed
                print(f'Native batch {batch_index+1}/{len(batches)} passed; retired {count} executables',
                      flush=True)
        result['expected_tests'] = expected
        if complete and args.test_target and expected == 0:
            raise RuntimeError('selected target/filter contains no tests')
        result['success'] = complete and expected > 0 and len(result['tests']) == expected
        print(f"Native assurance: {len(result['tests'])}/{expected} cases in {result['execution_seconds']:.3f}s", flush=True)
        return 0 if result['success'] else 1
    except KeyboardInterrupt:
        result['interrupted'] = True
        return 130
    except (OSError, RuntimeError, ValueError, KeyError, StopIteration) as error:
        result['error'] = str(error)
        print(str(error), file=sys.stderr, flush=True)
        return 1
    finally:
        result['active_processes'] = sorted(active_peers())
        if result['active_processes']:
            result['active_process_details'] = process_details(result['active_processes'])
            print('Active native peers: '+json.dumps(result['active_process_details']), flush=True)
        result['terminal'] = not result['active_processes']
        result['success'] = result['success'] and result['terminal']
        result['seconds'] = time.monotonic()-started
        args.report.write_text(json.dumps(result, indent=2)+'\n')
        print(f'Native test report: {args.report}', flush=True)
        if not result['terminal']:
            raise RuntimeError('native runner left active processes; outputs preserved')


if __name__ == '__main__':
    sys.exit(main())
