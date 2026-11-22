#!/usr/bin/env python3
"""Run bounded card/interaction assurance with separate build and execution timings.

Cargo outputs are reused by default; every run owns separate reports and immutable
library snapshots under ~/.cache. Use --fresh only for an intentional clean build.
Keep source inputs fixed until this command has terminated, including after an
observation timeout.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time

if __package__:
    from .resource_limits import managed_context
    from .build_platform import library_name, executable_name, allocated_bytes, exec_process
    from .run_native_tests import active_peers, interrupted, run_command
else:
    from resource_limits import managed_context
    from build_platform import library_name, executable_name, allocated_bytes, exec_process
    from run_native_tests import active_peers, interrupted, run_command

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_files():
    paths = subprocess.check_output(
        ['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard']).split(b'\0')
    return {os.fsdecode(raw): sha(ROOT/os.fsdecode(raw)) for raw in paths
            if raw and (ROOT/os.fsdecode(raw)).is_file()}


def verify_test_artifacts(compile_log, library, benchmark):
    """Require this native test run's Cargo receipts for both downstream inputs."""
    found = set()
    with compile_log.open(encoding='utf-8') as lines:
        for line in lines:
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue  # Cargo also prints plain-text progress and diagnostics.
            if (message.get('reason') != 'compiler-artifact'
                    or Path(message.get('manifest_path', '')) != ROOT/'Cargo.toml'
                    or message.get('profile', {}).get('test') is not False):
                continue
            target = message['target']
            files = {Path(path) for path in message['filenames']}
            if target['name'] == 'rustmage' and 'cdylib' in target['kind'] and library in files:
                found.add(library)
            if (target['name'] == 'benchmark' and target['kind'] == ['bin']
                    and message.get('executable') and Path(message['executable']) == benchmark
                    and benchmark in files):
                found.add(benchmark)
    for path in (library, benchmark):
        if path not in found:
            raise RuntimeError(f'native test compilation did not report {path}')
        if path.resolve() != path or not path.is_file():
            raise RuntimeError(f'expected a regular Cargo test-profile output at {path}')


def snapshot_library(source, report_dir):
    """Publish a copy, never a hardlink into the mutable Cargo target."""
    if not source.is_file() or source.is_symlink():
        raise RuntimeError(f'build did not produce a regular shared library at {source}')
    directory = report_dir/'libraries'
    directory.mkdir()
    library = directory/source.name
    shutil.copy2(source, library)
    digest = sha(library)
    if digest != sha(source):
        raise RuntimeError('shared library changed while publishing its snapshot')
    library.chmod(library.stat().st_mode & ~0o222)
    return library, digest


def completed_native_report(path):
    native = json.loads(path.read_text())
    if not native.get('success') or not native.get('terminal') or native.get('active_processes'):
        raise RuntimeError('native assurance is not terminal and successful; preserve outputs')
    return native


