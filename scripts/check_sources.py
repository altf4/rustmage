#!/usr/bin/env python3
"""Check generated linkage and each unique authored XMage source pin once."""
import argparse
import hashlib
import json
from pathlib import PurePosixPath
import re
import subprocess

if __package__:
    from . import land_card_names, link_catalogue
    from .source_paths import xmage_source
else:
    import land_card_names
    import link_catalogue
    from source_paths import xmage_source

ROOT = link_catalogue.ROOT
HASH = re.compile(r'[0-9a-f]{64}')
REVISION = re.compile(r'[0-9a-f]{40}')


def source_pins(records, inventory):
    pins = {}

    def add(revision, path, digest):
        if (not REVISION.fullmatch(revision) or not HASH.fullmatch(digest)
                or PurePosixPath(path).is_absolute() or '..' in PurePosixPath(path).parts
                or any(char in path for char in '\n\r\0:')):
            raise ValueError('invalid source pin: '+path)
        key = (revision, path)
        if key in pins and pins[key] != digest:
            raise ValueError('conflicting source digests: '+path)
        pins[key] = digest

    for record in records.values():
        lift = record.get('typed_lift')
        if lift:
            if not REVISION.fullmatch(lift['revision']) or not HASH.fullmatch(lift['definition_sha256']):
                raise ValueError('invalid historical typed-lift pin: '+record['name'])
        if 'java_source' in record:
            add(record['java_revision'], record['java_source'], record['java_sha256'])
        elif not lift:
            raise ValueError('definition has no source provenance: '+record['name'])
        for dependency in record.get('xmage_dependencies', []):
            add(dependency['revision'], dependency['path'], dependency['sha256'])
    for table in inventory['set_tables'].values():
        add(inventory['xmage_source_commit'], table['source'], table['sha256'])
    return pins



def check_absences(records, inventory, raw_inventory):
    """Verify explicit missing-class evidence without inventing a card class pin."""
    by_oracle = {card['oracle_id']: card for card in inventory['cards']}
    digest = hashlib.sha256(raw_inventory).hexdigest()
    for record in records.values():
        evidence = record.get('xmage_absence')
        if evidence is None:
            continue
        entry = by_oracle.get(record.get('oracle_id'))
        if (evidence.get('revision') != inventory['xmage_source_commit']
                or evidence.get('oracle_id') != record.get('oracle_id')
                or evidence.get('inventory_path') != 'data/standard/xmage-inventory.json'
                or evidence.get('inventory_sha256') != digest
                or entry is None or entry['name'] != record['name']
                or entry.get('implementations') or entry.get('xmage_name') is not None
                or 'java_source' in record or not record.get('xmage_dependencies')
                or not record.get('rules_authority', {}).get('url', '').startswith('https://')):
            raise ValueError('invalid explicit XMage absence evidence: '+record['name'])


def check_java(pins):
    # One git process reads every immutable blob. No XMage working-tree checkout,
    # Java compilation, network request per card, or repeated helper-source hash.
    source = xmage_source()
    process = subprocess.Popen(['git', '-C', str(source), 'cat-file', '--batch'],
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    try:
        for (revision, path), expected in sorted(pins.items()):
            process.stdin.write(f'{revision}:{path}\n'.encode())
            process.stdin.flush()
            header = process.stdout.readline().decode().strip().split()
            if len(header) != 3 or header[1] != 'blob':
                raise ValueError('missing pinned XMage blob: '+revision+':'+path)
            raw = process.stdout.read(int(header[2]))
            if process.stdout.read(1) != b'\n' or hashlib.sha256(raw).hexdigest() != expected:
                raise ValueError('XMage source hash changed: '+path)
    finally:
        process.stdin.close()
        process.stdout.close()
        code = process.wait()
    if code:
        raise RuntimeError(f'git cat-file exited {code}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--metadata-only', action='store_true',
                        help='check local metadata only; ordinary validation also verifies XMage blobs')
    args = parser.parse_args()
    land_card_names.check(ROOT)
    metadata, records, version = link_catalogue.load()
    if [card['index'] for card in metadata['cards']] != list(range(len(records))):
        raise ValueError('catalogue IDs must be contiguous and stable')
    for card in metadata['cards']:
        if records[card['index']]['catalogue_id'] != card['index']:
            raise ValueError('provenance catalogue ID changed: '+card['name'])
    outputs = link_catalogue.render(metadata, records, version)
    stale = [str(path.relative_to(ROOT)) for path, value in outputs.items()
             if not path.is_file() or path.read_text() != value]
    if stale:
        raise ValueError('stale generated linkage; run scripts/link_catalogue.py: '+', '.join(stale))
    inventory = json.loads((ROOT/'data/standard/xmage-inventory.json').read_text())
    oracle = json.loads((ROOT/'data/standard/cards.json').read_text())
    if inventory['oracle_cards_sha256'] != oracle['cards_sha256']:
        raise ValueError('XMage inventory and Oracle source pins disagree')
    check_absences(records, inventory, (ROOT/'data/standard/xmage-inventory.json').read_bytes())
    pins = source_pins(records, inventory)
    if not args.metadata_only:
        check_java(pins)
    print(json.dumps(dict(definitions=len(records), source_blobs=len(pins),
                          java_verified=not args.metadata_only, catalogue_version=version)))


if __name__ == '__main__':
    main()
