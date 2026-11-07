"""Native build artifact names and command sizes; no target-OS assumptions."""
import os
import hashlib
import platform
import subprocess
from pathlib import Path
import sys


def library_name(platform=None):
    platform = sys.platform if platform is None else platform
    if platform == 'win32':
        return 'rustmage.dll'
    if platform == 'darwin':
        return 'librustmage.dylib'
    return 'librustmage.so'


def executable_name(name, platform=None):
    return name + ('.exe' if (platform or sys.platform) == 'win32' else '')


def allocated_bytes(path):
    stat = Path(path).stat()
    return stat.st_blocks * 512 if hasattr(stat, 'st_blocks') else stat.st_size


def exec_process(command):
    """Preserve child completion/status without Windows CRT exec overlays.

    Windows exec*e can also fault while constructing the environment block:
    https://github.com/python/cpython/issues/137934
    subprocess uses CreateProcess and waits for the real exit status instead.
    """
    if os.name == 'nt':
        raise SystemExit(subprocess.run(command).returncode)
    os.execvpe(command[0], command, os.environ)


def argument_batches(arguments, budget=16000):
    """Stay below Windows' command-line limit, including quotes and UTF-16."""
    batch, size = [], 0
    for argument in map(str, arguments):
        cost = len(argument.encode('utf-16-le')) // 2 + 3
        if batch and size + cost > budget:
            yield batch
            batch, size = [], 0
        batch.append(argument)
        size += cost
    if batch:
        yield batch


def cargo_environment():
    env = dict(os.environ, CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1',
               PYTHONUTF8='1', PYTHONDONTWRITEBYTECODE='1', RUSTMAGE_PYTHON=sys.executable)
    # Shebang wrappers must use the same Python environment as the supervisor.
    env['PATH'] = str(Path(sys.executable).parent) + os.pathsep + env.get('PATH', '')
    root = Path(__file__).resolve().parents[1]
    cache = Path(env.get('XDG_CACHE_HOME', Path.home()/'.cache')).expanduser()
    identity = hashlib.sha256(os.fsencode(root)).hexdigest()[:16]
    env.setdefault('CARGO_TARGET_DIR', str(cache/'rustmage-targets'/identity/'target'))
    if sys.platform == 'win32':
        launcher = Path(__file__).with_name('rustc_guard_launcher.rs')
        digest = hashlib.sha256(launcher.read_bytes()+platform.machine().encode()).hexdigest()[:16]
        env['RUSTC_WRAPPER'] = str(cache/'rustmage-guard-launchers'/digest/'rustc_guard.exe')
        env['RUSTMAGE_RUSTC_GUARD'] = str(Path(__file__).with_name('rustc_guard.py').resolve())
    return env
