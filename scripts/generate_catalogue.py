#!/usr/bin/env python3
"""Compile pinned card facts into immutable native definitions.

Only explicit implementations and exact vanilla/keyword-only templates are
executable. All other identities remain known but unsupported, never guessed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
LEGACY = [
    ('Plains', 'Land', '{W}', 0), ('Island', 'Land', '{U}', 1),
    ('Swamp', 'Land', '{B}', 2), ('Mountain', 'Land', '{R}', 3), ('Forest', 'Land', '{G}', 4),
    ('Grizzly Bears', 'Creature(2, 2)', '{1}{G}', None),
    ('Goblin Piker', 'Creature(2, 1)', '{1}{R}', None),
    ('Hill Giant', 'Creature(3, 3)', '{3}{R}', None),
    ('Llanowar Elves', 'Creature(1, 1)', '{G}', 4),
    ('Lightning Bolt', 'Damage(3)', '{R}', None), ('Shock', 'Damage(2)', '{R}', None),
    ('Giant Growth', 'Grow', '{G}', None), ('Divination', 'Draw', '{2}{U}', None),
    ('Counterspell', 'Counter', '{U}{U}', None),
]
KEYWORDS = {'flying': 1, 'reach': 2, 'haste': 4, 'vigilance': 8, 'defender': 16, 'flash': 32,
            'lifelink': 64, 'deathtouch': 128, 'trample': 256, 'first strike': 512, 'double strike': 1024}
UNLIMITED = {'Cid, Timeless Artificer', 'Hare Apparent', 'Slime Against Humanity', "Sphinx's Approach", 'Tempest Hawk'}


def fixed_cost(cost):
    if not re.fullmatch(r'(?:\{(?:[0-9]+|[WUBRG])\})+', cost):
        return None
    colored = [0] * 5
    generic = 0
    for symbol in re.findall(r'\{([^}]+)\}', cost):
        if symbol.isdigit():
            generic += int(symbol)
        else:
            colored['WUBRG'.index(symbol)] += 1
    return colored, generic


def creature_template(card):
    # Missing/unknown rules text is not a blank text box. A fresh checksum only
    # establishes data integrity; it cannot establish omitted card semantics.
    if not isinstance(card.get('oracle_text'), str):
        return None
    if card.get('layout') != 'normal' or card.get('type_line', '').split(' — ')[0] not in ('Creature', 'Artifact Creature'):
        return None
    if not card.get('power', '').isdigit() or not card.get('toughness', '').isdigit():
        return None
    if fixed_cost(card.get('mana_cost', '')) is None:
        return None
    # Remove only balanced, unnested reminder text; fail closed on residual text.
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    words = {word.strip().casefold() for word in text.replace('\n', ',').split(',') if word.strip()}
    if not words <= KEYWORDS.keys():
        return None
    return f"Creature({int(card['power'])}, {int(card['toughness'])})", sum(KEYWORDS[word] for word in words)


def compile_catalogue(snapshot, target):
    canonical = json.dumps(snapshot['cards'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    if hashlib.sha256(canonical).hexdigest() != snapshot['cards_sha256'] or len(snapshot['cards']) != snapshot['card_count']:
        raise ValueError('card snapshot checksum/count mismatch')
    cards = {card['name']: card for card in snapshot['cards']}
    if len(cards) != len(snapshot['cards']):
        raise ValueError('duplicate card name')
    if set(cards) & set(target['banned_card_names']):
        raise ValueError('legal snapshot and banned-list target disagree')
    unreviewed_exceptions = {name for name, card in cards.items() if 'A deck can have' in card.get('oracle_text', '')} - UNLIMITED
    if unreviewed_exceptions:
        raise ValueError(f'unreviewed deck construction exception: {sorted(unreviewed_exceptions)}')
    legacy = {name: (effect, cost, color) for name, effect, cost, color in LEGACY}
    names = list(legacy) + sorted((set(cards) | set(target['banned_card_names'])) - set(legacy))
    if len(names) > 65536:
        raise ValueError('catalogue exceeds u16 identity space')
    definitions = []
    supported = []
    coverage = []
    for index, name in enumerate(names):
        card = cards.get(name, {})
        cost = card.get('mana_cost', '')
        keywords = sum(KEYWORDS[word] for word in {keyword.casefold() for keyword in card.get('keywords', [])} & KEYWORDS.keys())
        effect = 'Unsupported'
        color = None
        template = creature_template(card)
        if name in legacy:
            effect, fallback_cost, color = legacy[name]
            # Lands have no mana cost. Their legacy cost token encodes colour only.
            cost = '' if effect == 'Land' else card.get('mana_cost', fallback_cost)
            kind = 2 if effect == 'Land' else 1 if effect.startswith('Creature') else 4 if effect in ('Draw',) else 8
        else:
            types = card.get('type_line', '').split(' — ')[0].split()
            kind = (1 if 'Creature' in types else 0) | (2 if 'Land' in types else 0) | (4 if 'Sorcery' in types else 0) | (8 if 'Instant' in types else 0)
            if template is not None:
                effect, keywords = template
        if effect != 'Unsupported':
            supported.append(index)
        basic = name in {'Plains', 'Island', 'Swamp', 'Mountain', 'Forest'}
        colored, generic = fixed_cost(cost) or ([0] * 5, 0)
        deck_limit = 'None' if basic or name in UNLIMITED else 'Some(4)'
        quote = lambda value: json.dumps(value, ensure_ascii=False)
        oracle_id = f"Some({quote(card['oracle_id'])})" if card else 'None'
        type_line = card.get('type_line', 'Basic Land' if basic else 'Creature' if kind & 1 else 'Instant' if kind & 8 else 'Sorcery' if kind & 4 else '')
        oracle_keywords = '&[' + ', '.join(quote(word) for word in card.get('keywords', [])) + ']'
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {quote(name)}, oracle_id: {oracle_id}, mana_cost: {quote(cost)}, type_line: {quote(type_line)}, oracle_keywords: {oracle_keywords}, standard_legal: {str(name in cards).lower()}, implemented: {str(effect != 'Unsupported').lower()}, keywords: Keywords({keywords}), deck_limit: {deck_limit} }}, effect: Effect::{effect}, kind: {kind}, basic: {str(basic).lower()}, mana_color: {'None' if color is None else f'Some({color})'}, colored: {colored}, generic: {generic} }},")
        coverage.append({'name': name, 'standard_legal': name in cards, 'implemented': effect != 'Unsupported',
                         'implementation': 'explicit' if name in legacy else 'creature_template' if template else None})
    # The fingerprint covers data and compiler logic; card IDs are local to it.
    target_bytes = json.dumps(target, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    version = hashlib.sha256(canonical + target_bytes + Path(__file__).read_bytes()).hexdigest()
    lines = ['// Generated by scripts/generate_catalogue.py. Do not edit.',
             f'pub const CATALOGUE_VERSION: &str = "{version}";',
             '#[rustfmt::skip]', f'pub(super) static DEFINITIONS: [Definition; {len(names)}] = [', *definitions, '];',
             '#[rustfmt::skip]', f'pub(super) static NAME_INDEX: [(&str, u16); {len(names)}] = [']
    lines += [f'    ({json.dumps(name, ensure_ascii=False)}, {index}),' for name, index in sorted((name, index) for index, name in enumerate(names))]
    lines += ['];', '#[allow(non_upper_case_globals)]', 'impl Card {', '#[rustfmt::skip]',
              f'    pub const ALL: [Card; {len(supported)}] = [{", ".join(f"Card({i})" for i in supported)}];']
    # Preserve existing Rust constants and JSON tokens. New callers use English names.
    for index, (name, _, _, _) in enumerate(LEGACY):
        lines.append(f'    pub const {name.replace(" ", "")}: Card = Card({index});')
    lines += ['}', '']
    return '\n'.join(lines), {'schema': 1, 'catalogue_version': version, 'standard_legal_cards': len(cards),
        'implemented_standard_cards': sum(row['implemented'] and row['standard_legal'] for row in coverage),
        'implemented_total': len(supported), 'cards': coverage}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    snapshot = json.loads((ROOT / 'data/standard/cards.json').read_text())
    target = json.loads((ROOT / 'data/targets/standard-2026-10-15.json').read_text())
    rust, coverage = compile_catalogue(snapshot, target)
    outputs = {ROOT / 'src/generated/catalogue.rs': rust,
               ROOT / 'data/standard/coverage.json': json.dumps(coverage, ensure_ascii=False, indent=2) + '\n'}
    for path, content in outputs.items():
        if args.check:
            if not path.exists() or path.read_text() != content:
                raise SystemExit(f'outdated generated file: {path.relative_to(ROOT)}')
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
    print(json.dumps({key: value for key, value in coverage.items() if key != 'cards'}))
