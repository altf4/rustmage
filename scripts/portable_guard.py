"""Portable build supervision.

RSS is sampled, not a kernel allocation limit. An inherited random run marker
finds detached children; process birth times prevent signalling recycled PIDs.
Only a live supervisor can admit a compiler. This is an accident guard, not a
security boundary against another program running as the same user.
"""
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import socketserver
import subprocess
import sys
import threading
import time

try:
    import psutil
except ImportError:
    psutil = None

if __package__:
    from .resource_limits import MAX_MEMORY
else:
    from resource_limits import MAX_MEMORY

INTERVAL = 0.1
TOKEN = 'RUSTMAGE_GUARD_TOKEN'


def require_psutil():
    if psutil is None:
        raise RuntimeError('portable builds require psutil; install with: '
                           'python3 -m pip install -r scripts/requirements-build.txt')


def running(process):
    try:
        return process.is_running() and process.status() not in (psutil.STATUS_ZOMBIE, psutil.STATUS_DEAD)
    except psutil.NoSuchProcess:
        return False
    except psutil.AccessDenied:
        # Unknown liveness is not proof of exit. Keep it in cleanup/reporting.
        return True


def marked_processes(token):
    """Recover orphaned/new-session descendants without logging environments."""
    require_psutil()
    username = psutil.Process().username()
    found = []
    for process in psutil.process_iter(['pid', 'username']):
        if process.info['username'] != username:
            continue
        try:
            if running(process) and process.environ().get(TOKEN) == token:
                found.append(process)
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    return found


def stop_processes(processes):
    """psutil checks PID birth times before sending signals on every platform."""
    processes = list(processes)
    for process in processes:
        try:
            process.terminate()
        except psutil.NoSuchProcess:
            pass
        except psutil.AccessDenied:
            pass
    try:
        _, alive = psutil.wait_procs(processes, timeout=0.5)
    except psutil.Error:
        alive = [process for process in processes if running(process)]
    for process in alive:
        try:
            process.kill()
        except psutil.NoSuchProcess:
            pass
        except psutil.AccessDenied:
            pass
    try:
        _, alive = psutil.wait_procs(alive, timeout=3)
    except psutil.Error:
        alive = [process for process in alive if running(process)]
    if any(running(process) for process in alive):
        raise RuntimeError('managed descendants remain active; preserve outputs')


def supervisor_request():
    """A forged boolean or stale port cannot admit raw Cargo compilation."""
    try:
        port = int(os.environ['RUSTMAGE_GUARD_PORT'])
        token = os.environ[TOKEN]
        with socket.create_connection(('127.0.0.1', port), timeout=5) as connection:
            request = dict(token=token, pid=os.getpid())
            connection.sendall(json.dumps(request).encode() + b'\n')
            with connection.makefile('rb') as response:
                data = json.loads(response.readline(1024 * 1024))
        if data.get('admitted') and data.get('memory_max_bytes') == MAX_MEMORY:
            return data
    except (OSError, ValueError, KeyError):
        pass
    return None


class Supervisor:
    def __init__(self, token):
        require_psutil()
        self.token = token
        self.known = {}
        self.lock = threading.RLock()
        self.peak = 0
        self.exceeded = False

    def sample(self):
        with self.lock:
            for process in marked_processes(self.token):
                try:
                    self.known[(process.pid, process.create_time())] = process
                except psutil.NoSuchProcess:
                    pass
            # Keep children even if they alter the inherited environment later.
            for process in list(self.known.values()):
                try:
                    for child in process.children(recursive=True):
                        try:
                            self.known[(child.pid, child.create_time())] = child
                        except psutil.NoSuchProcess:
                            pass
                except psutil.NoSuchProcess:
                    pass
            self.known = {key: process for key, process in self.known.items() if running(process)}
            memory = 0
            for process in self.known.values():
                try:
                    memory += process.memory_info().rss
                except psutil.NoSuchProcess:
                    pass
            self.peak = max(memory, self.peak)
            self.exceeded |= memory > MAX_MEMORY
            return list(self.known.values())

    def context(self, pid):
        members = self.sample()
        return dict(admitted=any(process.pid == pid for process in members),
                    backend='portable', memory_limit_kind='sampled-rss',
                    memory_max_bytes=MAX_MEMORY, sample_interval_seconds=INTERVAL,
                    supervisor_pid=os.getpid(), members=[process.pid for process in members],
                    identities=[[process.pid, process.create_time()] for process in members])

    def stop(self, exclude=()):
        deadline = time.monotonic() + 5
        while True:
            try:
                members = self.sample()
            except psutil.Error:
                # Do not let a failed telemetry read abandon known children.
                members = list(self.known.values())
            members = [p for p in members if p.pid not in exclude and running(p)]
            if not members:
                return
            stop_processes(members)
            if time.monotonic() > deadline:
                raise RuntimeError('managed descendants remain active; preserve outputs')


def serve(supervisor):
    class Handler(socketserver.StreamRequestHandler):
        def handle(self):
            self.connection.settimeout(5)
            try:
                request = json.loads(self.rfile.readline(4096))
                if not secrets.compare_digest(request.get('token', ''), supervisor.token):
                    result = dict(admitted=False)
                else:
                    result = supervisor.context(int(request['pid']))
                self.wfile.write(json.dumps(result).encode() + b'\n')
            except (OSError, ValueError, KeyError, TypeError, psutil.Error):
                return

    server = socketserver.ThreadingTCPServer(('127.0.0.1', 0), Handler)
    server.daemon_threads = True
    threading.Thread(target=server.serve_forever, kwargs={'poll_interval': INTERVAL}, daemon=True).start()
    return server


