"""Full explore bodies and closed public-reveal/token semantics."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('explore_generate',Path(__file__).with_name('generate_catalogue.py'))
g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
class ExploreCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.snapshot=json.loads((g.ROOT/'data/standard/cards.json').read_text());cls.rows={c['name']:c for c in cls.snapshot['cards']};cls.target=json.loads((g.ROOT/'data/targets/standard-2026-10-15.json').read_text());cls.manifest=json.loads((g.ROOT/'docs/explore-admissions.json').read_text())
    def test_exact_source_target_and_sequential_double_explore_grammar(self):
        self.assertEqual(g.statement_program('This creature explores.'),['Explore(Subject::Source)'])
        self.assertEqual(g.targeted_program('Target creature you control explores.'),('CreatureYou',['Explore(Subject::Targets)']))
        self.assertEqual(g.targeted_program('Target creature you control explores, then it explores again.'),('CreatureYou',['Explore(Subject::Targets)','Explore(Subject::Targets)']))
        for text in ['This creature explores X times.','This creature explores. If a land is revealed, draw a card.','Explore.','Each creature you control explores.']:
            self.assertIsNone(g.statement_program(text),text)
        for text in ['Target creature explores.','Target creature you control explores three times.','Return target creature card from your graveyard to the battlefield. That creature explores, then it explores again.','Target creature you control explores, then another creature explores.']:
            self.assertIsNone(g.targeted_program(text),text)
    def test_complete_map_recipe_cost_target_and_timing(self):
        result=g.predefined_program('create a Map token.');self.assertIsNotNone(result)
        factory=next(v for v in g.TOKEN_DEFS.values() if v.get('predefined')=='Map')
        self.assertEqual((factory['subtypes'],factory['colors'],factory['power'],factory['extra_type']),('Map',0,None,'artifact'))
        row=dict(layout='normal',type_line='Artifact',mana_cost='{0}',oracle_text=g.PREDEFINED['Map'])
        ability=g.permanent_program_template(row)[3][0];cost=ability[0]
        self.assertEqual((cost['generic'],cost['tap'],cost['sacrifice']),(1,True,True))
        self.assertEqual(ability[1:4],('CreatureYou',['Explore(Subject::Targets)'],True))
        self.assertNotIn('AddMana',str(ability))
        self.assertIsNone(g.permanent_program_template(dict(row,oracle_text=row['oracle_text']+' Discover 3.')))
    def test_get_lost_single_old_controller_operation_and_closed_residual(self):
        program=g.targeted_program('Destroy target creature, enchantment, or planeswalker. Its controller creates two Map tokens.')
        self.assertEqual(program[0],'CreatureEnchantmentPlaneswalker');self.assertEqual(len(program[1]),1);self.assertTrue(program[1][0].startswith('DestroyAndCreateForController'))
        for text in ['Destroy target creature, enchantment, or planeswalker. If it died, its controller creates two Map tokens.','Destroy target creature, enchantment, or planeswalker. Its owner creates two Map tokens.','Destroy target creature, enchantment, or planeswalker. Its controller creates two Map tokens. Discover 3.']:
            self.assertIsNone(g.targeted_program(text),text)
    def test_all_fifteen_full_bodies_and_every_unknown_tail_rejects(self):
        self.assertEqual(len(self.manifest['cards']),15);source,coverage=g.compile_catalogue(self.snapshot,self.target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for row in self.manifest['cards']:
            self.assertEqual(row['oracle_text'],self.rows[row['name']]['oracle_text']);self.assertIn(row['name'],supported)
        for row in self.manifest['gated']:self.assertNotIn(row['name'],supported)
        later=self.manifest['subsequently_completed'];self.assertEqual({c['name']for c in later},{'Nicanzil, Current Conductor'})
        for row in later:self.assertIn(row['name'],supported);self.assertEqual(row['oracle_text'],self.rows[row['name']]['oracle_text'])
        names={c['name'] for c in self.manifest['cards']+later}
        cards=[dict(c,oracle_text=c['oracle_text']+'\nDiscover 3.') if c['name'] in names else c for c in self.snapshot['cards']]
        snapshot=dict(self.snapshot,cards=cards,cards_sha256=hashlib.sha256(json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest())
        _,invalid=g.compile_catalogue(snapshot,self.target);self.assertFalse(names&{c['name'] for c in invalid['cards'] if c['implemented']})
        self.assertIn('CreatureExploresYou',source);self.assertIn('Explore(Subject::Targets)',source)
