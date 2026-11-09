"""Offline supplementary facts preserve Standard legality and canonical identities."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_sources
import link_catalogue
import link_outside_names
import oracle_facts
import set_status

ROOT = Path(__file__).resolve().parents[1]
CUTTER = '46b1a169-bc32-47bf-b379-fa0c3d13878f'


class _FactsFixture(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.legal = dict(
            oracle_id='11111111-1111-4111-8111-111111111111',
            name='Legal Fixture', layout='normal', mana_cost='',
            type_line='Artifact', oracle_text='', colors=[],
            reference_printing=dict(set='tst'))
        self.standard = dict(schema=1, as_of='2026-10-15', card_count=1,
                             cards=[self.legal])
        self.standard['cards_sha256'] = oracle_facts.digest(self.standard['cards'])
        self.write('data/standard/cards.json', self.standard)
        self.target_path = 'data/targets/standard-2026-10-15.json'
        self.target = dict(schema=1, as_of='2026-10-15',
                           banned_card_names=['Cori-Steel Cutter'])
        self.write(self.target_path, self.target)
        self.supplement = json.loads((ROOT/oracle_facts.SUPPLEMENT).read_text())
        self.supplement['cards'] = [next(entry for entry in self.supplement['cards']
                                         if entry['facts']['oracle_id'] == CUTTER)]
        self.supplement['oracle_cards_sha256'] = self.standard['cards_sha256']
        self.pin_target(self.supplement)
        self.write_supplement(self.supplement)

    def write(self, relative, value):
        path = self.root/relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n')

    def pin_target(self, document):
        document['target'] = dict(
            path=self.target_path,
            sha256=hashlib.sha256((self.root/self.target_path).read_bytes()).hexdigest())

    def write_supplement(self, document):
        self.write(oracle_facts.SUPPLEMENT, document)


class OracleFactsTests(_FactsFixture):
    def test_supplement_adds_facts_without_changing_the_legal_pool_or_snapshot(self):
        before = (self.root/'data/standard/cards.json').read_bytes()
        facts, standard_ids = oracle_facts.load(self.root)
        self.assertEqual(set(facts), {self.legal['oracle_id'], CUTTER})
        self.assertEqual(standard_ids, {self.legal['oracle_id']})
        self.assertNotIn(CUTTER, standard_ids)
        self.assertEqual((self.root/'data/standard/cards.json').read_bytes(), before)
        (self.root/oracle_facts.SUPPLEMENT).unlink()
        facts, standard_ids = oracle_facts.load(self.root)
        self.assertEqual(set(facts), standard_ids)

    def test_supplement_rejects_altered_facts_and_source_pins(self):
        mutations = [
            ('facts', lambda d: d['cards'][0].update(facts_sha256='0'*64)),
            ('snapshot', lambda d: d.update(oracle_cards_sha256='0'*64)),
            ('target', lambda d: d['target'].update(sha256='0'*64)),
            ('schema', lambda d: d.update(schema=2)),
            ('source', lambda d: d['cards'][0].update(source='https://api.scryfall.com.invalid/card')),
            ('response artifact', lambda d: d['cards'][0].update(source_response_sha256='unknown')),
        ]
        for label, mutate in mutations:
            with self.subTest(label=label):
                document = copy.deepcopy(self.supplement)
                mutate(document)
                self.write_supplement(document)
                with self.assertRaises(ValueError):
                    oracle_facts.load(self.root)

    def test_supplement_cannot_grant_legality_or_contradict_the_pinned_ban(self):
        for field, value in [('standard_legal', True), ('standard_legal', 0),
                             ('standard_status', 'legal'), ('standard_status', 'not_legal')]:
            with self.subTest(field=field, value=value):
                document = copy.deepcopy(self.supplement)
                document['cards'][0][field] = value
                self.write_supplement(document)
                with self.assertRaises(ValueError):
                    oracle_facts.load(self.root)

    def test_nonlegal_supplement_is_distinct_from_a_banned_member(self):
        document = copy.deepcopy(self.supplement)
        entry = document['cards'][0]
        entry['facts']['name'] = 'Nonlegal Fixture'
        entry['facts_sha256'] = oracle_facts.digest(entry['facts'])
        entry['standard_status'] = 'not_legal'
        self.write_supplement(document)
        facts, standard_ids = oracle_facts.load(self.root)
        self.assertEqual(facts[CUTTER]['name'], 'Nonlegal Fixture')
        self.assertNotIn(CUTTER, standard_ids)

    def test_duplicate_standard_and_supplemental_identities_are_rejected(self):
        document = copy.deepcopy(self.supplement)
        document['cards'].append(copy.deepcopy(document['cards'][0]))
        self.write_supplement(document)
        with self.assertRaisesRegex(ValueError, 'duplicate supplemental Oracle identity'):
            oracle_facts.load(self.root)
        document = copy.deepcopy(self.supplement)
        document['cards'][0]['facts'] = self.legal
        document['cards'][0]['facts_sha256'] = oracle_facts.digest(self.legal)
        self.write_supplement(document)
        with self.assertRaisesRegex(ValueError, 'duplicate supplemental Oracle identity'):
            oracle_facts.load(self.root)

    def test_normalized_full_and_front_face_aliases_cannot_shadow_an_identity(self):
        for name in ['LÉGAL Fixture', 'Légal Fixture // Companion']:
            with self.subTest(name=name):
                document = copy.deepcopy(self.supplement)
                entry = document['cards'][0]
                entry['facts']['name'] = name
                entry['facts_sha256'] = oracle_facts.digest(entry['facts'])
                entry['standard_status'] = 'not_legal'
                self.write_supplement(document)
                with self.assertRaisesRegex(ValueError, 'ambiguous Oracle identity'):
                    oracle_facts.load(self.root)

    def test_target_paths_are_confined_to_the_pinned_repository_target_directory(self):
        for path in ['/tmp/foreign-target.json', '../data/targets/target.json',
                     'data/standard/cards.json', 'data/targets/../target.json']:
            with self.subTest(path=path):
                document = copy.deepcopy(self.supplement)
                document['target']['path'] = path
                self.write_supplement(document)
                with self.assertRaisesRegex(ValueError, 'invalid supplemental target path'):
                    oracle_facts.load(self.root)

    def test_repinning_a_different_target_date_does_not_refresh_standard_implicitly(self):
        self.target['as_of'] = '2027-01-01'
        self.write(self.target_path, self.target)
        self.pin_target(self.supplement)
        self.write_supplement(self.supplement)
        with self.assertRaisesRegex(ValueError, 'target and Oracle dates disagree'):
            oracle_facts.load(self.root)

    def test_missing_rules_body_cannot_be_hidden_by_rehashing(self):
        document = copy.deepcopy(self.supplement)
        entry = document['cards'][0]
        del entry['facts']['oracle_text']
        entry['facts_sha256'] = oracle_facts.digest(entry['facts'])
        self.write_supplement(document)
        with self.assertRaisesRegex(ValueError, 'missing explicit rules text'):
            oracle_facts.load(self.root)

    def test_standard_integrity_and_identity_count_are_authenticated(self):
        changed = copy.deepcopy(self.standard)
        changed['cards'][0]['name'] = 'Changed without a reviewed snapshot'
        self.write('data/standard/cards.json', changed)
        with self.assertRaisesRegex(ValueError, 'frozen Standard Oracle facts changed'):
            oracle_facts.load(self.root)
        changed = copy.deepcopy(self.standard)
        changed['cards'].append(copy.deepcopy(changed['cards'][0]))
        changed['card_count'] = 2
        changed['cards_sha256'] = oracle_facts.digest(changed['cards'])
        self.write('data/standard/cards.json', changed)
        with self.assertRaisesRegex(ValueError, 'duplicate or incomplete Standard Oracle identities'):
            oracle_facts.load(self.root)


class CatalogueFactsTests(_FactsFixture):
    def setUp(self):
        super().setUp()
        actual = json.loads((ROOT/'data/catalogue-identities.json').read_text())
        legal = dict(actual['cards'][849], index=0, module='card_0000',
                     path='fixture/legal.rs', set='fixture', name=self.legal['name'],
                     oracle_id=self.legal['oracle_id'], facts_sha256=oracle_facts.digest(self.legal),
                     standard_legal=True, mana_cost='', type_line='Artifact',
                     oracle_keywords=[], colored=[0]*6, generic=0)
        cutter = dict(actual['cards'][849], index=1, module='card_0001')
        self.metadata = dict(schema=1, cards=[legal, cutter], colors=[0, 8],
                             physical_count=2)
        self.write('data/catalogue-identities.json', self.metadata)
        self.records = []
        for card in self.metadata['cards']:
            if card['index'] == 1:
                record = json.loads((ROOT/'src/card_definitions/tdm/cori_steel_cutter.json').read_text())
                record['catalogue_id'] = 1
                record['complete'] = False
            else:
                record = dict(name=card['name'], catalogue_id=0,
                              oracle_id=card['oracle_id'], facts_sha256=card['facts_sha256'],
                              complete=False, typed_lift=dict(revision='a'*40, definition_sha256='b'*64))
            record['physical_metadata_sha256'] = link_catalogue.physical_digest(self.metadata, card)
            self.records.append(record)
            self.write(Path('src/card_definitions')/Path(card['path']).with_suffix('.json'), record)
            (self.root/'src/card_definitions'/card['path']).write_text(
                'pub(in crate::cards) const RULES: CardRules = CardRules {\n'
                '    effect: Effect::Unsupported, ..CardRules::EMPTY\n};\n')
        module = self.root/'crates/engine/src/card_definitions/mod.rs'
        module.parent.mkdir(parents=True)
        module.write_text('// Shared vocabulary fixture, no catalogue identities.\n')

    def test_linker_accepts_supplemental_provenance_but_keeps_it_gated(self):
        metadata, records, _ = link_catalogue.load(self.root)
        self.assertEqual([c['index'] for c in metadata['cards']], [0, 1])
        self.assertFalse(metadata['cards'][1]['standard_legal'])
        self.assertFalse(records[1]['complete'])
        self.assertEqual(records[1]['relative'], 'src/card_definitions/tdm/cori_steel_cutter.rs')

    def test_linker_rejects_legality_expansion_and_removal_of_a_legal_identity(self):
        for index, value in [(1, True), (0, False)]:
            with self.subTest(index=index):
                metadata = copy.deepcopy(self.metadata)
                metadata['cards'][index]['standard_legal'] = value
                self.write('data/catalogue-identities.json', metadata)
                with self.assertRaisesRegex(ValueError, 'Standard legality differs from frozen inventory'):
                    link_catalogue.load(self.root)

    def test_missing_supplement_fails_an_existing_catalogue_dependency(self):
        (self.root/oracle_facts.SUPPLEMENT).unlink()
        with self.assertRaisesRegex(ValueError, 'missing pinned Oracle facts: Cori-Steel Cutter'):
            link_catalogue.load(self.root)

    def test_oracle_and_physical_metadata_provenance_cannot_drift(self):
        for field in ['facts_sha256', 'physical_metadata_sha256']:
            with self.subTest(field=field):
                record = dict(self.records[1], **{field: '0'*64})
                self.write('src/card_definitions/tdm/cori_steel_cutter.json', record)
                with self.assertRaisesRegex(ValueError, 'pinned (facts|physical metadata) changed'):
                    link_catalogue.load(self.root)

    def test_canonical_move_rejects_old_owner_and_duplicate_definition(self):
        self.write('src/card_definitions/legacy/cori_steel_cutter.json', self.records[1])
        with self.assertRaisesRegex(ValueError, 'pinned canonical set/path'):
            link_catalogue.load(self.root)
        (self.root/'src/card_definitions/legacy/cori_steel_cutter.json').unlink()
        self.write('src/card_definitions/zz_duplicate/cori_steel_cutter.json', self.records[1])
        with self.assertRaisesRegex(ValueError, 'duplicate card definition'):
            link_catalogue.load(self.root)

    def test_catalogue_cannot_duplicate_a_physical_oracle_identity(self):
        duplicate = dict(self.metadata['cards'][1], index=2, module='card_0002',
                         path='tdm/duplicate.rs')
        self.metadata['cards'].append(duplicate)
        self.metadata['colors'].append(8)
        self.metadata['physical_count'] += 1
        self.write('data/catalogue-identities.json', self.metadata)
        with self.assertRaisesRegex(ValueError, 'duplicate physical Oracle identity'):
            link_catalogue.load(self.root)

    def test_java_provenance_uses_real_direct_source_and_dependencies_without_inventory_fiction(self):
        inventory = dict(set_tables={})
        record = self.records[1]
        pins = check_sources.source_pins({1: record}, inventory)
        self.assertGreaterEqual(len(pins), 10)
        revision = '492369c7844b8d76479f0393ff9eb6b0b0cf30d7'
        self.assertEqual(pins[revision, 'Mage.Sets/src/mage/cards/c/CoriSteelCutter.java'],
                         'b5dfef3b81079fdda2fd97637933aa37713d96224a0aceff8224d9682672fdf1')
        self.assertIn((revision, 'Mage/src/main/java/mage/game/permanent/token/MonasteryMentorToken.java'), pins)
        self.assertNotIn('xmage_absence', record)
        conflicting = copy.deepcopy(record)
        duplicate = dict(conflicting['xmage_dependencies'][0], sha256='0'*64)
        conflicting['xmage_dependencies'].append(duplicate)
        with self.assertRaisesRegex(ValueError, 'conflicting source digests'):
            check_sources.source_pins({1: conflicting}, inventory)
        malformed = dict(record, java_sha256='not-a-source-hash')
        with self.assertRaisesRegex(ValueError, 'invalid source pin'):
            check_sources.source_pins({1: malformed}, inventory)

    def set_table(self, unknown=False):
        table = 'Mage.Sets/src/mage/sets/Fixture.java'
        entries = [('Legal Fixture', 1), ('Cori-Steel Cutter', 103), ('Cori-Steel Cutter', 343)]
        if unknown:
            entries.append(('Unresolved Fixture', 500))
        raw = '\n'.join(f'new SetCardInfo("{name}", {number}, Rarity.RARE, mage.cards.c.Fixture.class);'
                        for name, number in entries).encode()
        xmage = self.root/'xmage'
        path = xmage/table
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)
        self.write('data/standard/xmage-inventory.json', dict(
            xmage_source_commit='a'*40, set_tables=dict(TDM=dict(
                source=table, sha256=hashlib.sha256(raw).hexdigest(), printing_count=len(entries)))))
        self.write('data/standard/coverage.json', dict(cards=[
            dict(name='Legal Fixture', implemented=True),
            dict(name='Cori-Steel Cutter', implemented=False)]))
        return xmage

    def test_freeze_reconciles_all_printings_once_and_status_stays_gated(self):
        manifest = set_status.freeze(self.root, self.set_table(), 'TDM')
        self.assertEqual(manifest['card_count'], 2)
        self.assertEqual(manifest['unresolved'], [])
        cutter = next(c for c in manifest['cards'] if c['oracle_id'] == CUTTER)
        self.assertEqual(cutter['printings'], ['103', '343'])
        status = set_status.status(self.root, manifest)
        self.assertEqual(status['gated'], ['Cori-Steel Cutter'])
        self.assertEqual(status['admitted'], ['Legal Fixture'])
        self.assertEqual(len(oracle_facts.load(self.root)[1]), 1)
        cutter['facts_sha256'] = '0'*64
        with self.assertRaisesRegex(ValueError, 'set facts changed'):
            set_status.status(self.root, manifest)

    def test_freeze_keeps_unresolved_table_identities_in_the_denominator(self):
        manifest = set_status.freeze(self.root, self.set_table(unknown=True), 'TDM')
        self.assertEqual(manifest['card_count'], 3)
        self.assertEqual(len(manifest['cards']), 2)
        self.assertEqual(manifest['unresolved'], ['Unresolved Fixture'])

    def test_outside_names_uses_the_same_nonduplicated_oracle_union(self):
        with patch.object(link_outside_names, 'ROOT', self.root):
            rendered = link_outside_names.render()
        self.assertEqual(set(rendered.splitlines()), {
            self.legal['oracle_id']+'\tLegal Fixture', CUTTER+'\tCori-Steel Cutter'})

    def test_real_render_preserves_engine_and_standard_pool_with_existing_ids_and_monk(self):
        metadata, records, version = link_catalogue.load(ROOT)
        facts, standard_ids = oracle_facts.load(ROOT)
        card = metadata['cards'][849]
        self.assertEqual((card['index'], card['module'], card['oracle_id']), (849, 'card_0849', CUTTER))
        self.assertEqual((card['path'], card['mana_cost'], card['type_line']),
                         ('tdm/cori_steel_cutter.rs', '{1}{R}', 'Artifact — Equipment'))
        self.assertEqual((card['kind'], metadata['colors'][849], card['colored'], card['generic']),
                         (16, 8, [0, 0, 0, 1, 0, 0], 1))
        self.assertEqual(records[849]['physical_metadata_sha256'],
                         '33156c4ab3eb6a75c0fe58449f60933e2d771f72acc97b4b9b6df63b90e35493')
        self.assertFalse(card['standard_legal'])
        self.assertNotIn(CUTTER, standard_ids)
        supplements = json.loads((ROOT/oracle_facts.SUPPLEMENT).read_text())['cards']
        self.assertEqual(len(facts)-len(standard_ids), len(supplements))
        self.assertEqual(metadata['aliases'].count(['Cori-Steel Cutter', 849]), 1)
        monk = metadata['cards'][5486]
        self.assertEqual((monk['index'], monk['name'], monk['path'], metadata['colors'][5486]),
                         (5486, '@token/tdm-monk-prowess', 'synthetic/token_tdm_monk_prowess.rs', 1))
        outputs = link_catalogue.render(metadata, records, version)
        engine = ROOT/'crates/engine/src/catalogue_accessors.rs'
        self.assertEqual(outputs[engine], engine.read_text())
        coverage = json.loads(outputs[ROOT/'data/standard/coverage.json'])
        meld_results = sum(oracle_facts.is_meld_result(facts[oracle_id]) for oracle_id in standard_ids)
        self.assertEqual(coverage['standard_legal_cards'], len(standard_ids) - meld_results)
        meld = next(c for c in coverage['cards'] if c['name'] == 'Ragnarok, Divine Deliverance')
        self.assertFalse(meld['standard_legal'])
        row = next(c for c in coverage['cards'] if c['name'] == 'Cori-Steel Cutter')
        self.assertEqual((row['set'], row['implemented'], row['standard_legal']),
                         ('tdm', records[849]['complete'], False))
        self.assertIn(CUTTER+'\tCori-Steel Cutter\n', outputs[ROOT/'data/outside-card-names.tsv'])
        self.assertIn('/src/card_definitions/tdm/cori_steel_cutter.rs',
                      outputs[ROOT/'crates/cards/tdm/src/lib.rs'])
        self.assertNotIn('cori_steel_cutter.rs', outputs[ROOT/'crates/cards/legacy/src/lib.rs'])


if __name__ == '__main__':
    unittest.main()
