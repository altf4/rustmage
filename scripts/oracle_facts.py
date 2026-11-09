"""Load authenticated offline Oracle facts without broadening Standard legality."""
import hashlib
import json
from pathlib import Path
import re
from urllib.parse import urlparse

if __package__:
    from .fetch_standard import validate_rules_text
    from .inventory_standard import name_key
else:
    from fetch_standard import validate_rules_text
    from inventory_standard import name_key

SUPPLEMENT = Path('data/supplemental/card-oracle-facts.json')


def digest(value):
    return hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True,
                                     separators=(',', ':')).encode()).hexdigest()


def is_meld_result(card):
    return card.get('layout') == 'meld' and any(
        part['component'] == 'meld_result' and part['name'] == card['name']
        for part in card.get('all_parts', ())
    )


def load(root):
    """Return all facts and the unchanged set of legally Standard Oracle IDs.

    Supplemental rows may describe banned or otherwise nonlegal set members.
    They cannot override a Standard identity, alias or legality decision.
    """
    root = Path(root)
    standard = json.loads((root/'data/standard/cards.json').read_text(encoding='utf-8'))
    if standard['schema'] != 1 or digest(standard['cards']) != standard['cards_sha256']:
        raise ValueError('frozen Standard Oracle facts changed')
    cards = {card['oracle_id']: card for card in standard['cards']}
    if len(cards) != len(standard['cards']) or len(cards) != standard['card_count']:
        raise ValueError('duplicate or incomplete Standard Oracle identities')
    standard_ids = frozenset(cards)
    aliases = {}

    def add_aliases(card):
        validate_rules_text(card)
        for name in {card['name'], card['name'].split(' // ')[0]}:
            key = name_key(name)
            if key in aliases and aliases[key] != card['oracle_id']:
                raise ValueError('ambiguous Oracle identity: ' + name)
            aliases[key] = card['oracle_id']

    for card in cards.values():
        add_aliases(card)
    path = root/SUPPLEMENT
    if path.is_file():
        document = json.loads(path.read_text(encoding='utf-8'))
        if document['schema'] != 1 or document['oracle_cards_sha256'] != standard['cards_sha256']:
            raise ValueError('supplemental Oracle snapshot pin changed')
        target = document['target']
        target_path = Path(target['path'])
        if (target_path.is_absolute() or '..' in target_path.parts
                or target_path.parts[:2] != ('data', 'targets')):
            raise ValueError('invalid supplemental target path')
        target_bytes = (root/target_path).read_bytes()
        if hashlib.sha256(target_bytes).hexdigest() != target['sha256']:
            raise ValueError('supplemental Standard target pin changed')
        rules = json.loads(target_bytes)
        if rules['schema'] != 1 or rules['as_of'] != standard['as_of']:
            raise ValueError('supplemental target and Oracle dates disagree')
        for entry in document['cards']:
            card = entry['facts']
            if card['oracle_id'] in cards:
                raise ValueError('duplicate supplemental Oracle identity: ' + card['name'])
            if digest(card) != entry['facts_sha256']:
                raise ValueError('supplemental Oracle facts changed: ' + card['name'])
            if entry['standard_legal'] is not False or entry['standard_status'] not in ('banned', 'not_legal'):
                raise ValueError('supplemental facts cannot grant Standard legality')
            if (entry['standard_status'] == 'banned') != (card['name'] in rules['banned_card_names']):
                raise ValueError('supplemental Standard ban status changed: ' + card['name'])
            source = urlparse(entry['source'])
            if source.scheme != 'https' or source.hostname != 'api.scryfall.com':
                raise ValueError('invalid supplemental Oracle source')
            if not re.fullmatch(r'[0-9a-f]{64}', entry['source_response_sha256']):
                raise ValueError('invalid supplemental response artifact digest')
            add_aliases(card)
            cards[card['oracle_id']] = card
    return cards, standard_ids
