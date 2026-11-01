#!/usr/bin/env python3
"""Preserve Cargo freshness across CI checkouts, only for identical input bytes.

Cargo still decides every artifact's validity. This never touches build outputs:
it restores a source timestamp only after verifying its path, context and SHA256.
Changed/new sources receive a fresh timestamp so backdated edits cannot look fresh.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
STATE = 'rustmage-compile-inputs.json'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def context(root):
    return dict(root=str(root.resolve()), platform=platform.system()+':'+platform.machine(),
                rustc=subprocess.check_output(['rustc', '--version', '--verbose'], text=True),
                rustflags=os.environ.get('RUSTFLAGS', ''),
                encoded_rustflags=os.environ.get('CARGO_ENCODED_RUSTFLAGS', ''),
                config=digest(root/'.cargo/config.toml'),
                wrapper=digest(root/'scripts/rustc_guard.py'))


def inputs(root):
    tracked = subprocess.check_output(
        ['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'], cwd=root)
    for raw in tracked.split(b'\0'):
        if not raw:
            continue
        name = os.fsdecode(raw)
        path = root/name
        # include_str!/include_bytes! accept any extension. Tracking every regular
        # repository input avoids silently missing TSV or future binary assets.
        if (path.is_file() and not path.is_symlink()
                and path.resolve().is_relative_to(root.resolve())):
            yield name, path


def save(root, target, paths=None, build_context=None):
    files = {}
    for name, path in inputs(root) if paths is None else paths:
        before = path.stat()
        sha = digest(path)
        after = path.stat()
        if (before.st_mtime_ns, before.st_size) != (after.st_mtime_ns, after.st_size):
            raise RuntimeError('compile input changed during cache publication: ' + name)
        files[name] = dict(sha256=sha, mtime_ns=after.st_mtime_ns)
    data = dict(schema=1, context=context(root) if build_context is None else build_context,
                saved_ns=time.time_ns(), files=files)
    target.mkdir(parents=True, exist_ok=True)
    temporary = target/(STATE + '.tmp')
    temporary.write_text(json.dumps(data, sort_keys=True) + '\n')
    temporary.replace(target/STATE)
    return dict(saved=len(files))


def restore(root, target, paths=None, build_context=None):
    try:
        data = json.loads((target/STATE).read_text())
    except (FileNotFoundError, ValueError):
        return dict(restored=0, reason='no valid input manifest')
    current_context = context(root) if build_context is None else build_context
    if data.get('schema') != 1 or data.get('context') != current_context:
        return dict(restored=0, reason='build context changed')
    files = data.get('files', {})
    if not isinstance(files, dict) or any(
            Path(name).is_absolute() or '..' in Path(name).parts for name in files):
        return dict(restored=0, reason='invalid input paths')
    restored, changed = 0, 0
    for name, path in inputs(root) if paths is None else paths:
        old = files.get(name)
        current = path.stat()
        if old and old.get('sha256') == digest(path) and isinstance(old.get('mtime_ns'), int):
            os.utime(path, ns=(current.st_atime_ns, old['mtime_ns']))
            restored += 1
        else:
            os.utime(path, ns=(current.st_atime_ns, time.time_ns()))
            changed += 1
    return dict(restored=restored, changed=changed)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['save', 'restore'])
    parser.add_argument('--target-dir', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(globals()[args.operation](ROOT, args.target_dir.resolve())))


if __name__ == '__main__':
    main()
