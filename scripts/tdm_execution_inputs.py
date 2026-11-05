#!/usr/bin/env python3
"""Freeze TDM library/provenance/build inputs for fail-closed game receipts.

The game helper separately hashes its own source. Its compiled manifest verifies
these current files before using a prior verification corpus, so engine/policy
changes cannot retain the identity of a catalogue-only checkpoint.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = Path('examples/support/tdm_execution_inputs.json')
REQUIRED = (
    'Cargo.toml', 'Cargo.lock', 'scripts/tdm_execution_inputs.py',
    'examples/tdm_games.rs', 'tests/tdm_complete_games.rs',
    'examples/support/tdm_game_decks.json', 'data/catalogue-identities.json',
    'data/standard/cards.json', 'data/supplemental/card-oracle-facts.json',
    'data/sets/tdm.json', 'data/observation-schema.txt',
    'data/card-names.txt', 'data/land-card-names.txt',
    'data/interchangeable-card-names.tsv', 'data/outside-card-names.tsv',
)
OPTIONAL = ('build.rs', '.cargo/config.toml', '.cargo/config',
            'rust-toolchain', 'rust-toolchain.toml')
SCOPE = ('Workspace library Rust, provenance and build inputs plus TDM acceptance '
         'entrypoints; shared harness is independently self-hashed.')


def regular_input(root, path):
    relative = path.relative_to(root)
    current = root
    for part in relative.parts:
        current = current/part
        if current.is_symlink():
            raise ValueError('missing or nonregular execution input: ' + str(relative))
    if not path.is_file():
        raise ValueError('missing or nonregular execution input: ' + str(relative))
    return path


def walk_error(error):
    raise error


def payload(root):
    root = Path(root)
    paths = set()
    for directory in ('src', 'crates'):
        base = root/directory
        if not base.is_dir() or base.is_symlink():
            raise ValueError('missing or nonregular execution source root: ' + directory)
        for parent, dirs, names in os.walk(base, followlinks=False, onerror=walk_error):
            parent = Path(parent)
            for name in dirs:
                p = parent/name
                if p.is_symlink():
                    raise ValueError('nonregular execution source directory: ' + str(p.relative_to(root)))
            for name in names:
                p = parent/name
                provenance = p.suffix == '.json' and (root/'src/card_definitions') in p.parents
                if p.suffix == '.rs' or name == 'Cargo.toml' or provenance:
                    paths.add(regular_input(root, p))
    for name in REQUIRED:
        paths.add(regular_input(root, root/name))
    for name in OPTIONAL:
        p = root/name
        if p.exists() or p.is_symlink():
            paths.add(regular_input(root, p))
    files = {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in sorted(paths)}
    return dict(schema=1, scope=SCOPE, source_count=len(files), files=files)


def encoded(root):
    return (json.dumps(payload(root), ensure_ascii=False, sort_keys=True, indent=2)+'\n').encode()


def check(root):
    root = Path(root)
    expected = encoded(root)
    path = regular_input(root, root/MANIFEST)
    if path.read_bytes() != expected:
        raise ValueError('stale TDM execution inputs; regenerate with --write before compiling games')
    return expected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--write', action='store_true')
    mode.add_argument('--check', action='store_true')
    args = parser.parse_args()
    try:
        if args.write:
            expected = encoded(ROOT)
            path = ROOT/MANIFEST
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(expected)
        else:
            expected = check(ROOT)
        print(json.dumps(dict(source_count=json.loads(expected)['source_count'],
                              execution_inputs_sha256=hashlib.sha256(expected).hexdigest())))
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
