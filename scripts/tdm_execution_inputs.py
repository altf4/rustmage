#!/usr/bin/env python3
"""Freeze TDM library/provenance/build inputs for fail-closed game receipts.

The game helper separately hashes its own source. Its compiled manifest verifies
these current files before using a prior verification corpus, so engine/policy
changes cannot retain the identity of a catalogue-only checkpoint.
"""
import argparse
import hashlib
import json
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


def payload(root):
    root = Path(root)
    paths = set()
    for directory in ('src', 'crates'):
        for pattern in ('*.rs', 'Cargo.toml'):
            paths.update(p for p in (root/directory).rglob(pattern) if p.is_file())
    paths.update(p for p in (root/'src/card_definitions').rglob('*.json') if p.is_file())
    for name in REQUIRED:
        p = root/name
        if not p.is_file() or p.is_symlink():
            raise ValueError('missing or nonregular execution input: ' + name)
        paths.add(p)
    paths.update(root/name for name in OPTIONAL if (root/name).is_file())
    files = {}
    for p in sorted(paths):
        if p.is_symlink():
            raise ValueError('execution input must not be a symlink: ' + str(p.relative_to(root)))
        files[p.relative_to(root).as_posix()] = hashlib.sha256(p.read_bytes()).hexdigest()
    return dict(schema=1, scope=SCOPE, source_count=len(files), files=files)


def encoded(root):
    return (json.dumps(payload(root), ensure_ascii=False, sort_keys=True, indent=2)+'\n').encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--write', action='store_true')
    mode.add_argument('--check', action='store_true')
    args = parser.parse_args()
    try:
        expected = encoded(ROOT)
        path = ROOT/MANIFEST
        if args.write:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(expected)
        elif not path.is_file() or path.read_bytes() != expected:
            raise ValueError('stale TDM execution inputs; regenerate with --write before compiling games')
        print(json.dumps(dict(source_count=json.loads(expected)['source_count'],
                              execution_inputs_sha256=hashlib.sha256(expected).hexdigest())))
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