def run_child(command, parent):
    child = subprocess.Popen(command)
    try:
        while child.poll() is None:
            if not running(parent):
                stop_processes(p for p in marked_processes(os.environ[TOKEN]) if p.pid != os.getpid())
                return 1
            time.sleep(INTERVAL)
        if not running(parent):
            stop_processes(p for p in marked_processes(os.environ[TOKEN]) if p.pid != os.getpid())
            return 1
        return child.returncode
    except BaseException:
        stop_processes(p for p in marked_processes(os.environ[TOKEN]) if p.pid != os.getpid())
        raise


def worker(command):
    """Wait for admission before executing, and clean up if the supervisor dies."""
    require_psutil()
    parent = psutil.Process(int(os.environ['RUSTMAGE_GUARD_PARENT']))
    expected_birth = float(os.environ['RUSTMAGE_GUARD_PARENT_BIRTH'])
    if parent.create_time() != expected_birth or supervisor_request() is None:
        raise RuntimeError('build supervisor unavailable before command start')
    if sys.platform == 'win32':
        launcher = Path(os.environ['RUSTC_WRAPPER'])
        if not launcher.is_file():
            launcher.parent.mkdir(parents=True, exist_ok=True)
            pending = launcher.with_suffix('.pending.exe')
            source = Path(__file__).with_name('rustc_guard_launcher.rs')
            try:
                # This tiny bootstrap is inside the same supervisor and lock.
                code = run_child(['rustc', '--edition=2021', str(source), '-o', str(pending)], parent)
                if code:
                    return code
                pending.replace(launcher)
            finally:
                pending.unlink(missing_ok=True)
    if supervisor_request() is None:
        raise RuntimeError('build supervisor unavailable before command start')
    return run_child(command, parent)


def run(command, report, env):
    require_psutil()
    supervisor = Supervisor(secrets.token_hex(32))
    server = serve(supervisor)
    parent = psutil.Process()
    env = dict(env, RUSTMAGE_GUARD_TOKEN=supervisor.token,
               RUSTMAGE_GUARD_PORT=str(server.server_address[1]),
               RUSTMAGE_GUARD_PARENT=str(parent.pid),
               RUSTMAGE_GUARD_PARENT_BIRTH=str(parent.create_time()))
    started = time.monotonic()
    code, orphaned, error = 1, False, None
    orphan_details = []
    process = None
    def interrupted(*_):
        raise KeyboardInterrupt

    previous_handler = signal.signal(signal.SIGTERM, interrupted)
    print('Portable guard: sampled 6 GiB aggregate RSS budget (not a hard allocation cap).', file=sys.stderr)
    try:
        process = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), '--worker', *command],
                                   env=env, start_new_session=os.name != 'nt')
        try:
            child = psutil.Process(process.pid)
            with supervisor.lock:
                supervisor.known[(child.pid, child.create_time())] = child
        except psutil.NoSuchProcess:
            pass
        while process.poll() is None:
            supervisor.sample()
            if supervisor.exceeded:
                error = 'sampled aggregate memory budget exceeded'
                supervisor.stop()
                break
            time.sleep(INTERVAL)
        code = process.wait()
        orphans = supervisor.sample()
        orphaned = bool(orphans)
        if orphaned:
            for orphan in orphans:
                try:
                    orphan_details.append(orphan.as_dict(
                        attrs=['pid', 'ppid', 'name', 'exe', 'create_time'], ad_value=None))
                except psutil.NoSuchProcess:
                    pass
            supervisor.stop()
        if supervisor.exceeded or orphaned:
            code = code or 1
    except (KeyboardInterrupt, OSError, RuntimeError, psutil.Error) as failure:
        error = str(failure) or 'interrupted'
        code = 130 if isinstance(failure, KeyboardInterrupt) else 1
    finally:
        try:
            supervisor.stop()
            if process is not None:
                process.wait(timeout=5)
        except (RuntimeError, psutil.Error, subprocess.TimeoutExpired) as failure:
            error, code = str(failure), 1
            # Retain and use the direct child handle even if telemetry failed.
            if process is not None and process.poll() is None:
                try:
                    process.kill()
                    process.wait(timeout=5)
                except (OSError, subprocess.TimeoutExpired):
                    pass
        finally:
            try:
                members = supervisor.sample()
            except psutil.Error as failure:
                members = [p for p in supervisor.known.values() if running(p)]
                error, code = str(failure), 1
            if process is not None and process.poll() is None and process.pid not in [p.pid for p in members]:
                members.append(process)
            server.shutdown()
            server.server_close()
            signal.signal(signal.SIGTERM, previous_handler)
            data = dict(command=command, terminal=not members, exit_code=code,
                        seconds=time.monotonic()-started, backend='portable',
                        memory_limit_kind='sampled-rss', memory_max_bytes=MAX_MEMORY,
                        memory_peak_bytes=supervisor.peak, sample_interval_seconds=INTERVAL,
                        memory_budget_exceeded=supervisor.exceeded,
                        orphaned_processes=orphaned, active_processes=[p.pid for p in members],
                        orphaned_process_details=orphan_details,
                        cargo_jobs=1, test_threads=1, error=error)
            report.write_text(json.dumps(data, indent=2)+'\n', encoding='utf-8')
            print('Resource report: '+str(report), file=sys.stderr)
    return code


if __name__ == '__main__':
    if sys.argv[1:2] != ['--worker']:
        sys.exit('invoke scripts/resource_guard.py')
    sys.exit(worker(sys.argv[2:]))
