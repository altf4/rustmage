#!/usr/bin/env python3
"""Run bounded card coverage with separate build and execution timings.

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
    from .resource_limits import bounded_group
    from .run_native_tests import active_peers, interrupted, run_command
else:
    from resource_limits import bounded_group
    from run_native_tests import active_peers, interrupted, run_command

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_files():
    paths = subprocess.check_output(
        ['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard']).split(b'\0')
    return {os.fsdecode(raw): sha(ROOT/os.fsdecode(raw)) for raw in paths
            if raw and (ROOT/os.fsdecode(raw)).is_file()}


def snapshot_library(target, report_dir, profile='release'):
    """Publish a copy, never a hardlink into the mutable Cargo target."""
    source = target/profile/'librustmage.so'
    if not source.is_file() or source.is_symlink():
        raise RuntimeError(f'{profile} build did not produce a regular librustmage.so')
    directory = report_dir/'libraries'
    directory.mkdir()
    library = directory/source.name
    shutil.copy2(source, library)
    digest = sha(library)
    if digest != sha(source):
        raise RuntimeError('shared library changed while publishing its snapshot')
    library.chmod(library.stat().st_mode & ~0o222)
    return library, digest


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true',
                        help='also build production library and run the bounded native benchmark')
    parser.add_argument('--python', choices=['none', 'smoke', 'full'], default='none',
                        help='run binding contracts against this run\'s library (debug unless --release)')
    parser.add_argument('--games', type=int, default=10,
                        help='native benchmark games, only with --release (default: 10)')
    parser.add_argument('--timeout', type=float, default=600,
                        help='total wall-clock budget after acquiring the build slot (seconds)')
    targets = parser.add_mutually_exclusive_group()
    targets.add_argument('--target-dir', type=Path,
                         help='reuse this Cargo target (default: CARGO_TARGET_DIR or a worktree cache)')
    targets.add_argument('--fresh', action='store_true',
                         help='use and clean a disposable target for an intentional clean build')
    arguments = sys.argv[1:] if argv is None else argv
    args = parser.parse_args(arguments)
    if args.games < 1 or (not math.isfinite(args.timeout) or args.timeout <= 0):
        parser.error('--games and --timeout must be positive')
    os.chdir(ROOT)
    if bounded_group() is None:
        os.execv(sys.executable, [sys.executable, str(ROOT/'scripts/resource_guard.py'), '--',
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
    files = source_files()
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    commands = [
        ('source pins', [sys.executable, 'scripts/check_sources.py'], 45),
        ('format', ['cargo', 'fmt', '--all', '--check'], 45),
        ('card format', ['rustfmt', '--edition', '2024', '--check',
                         *[str(p) for p in sorted((ROOT/'src/card_definitions').rglob('*.rs'))]], 45),
        ('tool contracts', [sys.executable, 'scripts/test_tools.py'], 30),
        ('clippy', ['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings'], 180),
        ('card coverage', [sys.executable, 'scripts/run_native_tests.py',
                              '--target-dir', str(target), '--report', str(report_dir/'native.json'),
                              '--suite-timeout', '30', '--compile-timeout', '360'], 400),
    ]
    if args.release:
        commands.append(('release build', ['cargo', 'build', '--release', '--lib', '--bin', 'benchmark'], 360))
    elif args.python != 'none':
        commands.append(('binding build', ['cargo', 'build', '--lib'], 180))
    if args.release:
        commands.append(('simulation sample', [str(target/'release/benchmark'), str(args.games)], 15))
    if args.python != 'none':
        commands.append(('Python assurance', [sys.executable, 'scripts/python_tests.py',
                                               '--suite', args.python,
                                               '--report', str(report_dir/'python.json')], 30))
    report = dict(head=head, terminal=False, target=str(target), phases=[], timeout_seconds=args.timeout,
                  fresh_target=args.fresh, target_preexisting=target_preexisting, shared_libraries={},
                  input_sha256=hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest())
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
            if name in ('release build', 'binding build'):
                if active_peers():
                    raise RuntimeError('release build left active processes; preserve outputs')
                library, digest = snapshot_library(target, report_dir, 'release' if args.release else 'debug')
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
            freed = sum(path.stat().st_blocks*512 for path in target.rglob('*')
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
