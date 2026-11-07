#!/usr/bin/env python3
"""Serialize builds across worktrees with a 6 GiB memory budget.

Linux uses a hard systemd cgroup limit; other hosts use sampled RSS supervision.
Preserves common Rust/C toolchain inputs. Reports
survive reboot under ~/.cache/rustmage-build-reports. Nested managed commands
reuse the same cgroup and lock rather than deadlocking.
"""
import argparse
import contextlib
import errno
import json
import os
from pathlib import Path
import subprocess
import shutil
import sys
import time
import uuid
if __package__:
    from .resource_limits import MAX_MEMORY, bounded_group, managed_context
    from .build_platform import cargo_environment
else:
    from resource_limits import MAX_MEMORY, bounded_group, managed_context
    from build_platform import cargo_environment


@contextlib.contextmanager
def shared_build_slot():
    lock_path = Path.home()/'.cache/rustmage-build.lock'
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    with lock_path.open('a+b') as lock:
        print('Waiting for the shared build slot...', file=sys.stderr)
        if os.name == 'nt':
            import msvcrt
            if lock_path.stat().st_size == 0:
                lock.write(b'\0')
                lock.flush()
            while True:
                try:
                    lock.seek(0)
                    msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
                    break
                except OSError as error:
                    if error.errno not in (errno.EACCES, errno.EAGAIN, errno.EDEADLK):
                        raise
                    time.sleep(0.1)
        else:
            import fcntl
            fcntl.flock(lock, fcntl.LOCK_EX)
        try:
            yield
        finally:
            if os.name == 'nt':
                lock.seek(0)
                msvcrt.locking(lock.fileno(), msvcrt.LK_UNLCK, 1)

def inside(command, report, acquire=True):
    group = bounded_group()
    if group is None:
        raise RuntimeError('refusing to execute without an enforced memory cgroup')
    env = cargo_environment()
    started = time.monotonic()
    with shared_build_slot() if acquire else contextlib.nullcontext():
        result = subprocess.run(command, env=env)
    data = dict(command=command, terminal=True, exit_code=result.returncode,
                backend='cgroup', memory_limit_kind='hard',
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
    parser.add_argument('--backend', choices=['auto', 'cgroup', 'portable'], default='auto',
                        help='auto uses an available Linux user systemd manager, otherwise portable supervision')
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
    context = managed_context()
    if context is not None:
        if args.backend == 'cgroup' and context['backend'] != 'cgroup':
            parser.error('cgroup backend cannot run inside a portable supervisor')
        started = time.monotonic()
        result = subprocess.run(command, env=cargo_environment())
        report.write_text(json.dumps(dict(command=command, terminal=True,
            exit_code=result.returncode, seconds=time.monotonic()-started,
            backend=context['backend'], memory_limit_kind=context['memory_limit_kind'],
            memory_max_bytes=MAX_MEMORY, nested=True), indent=2)+'\n')
        return result.returncode
    if os.environ.get('RUSTMAGE_GUARD_PORT'):
        parser.error('inherited build supervisor is unavailable; refusing unmanaged execution')
    systemd_available = False
    if args.backend != 'portable' and sys.platform == 'linux' and shutil.which('systemd-run') and shutil.which('systemctl'):
        try:
            systemd_available = subprocess.run(
                ['systemctl', '--user', 'show-environment'], stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL, timeout=10).returncode == 0
        except (OSError, subprocess.TimeoutExpired):
            pass
    if args.backend == 'cgroup' and not systemd_available:
        parser.error('cgroup backend requires a running Linux user systemd manager')
    if args.backend == 'portable' or not systemd_available:
        if __package__:
            from .portable_guard import run
        else:
            from portable_guard import run
        with shared_build_slot():
            return run(command, report, cargo_environment())
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
                            'PYTHONDONTWRITEBYTECODE', 'PYTHONUTF8', 'XDG_CACHE_HOME',
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
    try:
        sys.exit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        sys.exit('Build guard failed: '+str(error))
