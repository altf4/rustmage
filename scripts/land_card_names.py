#!/usr/bin/env python3
"""Derive and verify the pinned Oracle land-name pool without network access.

This handles immutable names/type metadata only; it does not compile behavior.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def derive(root=ROOT):
    facts = json.loads((root/'data/land-card-name-facts.json').read_text())
    if facts['schema'] != 1:
        raise ValueError('unknown land-name source snapshot version')
    selectable = set((root/'data/card-names.txt').read_text().splitlines())
    names = {face['name'] for card in facts['cards'] for face in card['faces']
             if 'Land' in face['type_line'].split(' — ')[0].split()}
    # CR201.4d uses the chosen face's characteristics. CR201.4g identifies
    # interchangeable names, while the existing pool excludes cosmetic titles.
    pairs = [line.split('\t') for line in
             (root/'data/interchangeable-card-names.tsv').read_text().splitlines()]
    changed = True
    while changed:
        before = len(names)
        for pair in pairs:
            if any(name in names for name in pair):
                names.update(pair)
        changed = len(names) != before
    return sorted(names & selectable)


def check(root=ROOT):
    record = json.loads((root/'data/land-card-names-source.json').read_text())
    facts = root/'data/land-card-name-facts.json'
    packed = root/'data/land-card-names.txt'
    if record['schema'] != 1 or record['facts_sha256'] != sha(facts):
        raise ValueError('land-name facts snapshot changed without provenance update')
    if record['selectable_names_sha256'] != sha(root/'data/card-names.txt'):
        raise ValueError('selectable card names changed; refresh the land-name derivation')
    if record['interchangeable_names_sha256'] != sha(root/'data/interchangeable-card-names.tsv'):
        raise ValueError('interchangeable names changed; refresh the land-name derivation')
    expected = derive(root)
    if packed.read_text() != ''.join(name+'\n' for name in expected):
        raise ValueError('land card names differ from the pinned Oracle face metadata')
    if record['name_count'] != len(expected) or record['packed_sha256'] != sha(packed):
        raise ValueError('land-name pool checksum or count changed')
    return len(expected)


if __name__ == '__main__':
    print(json.dumps({'land_names': check()}))
