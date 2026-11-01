#!/usr/bin/env python3
"""Account for printed card features using explicit, executed outcome proofs.

This checks authored proof membership and frozen source text. It does not infer
coverage from card names, metadata, Rust instructions, or a successful cast.
"""
import argparse
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]


def load_admitted(root):
    catalogue = json.loads((root/'data/catalogue-identities.json').read_text())
    physical = {card['name']: card for card in catalogue['cards'][:catalogue['physical_count']]}
    coverage = json.loads((root/'data/standard/coverage.json').read_text())
    facts = {card['oracle_id']: card for card in
             json.loads((root/'data/standard/cards.json').read_text())['cards']}
    admitted = {}
    for row in coverage['cards']:
        if not row['implemented']:
            continue
        card = physical[row['name']]
        provenance = json.loads((root/row['definition']).with_suffix('.json').read_text())
        admitted[card['index']] = dict(card=card, provenance=provenance,
                                      facts=facts.get(card['oracle_id']))
    return admitted


def source_paragraphs(facts):
    return {(face_index, paragraph): text for face_index, face in
            enumerate(facts.get('card_faces', [facts])) for paragraph, text in
            enumerate(face.get('oracle_text', '').splitlines()) if text.strip()}


def check(root, native):
    errors = []
    admitted = load_admitted(root)
    directory = root/'src/testing/coverage'
    shared = json.loads((directory/'shared.json').read_text())
    passing = {test['test'] for test in native['tests'] if test.get('status') == 'ok'}
    rows = {(row['case'], row['row']) for row in native.get('features', [])}
    proved, total, covered_cards = 0, 0, 0
    families, family_models = {}, {}
    for filename, model in [('oracle_families.json', 'spell'), ('creature_families.json', 'creature'),
                            ('land_families.json', 'land')]:
        family_path = root/'src/testing'/filename
        if family_path.exists():
            for family in json.loads(family_path.read_text()):
                if family['id'] in families:
                    raise ValueError('duplicate semantic family: '+family['id'])
                families[family['id']] = family
                family_models[family['id']] = model
    foundation_path = root/'src/testing/foundations.json'
    foundations = {card['catalogue_id']: card for card in
                   json.loads(foundation_path.read_text())} if foundation_path.exists() else {}
    proof_types = dict(direct=0, shared=0, composed=0)

    def proof(value, context, trail=()):
        if not isinstance(value, dict):
            errors.append(dict(context=context, reason='uncovered'))
            return False
        if set(value) == {'all'}:
            children = value['all']
            if not isinstance(children, list) or not children:
                errors.append(dict(context=context, reason='composed proof needs its witnesses'))
                return False
            return all(proof(child, context, trail) for child in children)
        if set(value) == {'shared'}:
            key = value['shared']
            if key in trail:
                errors.append(dict(context=context, reason='cyclic shared proof', shared=key))
                return False
            if key not in shared:
                errors.append(dict(context=context, reason='unknown shared mechanic', shared=key))
                return False
            return proof(shared[key], context, (*trail, key))
        if set(value) == {'family', 'member'}:
            identity = context['catalogue_id']
            family = families.get(value['family'])
            if family is None or value['member'] != identity:
                errors.append(dict(context=context, reason='unknown or wrong-member spell family'))
                return False
            members = {member['catalogue_id']: member for member in family['members']}
            if identity not in members or not any(member['card'] == family['representative']
                                                  for member in members.values()):
                errors.append(dict(context=context, reason='family needs this member and an actual representative'))
                return False
            if family_models[family['id']] == 'spell':
                gameplay = 'engine::testing::plain_families::representative_outcomes_and_each_members_normal_cast'
                binding = 'cards::coverage_bindings::oracle_families_bind_the_engine_programs'
                casting = gameplay
            elif family_models[family['id']] == 'creature':
                gameplay = 'engine::testing::creature_families::representative_creature_abilities_have_their_printed_outcomes'
                binding = 'cards::creature_coverage_bindings::complete_creature_configurations_bind_the_engine_programs'
                casting = 'engine::testing::normal_permanents::each_ordinary_permanent_pays_its_printed_mana_and_enters'
            else:
                gameplay = 'engine::testing::land_families::representative_land_entry_and_mana_have_their_printed_outcomes'
                binding = 'cards::land_coverage_bindings::complete_land_configurations_bind_the_engine_programs'
                casting = 'engine::testing::land_families::each_land_is_played_from_hand'
            route = 'play/land' if family_models[family['id']] == 'land' else 'cast/normal'
            obligations = [
                (gameplay, 'family/'+family['id'], 'representative effect outcome'),
                (binding, f'card/{identity}/binding/'+family['id'], 'complete engine configuration'),
                (casting, f'card/{identity}/{route}', 'actual casting or land-play route and final zone'),
            ]
            return all(proof(dict(case=case, row=row, outcomes=[outcome]), context)
                       for case, row, outcome in obligations)
        case = value.get('case')
        if case not in passing:
            errors.append(dict(context=context, reason='case did not execute successfully', case=case))
            return False
        row = value.get('row')
        if row is not None and (case, row) not in rows:
            errors.append(dict(context=context, reason='table row did not complete its assertions', case=case, row=row))
            return False
        if not value.get('outcomes'):
            errors.append(dict(context=context, reason='proof needs concrete outcome assertions'))
            return False
        return True

    inventory = {}
    for path in sorted(directory.glob('*.json')):
        if path.name == 'shared.json':
            continue
        data = json.loads(path.read_text())
        if data.get('schema') != 1:
            raise ValueError('unknown card coverage schema: '+str(path))
        for entry in data['cards']:
            identity = entry['catalogue_id']
            if identity in inventory:
                errors.append(dict(context=identity, reason='duplicate card entry'))
            inventory[identity] = entry
    for identity in sorted(inventory.keys()-admitted.keys()):
        errors.append(dict(context=identity, reason='entry is not an admitted physical card'))
    for identity, source in admitted.items():
        card, provenance, facts = source['card'], source['provenance'], source['facts']
        context = dict(catalogue_id=identity, name=card['name'])
        entry = inventory.get(identity)
        if entry is None:
            errors.append(dict(context=context, reason='admitted card has no feature inventory'))
            continue
        start_errors = len(errors)
        if entry.get('oracle_id') != card['oracle_id'] or entry.get('facts_sha256') != provenance['facts_sha256']:
            errors.append(dict(context=context, reason='card identity or pinned facts changed'))
        if facts is None:
            foundation = foundations.get(identity)
            if (foundation is None or foundation['name'] != card['name'] or
                    entry.get('foundation_contract') != identity):
                errors.append(dict(context=context, reason='legacy card needs its literal foundation contract'))
            facts = foundation
        expected = source_paragraphs(facts) if facts is not None else {}
        accounted = set()
        ids = set()
        features = entry.get('features', [])
        routes = entry.get('casting_routes', [])
        if not routes:
            errors.append(dict(context=context, reason='casting routes must be explicitly inventoried'))
        if not features and expected:
            errors.append(dict(context=context, reason='printed abilities are not inventoried'))
        for feature in [*features, *routes]:
            total += 1
            feature_id = feature['id']
            feature_context = dict(**context, feature=feature_id)
            if feature_id in ids:
                errors.append(dict(context=feature_context, reason='duplicate feature id'))
            ids.add(feature_id)
            for location in feature.get('source', []):
                key = (location['face'], location['paragraph'])
                if expected.get(key) != location['text']:
                    errors.append(dict(context=feature_context, reason='printed feature text changed', source=location))
                accounted.add(key)
            if proof(feature.get('proof'), feature_context):
                proved += 1
                value = feature['proof']
                kind = 'composed' if 'family' in value or 'all' in value else 'shared' if 'shared' in value else 'direct'
                proof_types[kind] += 1
        for key in sorted(expected.keys()-accounted):
            errors.append(dict(context=context, reason='unmapped printed paragraph', face=key[0], paragraph=key[1], text=expected[key]))
        if not entry.get('choices_reviewed'):
            errors.append(dict(context=context, reason='spell modes and casting routes need semantic review'))
        if len(errors) == start_errors:
            covered_cards += 1
    return dict(success=not errors and bool(native.get('success')),
                admitted_cards=len(admitted), covered_cards=covered_cards,
                features=total, proved_features=proved, proof_types=proof_types, errors=errors)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    result = check(ROOT, json.loads(args.native.read_text()))
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({key: value for key, value in result.items() if key != 'errors'}))
    for error in result['errors'][:12]:
        print(json.dumps(error), file=sys.stderr)
    return 0 if result['success'] else 1


if __name__ == '__main__':
    sys.exit(main())
