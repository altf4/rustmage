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
# The preceding Standard inventory remains pinned by its verified digest. Its
# full bytes are in repository history at e287b66bab60272ac7c043510fc0051a87a6b68a;
# CI's shallow checkout need not fetch that historical tree for every run.
HISTORICAL_INVENTORIES = {
    '492369c7844b8d76479f0393ff9eb6b0b0cf30d7':
        '06a0e948e65878232f223738461a85c11e417d6a279c11ee1c38d8f971d37cc7',
}
# These 25 Oracle IDs were checked against the exact preceding inventory blob
# above: every row had a null XMage name and an empty implementation list.
HISTORICAL_ABSENT_ORACLE_IDS = frozenset({
    '008a11c1-d283-49fe-abd7-ff4fe8b1fe79',
    '020d561b-2e5d-439e-9e6a-318d23e73542',
    '0bbceb51-1fb8-44d7-8afc-b4746d292262',
    '11932191-4b19-49b1-bfe4-abb7b83b2e59',
    '19ef0329-08de-4835-8079-2e791e19f3a6',
    '1f47c869-f4a9-43b0-aec0-8da4e3a3fec5',
    '28ffddca-a3c4-467d-a2dc-2c5e62393e11',
    '3cd3cdc9-2ed2-4cdf-b58a-12c3a2ef218b',
    '3ebd64d2-c178-45a2-a51f-9869754baa0f',
    '3f4d6f91-95ad-4687-8899-5a21a0abb49e',
    '43037ef5-09e8-4881-beff-ed38cf7574c6',
    '4c67359d-5590-4eb0-827e-b411967ee238',
    '4d891515-39da-492e-ac19-1aa524245449',
    '4f69d2f9-c16a-4752-9026-794a29d73af8',
    '514b451b-814d-45fb-a2ba-8fe6f0bdad60',
    '61a41cf1-60cc-45ba-aa98-493c14e87d9d',
    '743fa824-18cb-4225-ba7b-5b0bbc3c8d41',
    '89bd056d-8f5b-4d0e-b80a-58a984e21100',
    '9efb3aa4-0023-4e0e-8bd4-f4dff1b3be66',
    'a8b018a7-0350-4ee0-9582-8d391018bdee',
    'b8d563e4-e2bc-4e8b-8841-6655beff9138',
    'c3936db3-d3b6-4a92-9773-6f3c52dc419d',
    'dcd97730-b793-4c0c-b158-625ac3656b4a',
    'e0142e7d-7b0b-4290-a56c-5fa950fdccf8',
    'e4402d21-bb7f-4777-bc8d-b24bf2faccb0',
})
OFFICIAL_TRK_TOKEN_FALLBACKS = {
    '@token/trk-neuroflux-heirloom': 6015,
    '@token/trk-human-no-phase-in': 6017,
}


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
        elif not lift and not record.get('xmage_missing'):
            raise ValueError('definition has no source provenance: '+record['name'])
        for dependency in record.get('xmage_dependencies', []):
            add(dependency['revision'], dependency['path'], dependency['sha256'])
    for table in inventory['set_tables'].values():
        add(inventory['xmage_source_commit'], table['source'], table['sha256'])
    return pins



def check_absences(records, inventory, raw_inventory):
    """Verify missing-class evidence at the XMage revision it actually pins.

    An inventory refresh can add a class for a card that had none at an older
    revision. That does not invalidate the earlier absence or its pinned set
    table; only evidence claiming the current revision can use the current
    inventory as proof of absence.
    """
    by_oracle = {card['oracle_id']: card for card in inventory['cards']}
    digest = hashlib.sha256(raw_inventory).hexdigest()
    for record in records.values():
        evidence = record.get('xmage_absence')
        if evidence is None:
            continue
        entry = by_oracle.get(record.get('oracle_id'))
        revision = evidence.get('revision', '')
        historical = revision != inventory['xmage_source_commit']
        if (not REVISION.fullmatch(revision)
                or evidence.get('oracle_id') != record.get('oracle_id')
                or evidence.get('inventory_path') != 'data/standard/xmage-inventory.json'
                or not HASH.fullmatch(evidence.get('inventory_sha256', ''))
                or entry is None or entry['name'] != record['name']
                or (historical and evidence['inventory_sha256'] !=
                    HISTORICAL_INVENTORIES.get(revision))
                or (historical and record.get('oracle_id') not in
                    HISTORICAL_ABSENT_ORACLE_IDS)
                or (not historical and (evidence['inventory_sha256'] != digest
                    or entry.get('implementations')
                    or entry.get('xmage_name') is not None))
                or 'java_source' in record or not record.get('xmage_dependencies')
                or (historical and not any(
                    dependency.get('revision') == revision
                    for dependency in record['xmage_dependencies']))
                or not record.get('rules_authority', {}).get('url', '').startswith('https://')):
            raise ValueError('invalid explicit XMage absence evidence: '+record['name'])


def check_official_fallbacks(records):
    """Check the user-authorized TRK Oracle/release-note source fallback."""
    audit_path = 'data/audits/trk.json'
    audit = json.loads((ROOT/audit_path).read_text())
    by_oracle = {card['oracle_id']: card for card in audit['cards']}
    rules_url = 'https://magic.wizards.com/en/news/feature/star-trek-release-notes'
    card_count = 0
    token_names = set()
    for record in records.values():
        if not record.get('xmage_missing'):
            continue
        if (record.get('official_rules_source') != rules_url
                or 'java_source' in record):
            raise ValueError('invalid official rules fallback: '+record['name'])
        oracle_id = record.get('oracle_id')
        if oracle_id is None:
            if (record['name'] not in OFFICIAL_TRK_TOKEN_FALLBACKS
                    or record.get('catalogue_id') !=
                    OFFICIAL_TRK_TOKEN_FALLBACKS[record['name']]
                    or record.get('oracle_source') != audit_path
                    or record.get('facts_sha256') is not None):
                raise ValueError('invalid TRK token fallback: '+record['name'])
            token_names.add(record['name'])
            continue
        source = by_oracle.get(oracle_id)
        if (record.get('audit_source') != audit_path
                or record.get('java_revision') != audit['xmage_revision']
                or source is None or source['name'] != record['name']
                or source['facts_sha256'] != record.get('facts_sha256')
                or source['xmage'] is not None):
            raise ValueError('invalid TRK card fallback: '+record['name'])
        card_count += 1
    if card_count != audit['missing_xmage_identities']:
        raise ValueError('TRK official fallback does not cover every missing XMage identity')
    if token_names != OFFICIAL_TRK_TOKEN_FALLBACKS.keys():
        raise ValueError('TRK official fallback token inventory changed')


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
    check_official_fallbacks(records)
    pins = source_pins(records, inventory)
    if not args.metadata_only:
        check_java(pins)
    print(json.dumps(dict(definitions=len(records), source_blobs=len(pins),
                          java_verified=not args.metadata_only, catalogue_version=version)))


if __name__ == '__main__':
    main()