def native_retirement(native):
    """Record reclamation from the terminal native runner."""
    count = native.get('retired_native_test_binaries')
    reclaimed = native.get('retired_native_test_bytes')
    if not isinstance(count, int) or count <= 0 or not isinstance(reclaimed, int) or reclaimed < 0:
        raise RuntimeError('native assurance did not retire its test binaries')
    if active_peers():
        raise RuntimeError('native test processes remain active; preserve outputs')
    return count, reclaimed


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    windows = sys.platform == 'win32'
    profiles = parser.add_mutually_exclusive_group()
    profiles.add_argument('--release', action='store_true',
                          help='also build the production library and run the bounded native benchmark')
    profiles.add_argument('--test-artifacts', action='store_true',
                          help='run simulation, Standard gameplay and requested Python checks with native test outputs')
    parser.add_argument('--python', choices=['none', 'smoke', 'full'], default='none',
                        help='run binding assurance against the shared library from this validation')
    parser.add_argument('--games', type=int, default=10,
                        help='native benchmark games with --release or --test-artifacts (default: 10)')
    parser.add_argument('--retire-native-binaries', action='store_true',
                        help='build/run/remove native test executables in bounded batches')
    parser.add_argument('--timeout', type=float,
                        default=11700 if windows else 6600 if sys.platform == 'darwin' else 5100,
                        help='total wall-clock budget after acquiring the build slot (default: %(default)s seconds)')
    targets = parser.add_mutually_exclusive_group()
    targets.add_argument('--target-dir', type=Path,
                         help='reuse this Cargo target (default: CARGO_TARGET_DIR or a worktree cache)')
    targets.add_argument('--fresh', action='store_true',
                         help='use and clean a disposable target for an intentional clean build')
    arguments = sys.argv[1:] if argv is None else argv
    args = parser.parse_args(arguments)
    if args.games < 1 or (not math.isfinite(args.timeout) or args.timeout <= 0):
        parser.error('--games and --timeout must be positive')
    if args.retire_native_binaries and not (args.release or args.test_artifacts or args.python != 'none'):
        parser.error('--retire-native-binaries requires a subsequent build or artifact check')
    os.chdir(ROOT)
    if managed_context() is None:
        exec_process([sys.executable, str(ROOT/'scripts/resource_guard.py'), '--',
                      sys.executable, str(Path(__file__).resolve()), *arguments])
    signal.signal(signal.SIGTERM, interrupted)
    cache = Path(os.environ.get('XDG_CACHE_HOME', Path.home()/'.cache')).expanduser().resolve()
    if cache == ROOT or ROOT in cache.parents:
        parser.error('XDG_CACHE_HOME must be outside the source checkout')
    cache.mkdir(parents=True, exist_ok=True)
    target = None
    target_preexisting = False
    if not args.fresh:
        identity = hashlib.sha256(os.fsencode(ROOT)).hexdigest()[:16]
        default = cache/'rustmage-targets'/identity/'target'
        target = Path(args.target_dir or os.environ.get('CARGO_TARGET_DIR') or default).expanduser().resolve()
        if target == ROOT or ROOT in target.parents:
            parser.error('--target-dir must be outside the source checkout')
        if (target/'report/result.json').exists():
            parser.error('legacy validation target pins library paths; choose a new reusable target')
        if target.exists() and not target.is_dir():
            parser.error('--target-dir must name a directory')
        target_preexisting = target.exists()
    run_dir = Path(tempfile.mkdtemp(prefix='rustmage-validation-', dir=cache))
    if args.fresh:
        target = run_dir/'target'
    target.mkdir(parents=True, exist_ok=True)
    report_dir = run_dir/'report'
    report_dir.mkdir()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), PYTHONDONTWRITEBYTECODE='1')
    # Game receipts belong to this immutable report, never the reusable Cargo cache.
    env.setdefault('RUSTMAGE_TDM_EVIDENCE_DIR', str(report_dir/'tdm-games'))
    files = source_files()
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    # Cold hosted runners link hundreds of integration targets serially.
    # Windows CI run 43775947252 took 4828s to link 872 targets and 1131s
    # for clippy. Allow runner variation while keeping every phase bounded.
    native_compile_timeout = 6000 if windows else 3300 if sys.platform == 'darwin' else 2100
    # Hosted macOS and Windows runners also pay process startup for each target.
    # Windows exhausted 1200s after 775/872 binaries, with no failed case.
    # This aggregate allowance does not relax the 180s per-case deadline.
    native_suite_timeout = 1800 if windows else 1200 if sys.platform == 'darwin' else 900
    commands = [
        ('source pins', [sys.executable, 'scripts/check_sources.py'], 45),
        ('format', [sys.executable, 'scripts/format_workspace.py', '--check'], 45),
        ('card format', [sys.executable, 'scripts/format_cards.py', '--check'], 90),
        ('script contracts', [sys.executable, 'scripts/test_assurance_runner.py'], 60),
        # Cold native toolchains on hosted Windows/macOS need time to populate
        # the per-set cache; compilation budgets are separate from game limits.
        ('clippy', ['cargo', 'clippy', '--workspace', '--all-targets', '--', '-D', 'warnings'],
         1500 if windows else 600),
        ('native assurance', [sys.executable, 'scripts/run_native_tests.py',
                              '--target-dir', str(target), '--report', str(report_dir/'native.json'),
                              '--test-timeout', '180', '--suite-timeout', str(native_suite_timeout),
                              '--compile-timeout', str(native_compile_timeout)]
                             + (['--retire-binaries'] if args.retire_native_binaries else []),
         native_compile_timeout + native_suite_timeout + 20),
    ]
    profile_dir = target/('debug' if args.test_artifacts else 'release')
    benchmark = profile_dir/executable_name('benchmark')
    source_library = (profile_dir/'deps' if args.test_artifacts else profile_dir)/library_name()
    if args.release or (args.python != 'none' and not args.test_artifacts):
        # A cold Windows release still compiles the whole pool independently of
        # the test profile. Reserve time for it after the complete native suite.
        release_build = ['cargo', 'build', '--release', '--lib']
        if args.release:
            release_build.extend(['--bin', 'benchmark'])
        commands.append(('release build', release_build, 1800 if windows else 900))
    if args.release or args.test_artifacts:
        commands.append(('simulation sample', [str(benchmark), str(args.games)], 15))
    if args.test_artifacts:
        commands.append(('Standard gameplay', [str(benchmark), '--standard-smoke', '1'], 30))
    if args.python != 'none':
        commands.append(('Python assurance', [sys.executable, 'scripts/python_tests.py',
                                               '--suite', args.python,
                                               '--report', str(report_dir/'python.json')], 30))
    report = dict(head=head, terminal=False, target=str(target), phases=[], timeout_seconds=args.timeout,
                  fresh_target=args.fresh, target_preexisting=target_preexisting, shared_libraries={},
                  game_evidence_directory=env['RUSTMAGE_TDM_EVIDENCE_DIR'],
                  input_sha256=hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest(),
                  retired_native_test_binaries=0, retired_native_test_bytes=0)
    started = time.monotonic()
    ok = False
    print(f'Validation report: {report_dir/"result.json"}', flush=True)
    try:
        for index, (name, command, limit) in enumerate(commands):
            remaining = args.timeout - (time.monotonic()-started)
            if remaining <= 0:
                report['budget_exceeded'] = True
                break
            phase = run_command(command, env=env, log=report_dir/f'{index:02}.log',
                                timeout=min(limit, remaining))
            phase['name'] = name
            report['phases'].append(phase)
            print(f"{index+1}/{len(commands)} {name}: exit={phase['exit']} {phase['seconds']:.3f}s", flush=True)
            if phase['exit']:
                print(Path(phase['log']).read_text()[-12000:], flush=True)
                break
            if name == 'native assurance' and (args.retire_native_binaries or args.test_artifacts):
                native = completed_native_report(report_dir/'native.json')
                if args.retire_native_binaries:
                    count, reclaimed = native_retirement(native)
                    report['retired_native_test_binaries'] = count
                    report['retired_native_test_bytes'] = reclaimed
                    print(f'Retired {count} completed native test executables ({reclaimed} allocated bytes)',
                          flush=True)
                if args.test_artifacts:
                    verify_test_artifacts(report_dir/'native-compile.log', source_library, benchmark)
            if name == ('native assurance' if args.test_artifacts else 'release build'):
                if active_peers():
                    raise RuntimeError(f'{name} left active processes; preserve outputs')
                library, digest = snapshot_library(source_library, report_dir)
                report['shared_libraries'][str(library)] = digest
                env['RUSTMAGE_LIBRARY'] = str(library)
            if name == 'simulation sample':
                report['simulation'] = json.loads(Path(phase['log']).read_text())
        else:
            ok = True
    except KeyboardInterrupt:
        report['interrupted'] = True
    except (OSError, RuntimeError, ValueError) as error:
        report['error'] = str(error)
        print(str(error), file=sys.stderr, flush=True)
    finally:
        current = source_files()
        report['changed'] = sorted(path for path in files.keys() | current.keys()
                                   if files.get(path) != current.get(path))
        report['files_verified'] = len(files)
        active = sorted(active_peers())
        report['active_processes'] = active
        report['success'] = ok and not report['changed'] and not active
        report['terminal'] = not active
        report['seconds'] = time.monotonic()-started
        freed = 0
        if args.fresh and not active:
            freed = sum(allocated_bytes(path) for path in target.rglob('*')
                        if path.is_file() and not path.is_symlink())
            shutil.rmtree(target)
        for path, digest in report['shared_libraries'].items():
            if sha(Path(path)) != digest:
                report['success'] = False
                report['error'] = 'retained shared library changed: '+path
        report['cleaned_allocated_bytes'] = freed
        report['complete_cards'] = json.loads((ROOT/'data/standard/coverage.json').read_text())['implemented_standard_cards']
        (report_dir/'result.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(dict(success=report['success'], report=str(report_dir/'result.json'),
                              seconds=report['seconds'], complete_cards=report['complete_cards'],
                              cleaned_bytes=freed)), flush=True)
    return 0 if report['success'] else 1


if __name__ == '__main__':
    sys.exit(main())
