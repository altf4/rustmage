#!/usr/bin/env python3
"""Reconcile the pinned Standard identities with XMage's actual source tables.

Dependency imports are migration hints, not a translation or proof of semantics.
Missing XMage cards remain explicit. Comments never count as implementations.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import unicodedata

TOKEN = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|//[^\n]*|/\*[\s\S]*?\*/')
SET = re.compile(r'super\(\s*"((?:\\.|[^"\\])*)"\s*,\s*"([A-Z0-9]+)"')
ENTRY = re.compile(r'new\s+SetCardInfo\s*\(\s*"((?:\\.|[^"\\])*)"\s*,\s*("(?:\\.|[^"\\])*"|\d+)\s*,\s*Rarity\.(\w+)\s*,\s*([\w.]+)\.class')


def uncomment(source):
    return TOKEN.sub(lambda m: re.sub(r'[^\n]', ' ', m[0]) if m[0].startswith(('//', '/*')) else m[0], source)


def java_string(value):
    # Java permits escaped apostrophes in strings, unlike JSON.
    return json.loads('"' + value.replace("\\'", "'") + '"')


def name_key(name):
    # XMage uses ASCII transliterations for some printed English names. Keep
    # the authoritative spelling in output; this key only reconciles sources.
    text = ''.join(c for c in unicodedata.normalize('NFKD', name) if not unicodedata.combining(c))
    return re.sub(r'\s*\.\s*', '.', text.replace('…', '...').replace('’', "'")).casefold()


def set_entries(source):
    clean = uncomment(source)
    matches = list(ENTRY.finditer(clean))
    if len(matches) != len(re.findall(r'new\s+SetCardInfo\s*\(', clean)):
        raise ValueError("unparsed SetCardInfo declaration; do not silently drop printings")
    return [(java_string(m[1]), java_string(m[2][1:-1]) if m[2].startswith('"') else m[2], m[3], m[4])
            for m in matches]


def inventory(root, target, oracle):
    expected = target['xmage_source_commit']
    actual = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
    if actual != expected:
        raise ValueError(f"XMage revision {actual} differs from pin {expected}")
    if subprocess.check_output(['git', '-C', str(root), 'status', '--porcelain', '--', 'Mage.Sets/src', 'Mage/src/main/java'], text=True).strip():
        raise ValueError("XMage source is dirty; pin reviewed source before inventorying")
    tables = {}
    by_name = {}
    for path in sorted((root / 'Mage.Sets/src/mage/sets').glob('*.java')):
        text = path.read_text()
        match = SET.search(uncomment(text))
        if match is None or match[2] not in target['set_codes']:
            continue
        code = match[2]
        if code in tables:
            raise ValueError(f"duplicate set code {code}")
        entries = set_entries(text)
        tables[code] = {'source': str(path.relative_to(root)), 'sha256': hashlib.sha256(text.encode()).hexdigest(), 'printing_count': len(entries)}
        for name, number, rarity, implementation in entries:
            record = by_name.setdefault(name, {'classes': set(), 'printings': []})
            record['classes'].add(implementation)
            record['printings'].append({'set': code, 'number': number, 'rarity': rarity})
    if set(tables) != set(target['set_codes']):
        raise ValueError(f"missing set tables: {set(target['set_codes']) - set(tables)}")
    normalized = {}
    for name in by_name:
        key = name_key(name)
        if key in normalized and normalized[key] != name:
            raise ValueError(f"ambiguous normalized source name: {name}")
        normalized[key] = name
    cards = []
    dependencies = Counter()
    consumed = set()
    for card in oracle['cards']:
        # XMage's set table generally names the front face of a multiface card.
        names = [card['name'], card['name'].split(' // ')[0]]
        matched_name = next((normalized[name_key(name)] for name in names if name_key(name) in normalized), None)
        record = {'oracle_id': card['oracle_id'], 'name': card['name'], 'xmage_name': matched_name,
                  'implementations': [], 'printings': []}
        if matched_name is not None:
            consumed.add(matched_name)
            row = by_name[matched_name]
            record['printings'] = sorted(row['printings'], key=lambda p: (p['set'], p['number']))
            for cls in sorted(row['classes']):
                candidates = [root / directory / (cls.replace('.', '/') + '.java')
                              for directory in ('Mage.Sets/src', 'Mage/src/main/java')]
                path = next((p for p in candidates if p.exists()), None)
                if path is None:
                    record['implementations'].append({'class': cls, 'missing': True})
                    continue
                text = path.read_text()
                imports = sorted(set(re.findall(r'\bimport\s+(mage\.(?:abilities|target|watchers|counters)\.[\w.*]+)\s*;', uncomment(text))))
                dependencies.update(imports)
                record['implementations'].append({'class': cls, 'source': str(path.relative_to(root)),
                    'sha256': hashlib.sha256(text.encode()).hexdigest(), 'dependencies': imports})
        cards.append(record)
    missing = [card['name'] for card in cards if not card['implementations'] or any(item.get('missing') for item in card['implementations'])]
    extras = sorted(set(by_name) - consumed)
    banned = set(target['banned_card_names'])
    return {
        'schema': 1, 'as_of': oracle['as_of'], 'xmage_source_commit': actual,
        'oracle_cards_sha256': oracle['cards_sha256'], 'set_tables': tables,
        'summary': {'standard_legal_identities': len(cards), 'xmage_table_identities': len(by_name),
                    'missing_xmage_implementations': missing,
                    'xmage_only_banned_identities': sorted(set(extras) & banned),
                    'xmage_only_unresolved_identities': sorted(set(extras) - banned)},
        'dependency_frequency': dict(sorted(dependencies.items(), key=lambda item: (-item[1], item[0]))),
        'cards': cards,
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--xmage', type=Path, required=True)
    parser.add_argument('--target', type=Path, default=Path('data/targets/standard-2026-10-15.json'))
    parser.add_argument('--cards', type=Path, default=Path('data/standard/cards.json'))
    parser.add_argument('--output', type=Path, default=Path('data/standard/xmage-inventory.json'))
    args = parser.parse_args()
    report = inventory(args.xmage, json.loads(args.target.read_text()), json.loads(args.cards.read_text()))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report['summary'], indent=2))
