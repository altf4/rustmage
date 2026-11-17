#!/usr/bin/env python3
"""Resolve historical menu indices using their pinned library, without changing captures.

The output uses concrete actions and records both input hashes. It must be replayed
with identity/state assertions when the action contract changes.
"""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('trace', type=Path)
    parser.add_argument('library', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--library-sha256', required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('output already exists; preserved captures are never overwritten')
    if digest(args.library) != args.library_sha256:
        parser.error('baseline library hash does not match')
    lib = ctypes.CDLL(str(args.library.resolve()))
    for name, parameters, result in [
        ('rm_abi_version', [], ctypes.c_uint32),
        ('rm_new', [ctypes.c_char_p, ctypes.POINTER(ctypes.c_void_p)], ctypes.c_void_p),
        ('rm_request', [ctypes.c_void_p, ctypes.c_char_p], ctypes.c_void_p),
        ('rm_free', [ctypes.c_void_p], None),
        ('rm_string_free', [ctypes.c_void_p], None),
    ]:
        function = getattr(lib, name)
        function.argtypes, function.restype = parameters, result
    if lib.rm_abi_version() != 1:
        raise RuntimeError('unsupported baseline ABI')

    def take(pointer):
        if not pointer:
            raise RuntimeError('null native response')
        try:
            return ctypes.string_at(pointer).decode()
        finally:
            lib.rm_string_free(pointer)

    rows = [json.loads(line) for line in args.trace.read_text().splitlines()]
    creation = rows[0]
    config = dict(creation['arguments'])
    config.pop('library', None)
    if config.pop('outside', None) is not None:
        raise ValueError('this translator requires an ordinary game without outside pools')
    error = ctypes.c_void_p()
    handle = lib.rm_new(json.dumps(config).encode(), ctypes.byref(error))
    if not handle:
        raise RuntimeError(take(error.value) if error.value else 'creation failed')

    def request(value):
        response = json.loads(take(lib.rm_request(handle, json.dumps(value).encode())))
        if response.get('error'):
            raise RuntimeError(response['error'])
        return response

    output = [dict(creation, semantic_translation={
        'trace_sha256': digest(args.trace),
        'library_sha256': args.library_sha256,
        'contract': 'concrete_actions_from_baseline_indices_v1',
    })]
    try:
        for row in rows[1:]:
            original = row['request']
            concrete = dict(original)
            if original['op'] == 'step':
                observed = request({'op': 'observe', 'viewer': original['actor']})['observation']
                if observed['revision'] != original['revision'] or observed['actor'] != original['actor']:
                    raise RuntimeError(f"stale capture at request {row['index']}")
                concrete.pop('index')
                concrete['op'] = 'act'
                concrete['action'] = observed['actions'][original['index']]
            result = request(original)
            output.append(dict(row, request=concrete))
        final = result['observation']
        output[-1]['expected_boundary'] = {
            key: final[key] for key in ['turn', 'step', 'active', 'stack', 'objects', 'players']
        }
    finally:
        lib.rm_free(handle)
    args.output.write_text(''.join(json.dumps(row, separators=(',', ':')) + '\n' for row in output))
    print(json.dumps({'output': str(args.output), 'sha256': digest(args.output), 'requests': len(output) - 1}))


if __name__ == '__main__':
    main()
