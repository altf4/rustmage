"""Complete pinned source inventory for effective base-power consumers."""
import hashlib
import json
from pathlib import Path
import subprocess
import unittest
from scripts import generate_catalogue as g


class BasePowerAuditTests(unittest.TestCase):
    def test_three_complete_physical_bodies_and_direct_source_pins(self):
        audit = json.loads((g.ROOT / 'docs/base-power-admissions.json').read_text())
        facts = {c['name']: c for c in json.loads((g.ROOT / 'data/standard/cards.json').read_text())['cards']}
        expected = {'Kutzil, Malamet Exemplar', 'Sovereign Okinec Ahau', 'Voice of Victory'}
        self.assertEqual(set(audit['proposed_complete']), expected)
        self.assertEqual({r['name'] for r in audit['cards']}, expected)
        seen = {}
        records = 0
        for row in audit['cards']:
            self.assertEqual(row['facts'], facts[row['name']])
            for item in [row, *row['direct_import_sources']]:
                records += 1
                path, digest = item['java_source'], item['java_sha256']
                self.assertEqual(seen.setdefault(path, digest), digest)
        self.assertEqual(records, 35)
        self.assertEqual(audit['source_records'], records)
        self.assertEqual(len(seen), 25)
        self.assertEqual(audit['unique_source_count'], len(seen))
        self.assertEqual({r['java_source']: r['java_sha256'] for r in audit['unique_sources']}, seen)
        for path, digest in seen.items():
            blob = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show', audit['java_revision'] + ':' + path])
            self.assertEqual(hashlib.sha256(blob).hexdigest(), digest)
            self.assertEqual((Path('/home/altf4/Code/mage') / path).read_bytes(), blob)


class BasePowerCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cards = json.loads((g.ROOT / 'data/standard/cards.json').read_text())['cards']
        cls.names = {'Kutzil, Malamet Exemplar', 'Sovereign Okinec Ahau', 'Voice of Victory'}
        cls.rows = [c for c in cls.cards if c['name'] in cls.names]

    def compile(self, card):
        payload = json.dumps([card], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
        rust, report = g.compile_catalogue(dict(cards=[card], card_count=1, cards_sha256=hashlib.sha256(payload).hexdigest()), dict(banned_card_names=[]))
        return rust, next(r['implemented'] for r in report['cards'] if r['name'] == card['name'])

    def test_all_three_whole_bodies_and_renamed_semantics_reject_unknown_residuals(self):
        for card in self.rows:
            self.assertTrue(self.compile(card)[1], card['name'])
            for name in ['Generic qualified source', *[row[0] for row in g.LEGACY]]:
                raw = card['oracle_text'].replace(card['name'], name)
                self.assertTrue(self.compile(dict(card, name=name, oracle_text=raw))[1], (card['name'], name))
            for line in card['oracle_text'].splitlines():
                for bad in [line + ' (You win the game.)', '(You win the game.) ' + line, line + ' ((Draw seven cards.))']:
                    self.assertFalse(self.compile(dict(card, oracle_text=card['oracle_text'].replace(line, bad)))[1], bad)
            for cost in ['{X}', '{Q}', '{4294967296}', '{W/P}']:
                self.assertFalse(self.compile(dict(card, mana_cost=cost))[1], (card['name'], cost))
            for layout in ['transform', 'split', 'modal_dfc']:
                self.assertFalse(self.compile(dict(card, layout=layout))[1])
            self.assertFalse(self.compile(dict(card, oracle_text=card['oracle_text'] + '\nDiscover 4.'))[1])

    def test_generic_complete_clauses_bind_the_actual_base_query_and_mobilize_reminder(self):
        rows = {c['name']: c for c in self.rows}
        sovereign = rows['Sovereign Okinec Ahau']
        kutzil = rows['Kutzil, Malamet Exemplar']
        voice = rows['Voice of Victory']
        self.assertIn('PutPowerExcessCounters', self.compile(sovereign)[0])
        self.assertIn('DamageQualification::PowerAboveBase', self.compile(kutzil)[0])
        self.assertIn('static_cast_restriction', self.compile(voice)[0])
        for card in self.rows:
            for word in set(card['oracle_text'].split()):
                if len(word) < 3: continue
                bad = card['oracle_text'].replace(word, word[:1] + '(You win the game.)' + word[1:], 1)
                self.assertFalse(self.compile(dict(card, oracle_text=bad))[1], (card['name'], bad))
        raw = voice['oracle_text']
        for bad in [raw.replace('create two', 'create three'), raw.replace('1/1 red', '2/2 red'), raw.replace('Warrior', 'Goblin'), raw.replace('next end step', 'next upkeep'), raw.replace('Sacrifice them', 'Draw a card. Sacrifice them')]:
            self.assertFalse(self.compile(dict(voice, oracle_text=bad))[1], bad)
        for card in [sovereign, kutzil]:
            self.assertFalse(self.compile(dict(card, oracle_text=card['oracle_text'].replace('base power', 'printed power')))[1])

    def test_exact_actual_parent_three_body_union_and_unrelated_mutation_control(self):
        import copy
        from scripts.catalogue_preservation import canonical, assert_union
        physical = {c['oracle_id']: c['name'] for c in self.cards}
        parent = 'dffafa6caaf7288ae8bbb8fa4c90b206e5960b94'
        before = canonical(subprocess.check_output(['git', '-C', str(g.ROOT), 'show', parent + ':src/generated/catalogue.rs'], text=True), physical)
        after = canonical(subprocess.check_output(['git', '-C', str(g.ROOT), 'show', 'aab3560:src/generated/catalogue.rs'], text=True), physical)
        current = canonical((g.ROOT / 'src/generated/catalogue.rs').read_text(), physical)
        args = dict(changed={n+'#0' for n in self.names}, added=set(), helpers={('static_cast_restriction', n+'#0') for n in ['Kutzil, Malamet Exemplar','Voice of Victory']}, aliases={})
        proof = assert_union(before, after, **args)
        self.assertEqual(proof['preserved_definitions'], 5329)
        self.assertEqual(proof['prior_helpers'], 924)
        for name in self.names: self.assertEqual(current['definitions'][name+'#0'], after['definitions'][name+'#0'])
        for helper in args['helpers']: self.assertEqual(current['helpers'][helper], after['helpers'][helper])
        mutant = copy.deepcopy(after)
        mutant['definitions']['Plains#0'] = 'unrelated mutation'
        with self.assertRaises(AssertionError): assert_union(before, mutant, **args)


    def test_actual_joint_land_parent_preserves_every_prior_program_and_recipe(self):
        import copy
        from scripts.catalogue_preservation import canonical, assert_union
        physical = {c['oracle_id']: c['name'] for c in self.cards}
        parent = 'f35e95302aad7924be38def41598dcf9013c0fb7'
        before = canonical(subprocess.check_output(['git', '-C', str(g.ROOT), 'show', parent+':src/generated/catalogue.rs'], text=True),physical)
        after = canonical(subprocess.check_output(['git', '-C', str(g.ROOT), 'show', '9168218cce337b8f39ec85e4e1d227e6e4e8aa71:src/generated/catalogue.rs'], text=True), physical)
        current = canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
        for name in self.names: self.assertEqual(current['definitions'][name+'#0'], after['definitions'][name+'#0'])
        for name in ['Kutzil, Malamet Exemplar','Voice of Victory']:
            self.assertEqual(current['helpers'][('static_cast_restriction',name+'#0')], after['helpers'][('static_cast_restriction',name+'#0')])
        args = dict(changed={n+'#0'for n in self.names},added=set(),helpers={('static_cast_restriction',n+'#0')for n in ['Kutzil, Malamet Exemplar','Voice of Victory']},aliases={})
        proof=assert_union(before,after,**args)
        self.assertEqual(proof['preserved_definitions'],5330)
        self.assertEqual(proof['prior_helpers'],926)
        self.assertEqual(proof['token_recipes'],96)
        mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
        with self.assertRaises(AssertionError):assert_union(before,mutant,**args)


if __name__ == '__main__':
    unittest.main()
