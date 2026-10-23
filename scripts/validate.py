#!/usr/bin/env python3
"""Run bounded card/interaction assurance with separate build and execution timings.

Each run owns a unique Cargo target. Terminal cleanup retains exact shared
libraries, logs and JSON reports under ~/.cache. Keep source inputs fixed until
this command has terminated, including after an observation timeout.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

from resource_limits import bounded_group
from run_native_tests import descendants, interrupted, process_running, run_command

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_files():
    paths = subprocess.check_output(
        ['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard']).split(b'\0')
    return {os.fsdecode(raw): sha(ROOT/os.fsdecode(raw)) for raw in paths
            if raw and (ROOT/os.fsdecode(raw)).is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true',
                        help='also build production library and run the bounded native benchmark')
    parser.add_argument('--python', choices=['none', 'smoke', 'full'], default='none',
                        help='run binding assurance against this run\'s release library')
    parser.add_argument('--games', type=int, default=10,
                        help='native benchmark games, only with --release (default: 10)')
    parser.add_argument('--timeout', type=float, default=600,
                        help='total wall-clock budget after acquiring the build slot (seconds)')
    args = parser.parse_args()
    if args.games < 1 or args.timeout <= 0:
        parser.error('--games and --timeout must be positive')
    os.chdir(ROOT)
    if bounded_group() is None:
        os.execv(sys.executable, [sys.executable, str(ROOT/'scripts/resource_guard.py'), '--',
                                 sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    signal.signal(signal.SIGTERM, interrupted)
    cache = Path(os.environ.get('XDG_CACHE_HOME', Path.home()/'.cache'))
    cache.mkdir(parents=True, exist_ok=True)
    target = Path(tempfile.mkdtemp(prefix='rustmage-validation-', dir=cache))
    report_dir = target/'report'
    report_dir.mkdir()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), PYTHONDONTWRITEBYTECODE='1')
    files = source_files()
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    commands = [
        ('source pins', [sys.executable, 'scripts/check_sources.py'], 45),
        ('format', ['cargo', 'fmt', '--all', '--check'], 45),
        ('card format', ['rustfmt', '--edition', '2024', '--check',
                         *[str(p) for p in sorted((ROOT/'src/card_definitions').rglob('*.rs'))]], 45),
        ('script contracts', [sys.executable, '-m', 'unittest', 'discover', '-s', 'scripts',
                               '-p', 'test_*.py'], 30),
        ('clippy', ['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings'], 180),
        ('native assurance', [sys.executable, 'scripts/run_native_tests.py',
                              '--target-dir', str(target), '--report', str(report_dir/'native.json'),
                              '--suite-timeout', '30', '--compile-timeout', '360'], 395),
    ]
    if args.release or args.python != 'none':
        commands.append(('release build', ['cargo', 'build', '--release', '--lib', '--bin', 'benchmark'], 360))
    if args.release:
        commands.append(('simulation sample', [str(target/'release/benchmark'), str(args.games)], 15))
    if args.python != 'none':
        env['RUSTMAGE_LIBRARY'] = str(target/'release/librustmage.so')
        commands.append(('Python assurance', [sys.executable, 'scripts/python_tests.py',
                                               '--suite', args.python,
                                               '--report', str(report_dir/'python.json')], 30))
    report = dict(head=head, terminal=False, target=str(target), phases=[], timeout_seconds=args.timeout)
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
            if name == 'simulation sample':
                report['simulation'] = json.loads(Path(phase['log']).read_text())
        else:
            ok = True
    except KeyboardInterrupt:
        report['interrupted'] = True
    finally:
        current = source_files()
        report['changed'] = sorted(path for path in files.keys() | current.keys()
                                   if files.get(path) != current.get(path))
        report['files_verified'] = len(files)
        active = [pid for pid in descendants(os.getpid()) if process_running(pid)]
        report['active_processes'] = active
        report['success'] = ok and not report['changed'] and not active
        report['terminal'] = not active
        report['seconds'] = time.monotonic()-started
        report['shared_libraries'] = {
            str(path): sha(path) for path in target.rglob('*')
            if not active and path.is_file() and not path.is_symlink() and '.so' in path.name}
        freed = 0
        for path in target.rglob('*'):
            if (not active and path.is_file() and not path.is_symlink() and report_dir not in path.parents
                    and str(path) not in report['shared_libraries']):
                freed += path.stat().st_blocks*512
                path.unlink()
        for path, digest in report['shared_libraries'].items():
            if sha(Path(path)) != digest:
                raise RuntimeError('retained shared library changed: '+path)
        report['cleaned_allocated_bytes'] = freed
        report['complete_cards'] = json.loads((ROOT/'data/standard/coverage.json').read_text())['implemented_standard_cards']
        (report_dir/'result.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(dict(success=report['success'], report=str(report_dir/'result.json'),
                              seconds=report['seconds'], complete_cards=report['complete_cards'],
                              cleaned_bytes=freed)), flush=True)
    return 0 if report['success'] else 1


if __name__ == '__main__':
    sys.exit(main())
