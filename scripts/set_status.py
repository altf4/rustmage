#!/usr/bin/env python3
"""Freeze a full set's identities, then report admission without claiming rules acceptance."""
import argparse
import hashlib
import json
from pathlib import Path
if __package__:
    from .inventory_standard import name_key, set_entries
else:
    from inventory_standard import name_key, set_entries

ROOT = Path(__file__).resolve().parents[1]

def digest(value):
    return hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()

def freeze(root, xmage, code):
    inventory = json.loads((root / 'data/standard/xmage-inventory.json').read_text())
    oracle = json.loads((root / 'data/standard/cards.json').read_text())
    table = inventory['set_tables'][code]
    raw = (xmage / table['source']).read_bytes()
    if hashlib.sha256(raw).hexdigest() != table['sha256']:
        raise ValueError('set table differs from pinned source')
    facts = {}
    for card in oracle['cards']:
        for alias in {card['name'], card['name'].split(' // ')[0]}:
            key = name_key(alias)
            if key in facts and facts[key]['oracle_id'] != card['oracle_id']:
                raise ValueError('ambiguous set identity: ' + alias)
            facts[key] = card
    entries = set_entries(raw.decode())
    if len(entries) != table['printing_count']:
        raise ValueError('set printing count differs from inventory')
    members = {}
    unresolved = set()
    for name, number, rarity, cls in entries:
        card = facts.get(name_key(name))
        if card is None:
            unresolved.add(name)
            continue
        row = members.setdefault(card['oracle_id'], dict(name=card['name'], oracle_id=card['oracle_id'], facts_sha256=digest(card), printings=[]))
        row['printings'].append(number)
    # A missing XMage table entry must not shrink a set that has pinned Oracle facts.
    for card in oracle['cards']:
        if card['reference_printing']['set'].upper() == code:
            members.setdefault(card['oracle_id'], dict(name=card['name'], oracle_id=card['oracle_id'], facts_sha256=digest(card), printings=[]))
    return dict(schema=1, set=code, xmage_revision=inventory['xmage_source_commit'], table=table,
                scope='All physical set card identities, including reprints and starter products; cosmetic printings counted once',
                card_count=len(members)+len(unresolved), cards=sorted(members.values(), key=lambda c: c['name']), unresolved=sorted(unresolved))

def status(root, manifest):
    if manifest['card_count'] != len(manifest['cards'])+len(manifest['unresolved']):
        raise ValueError('incomplete set manifest')
    facts = {c['oracle_id']: c for c in json.loads((root / 'data/standard/cards.json').read_text())['cards']}
    coverage = {c['name']: c for c in json.loads((root / 'data/standard/coverage.json').read_text())['cards']}
    seen = set()
    admitted, gated = [], []
    for row in manifest['cards']:
        if row['oracle_id'] in seen:
            raise ValueError('duplicate set Oracle identity')
        seen.add(row['oracle_id'])
        card = facts[row['oracle_id']]
        if digest(card) != row['facts_sha256'] or card['name'] != row['name']:
            raise ValueError('set facts changed: ' + row['name'])
        entry = coverage[row['name']]
        (admitted if entry['implemented'] else gated).append(row['name'])
    return dict(set=manifest['set'], total_cards=len(seen) + len(manifest['unresolved']),
                admitted_cards=len(admitted), gated_cards=len(gated), unresolved=manifest['unresolved'],
                admitted=admitted, gated=gated,
                acceptance='pending: admission flags do not prove complete rules or full-game coverage')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('set', help='Full set code, e.g. FDN')
    mode=parser.add_mutually_exclusive_group()
    mode.add_argument('--freeze', action='store_true')
    mode.add_argument('--check', action='store_true', help='Authenticate full membership against pinned source')
    parser.add_argument('--xmage', type=Path, default=Path('/home/altf4/Code/mage'))
    args = parser.parse_args()
    code = args.set.upper()
    path = ROOT / 'data/sets' / (code.lower() + '.json')
    if args.freeze:
        manifest = freeze(ROOT, args.xmage, code)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    manifest=json.loads(path.read_text())
    if args.check and manifest != freeze(ROOT, args.xmage, code):
        raise ValueError('full set membership differs from pinned source')
    print(json.dumps(status(ROOT, manifest), ensure_ascii=False, indent=2))

if __name__ == '__main__':
    main()
