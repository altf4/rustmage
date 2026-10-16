"""In-process rustmage binding. No server, socket, JVM, or third-party packages.

Build with `cargo build --release`, then import with PYTHONPATH=python.
Each Game owns a native handle; different handles may run on different threads.
The lock serializes ctypes calls (ctypes releases the GIL during native calls).
"""
from __future__ import annotations

import ctypes
import json
from pathlib import Path
import sys
import threading


class EngineError(RuntimeError):
    pass


class CapacityError(EngineError):
    """The exact action could not fit; game state and revision are unchanged."""
    def __init__(self, message, resource):
        super().__init__(message)
        self.resource = resource


class Game:
    def __init__(self, decks, seed=0, starting_player="P0", deck_rules="Constructed", *, library=None):
        self._lock = threading.RLock()
        self._handle = None
        if library is None:
            filename = {"linux": "librustmage.so", "darwin": "librustmage.dylib", "win32": "rustmage.dll"}[sys.platform]
            library = Path(__file__).resolve().parents[1] / "target" / "release" / filename
        self._lib = ctypes.CDLL(str(library))
        self._configure()
        error = ctypes.c_void_p()
        config = json.dumps(dict(decks=decks, seed=seed, starting_player=starting_player, deck_rules=deck_rules)).encode()
        self._handle = self._lib.rm_new(config, ctypes.byref(error))
        if not self._handle:
            message = self._take_string(error.value) if error.value else "engine creation failed"
            raise EngineError(message)

    def _configure(self):
        lib = self._lib
        lib.rm_abi_version.argtypes = []
        lib.rm_abi_version.restype = ctypes.c_uint32
        if lib.rm_abi_version() != 1:
            raise EngineError("unsupported native ABI")
        lib.rm_new.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_void_p)]
        lib.rm_new.restype = ctypes.c_void_p
        lib.rm_request.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        lib.rm_request.restype = ctypes.c_void_p
        lib.rm_clone.argtypes = [ctypes.c_void_p]
        lib.rm_clone.restype = ctypes.c_void_p
        lib.rm_free.argtypes = [ctypes.c_void_p]
        lib.rm_free.restype = None
        lib.rm_string_free.argtypes = [ctypes.c_void_p]
        lib.rm_string_free.restype = None

    def _take_string(self, pointer):
        try:
            return ctypes.string_at(pointer).decode("utf-8")
        finally:
            self._lib.rm_string_free(pointer)

    def request(self, op, **fields):
        with self._lock:
            if not self._handle:
                raise EngineError("game is closed")
            payload = json.dumps(dict(op=op, **fields)).encode()
            pointer = self._lib.rm_request(self._handle, payload)
            if not pointer:
                raise EngineError("null native response")
            response = json.loads(self._take_string(pointer))
            if response.get("error"):
                if response.get("capacity"):
                    raise CapacityError(response["error"], response["capacity"]["resource"])
                raise EngineError(response["error"])
            return response

    def observe(self, viewer):
        return self.request("observe", viewer=viewer)["observation"]

    def step(self, observation, index):
        """Indices are tied to the supplied observation's revision and actor."""
        return self.request("step", actor=observation["actor"], revision=observation["revision"], index=index,
                            viewer=observation["actor"])["observation"]

    def act(self, observation, action):
        """Submit a concrete action, including any integer in damage_assignment bounds."""
        return self.request("act", actor=observation["actor"], revision=observation["revision"],
                            action=action, viewer=observation["actor"])["observation"]

    def snapshot(self):
        return self.request("snapshot")["snapshot"]

    def restore(self, snapshot):
        self.request("restore", snapshot=snapshot)

    def drop_snapshot(self, snapshot):
        self.request("drop_snapshot", snapshot=snapshot)

    def clone(self):
        with self._lock:
            if not self._handle:
                raise EngineError("game is closed")
            other = object.__new__(Game)
            other._lock = threading.RLock()
            other._lib = self._lib
            other._handle = self._lib.rm_clone(self._handle)
            if not other._handle:
                raise EngineError("native clone failed")
            return other

    def close(self):
        with self._lock:
            if self._handle:
                self._lib.rm_free(self._handle)
                self._handle = None

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()

    def __del__(self):
        if getattr(self, "_handle", None):
            self.close()
