#!/usr/bin/env python3
"""Generate exact outside-game names from pinned structured Oracle facts."""
import argparse
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]

def render():
    cards = json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']
    rows = []
    for card in cards:
        layout = card['layout']
        faces = card.get('card_faces', [])
        if layout == 'split':
            if len(faces) != 2:
                raise ValueError('split card must have exactly two faces: ' + card['name'])
            names = [face['name'] for face in faces]
        elif layout in ('transform', 'modal_dfc', 'prepare', 'adventure'):
            if not faces:
                raise ValueError('missing front face: ' + card['name'])
            names = [faces[0]['name']]
        elif layout in ('normal', 'saga', 'class', 'case', 'meld'):
            names = [card['name']]
        else:
            raise ValueError('unknown layout: ' + layout)
        fields = [card['oracle_id'], *names]
        if any('\t' in field or '\n' in field for field in fields):
            raise ValueError('name cannot be encoded in TSV')
        rows.append('\t'.join(fields))
    if len(rows) != len({row.split('\t', 1)[0] for row in rows}):
        raise ValueError('duplicate Oracle identity')
    return '\n'.join(sorted(rows)) + '\n'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    output = render()
    target = ROOT / 'data/outside-card-names.tsv'
    if args.check:
        if not target.is_file() or target.read_text() != output:
            raise SystemExit('stale outside-game card name table')
    else:
        target.write_text(output)

if __name__ == '__main__':
    main()
