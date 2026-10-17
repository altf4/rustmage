#!/usr/bin/env python3
"""Serialize builds across worktrees and bound their entire process tree to 6 GiB.

Uses a user systemd cgroup and preserves common Rust/C toolchain inputs. Fails closed if limits cannot be installed. Reports
survive reboot under ~/.cache/rustmage-build-reports. Nested managed commands
reuse the same cgroup and lock rather than deadlocking.
"""
import argparse
import fcntl
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid
if __package__:
    from .resource_limits import MAX_MEMORY, bounded_group
else:
    from resource_limits import MAX_MEMORY, bounded_group

def inside(command, report, acquire=True):
    group = bounded_group()
    if group is None:
        raise RuntimeError('refusing to execute without an enforced memory cgroup')
    env = dict(os.environ, CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1')
    started = time.monotonic()
    cache = Path(os.environ.get('XDG_CACHE_HOME', Path.home()/'.cache'))
    lock_path = Path.home()/'.cache/rustmage-build.lock'
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    with lock_path.open('a') as lock:
        if acquire:
            print('Waiting for the shared build slot...', file=sys.stderr)
            fcntl.flock(lock, fcntl.LOCK_EX)
        result = subprocess.run(command, env=env)
    data = dict(command=command, terminal=True, exit_code=result.returncode,
                seconds=time.monotonic()-started, memory_max_bytes=int((group/'memory.max').read_text()),
                memory_peak_bytes=int((group/'memory.peak').read_text()),
                memory_events=(group/'memory.events').read_text(), cargo_jobs=1, test_threads=1)
    report.write_text(json.dumps(data, indent=2)+'\n')
    print('Resource report: '+str(report), file=sys.stderr)
    return result.returncode

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    parser.add_argument('--report', type=Path)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        parser.error('supply a command after --')
    cache = Path(os.environ.get('XDG_CACHE_HOME', Path.home()/'.cache'))
    reports = cache/'rustmage-build-reports'
    reports.mkdir(parents=True, exist_ok=True)
    report = args.report or reports/(uuid.uuid4().hex+'.json')
    report = report.absolute()
    if not args.inside and report.exists():
        parser.error('report already exists; choose a fresh report path')
    report.parent.mkdir(parents=True, exist_ok=True)
    if args.inside:
        return inside(command, report)
    if bounded_group() is not None:
        return inside(command, report, acquire=False)
    # The parent slice also bounds orphaned units if the desktop/outer runner dies.
    subprocess.run(['systemctl', '--user', 'start', 'rustmage-builds.slice'], check=True)
    subprocess.run(['systemctl', '--user', 'set-property', '--runtime', 'rustmage-builds.slice',
                    'MemoryHigh=5G', 'MemoryMax='+str(MAX_MEMORY), 'MemorySwapMax=0',
                    'CPUQuota=200%', 'TasksMax=256'], check=True)
    unit = 'rustmage-build-'+uuid.uuid4().hex
    forwarded = ['--setenv='+key+'='+value for key, value in os.environ.items()
                 if key.startswith(('CARGO_PROFILE_', 'CARGO_TARGET_', 'RUSTMAGE_')) or key in ('PATH', 'CARGO_TARGET_DIR', 'CARGO_HOME', 'RUSTUP_HOME',
                            'RUSTUP_TOOLCHAIN', 'RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTFLAGS',
                            'CARGO_INCREMENTAL', 'CC', 'CXX', 'AR', 'CFLAGS', 'CXXFLAGS',
                            'LDFLAGS', 'LD_LIBRARY_PATH', 'PKG_CONFIG_PATH', 'PYTHONPATH',
                            'PYTHONDONTWRITEBYTECODE', 'XDG_CACHE_HOME',
                            'RUSTMAGE_CATALOGUE_SNAPSHOT')]
    invocation = ['systemd-run', '--user', '--wait', '--pipe', '--collect', '--unit='+unit,
                  '--slice=rustmage-builds.slice', '--working-directory='+str(Path.cwd()),
                  '-p', 'MemoryHigh=5G', '-p', 'MemoryMax='+str(MAX_MEMORY),
                  '-p', 'MemorySwapMax=0', '-p', 'CPUQuota=200%', '-p', 'TasksMax=128',
                  '-p', 'OOMPolicy=kill', '-p', 'KillMode=control-group', *forwarded,
                  sys.executable, str(Path(__file__).resolve()), '--inside', '--report', str(report), '--', *command]
    result = subprocess.run(invocation)
    if not report.exists():
        report.write_text(json.dumps(dict(command=command, terminal=True, exit_code=result.returncode,
            memory_max_bytes=MAX_MEMORY, incomplete_metrics=True,
            note='Unit failed or exceeded its memory limit before it could write metrics; inspect systemd journal.'), indent=2)+'\n')
        print('Resource report: '+str(report), file=sys.stderr)
    return result.returncode

if __name__ == '__main__':
    sys.exit(main())
