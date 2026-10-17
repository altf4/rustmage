"""Whole physical Adventure pairs and bounded independent immutable spell traits."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('adventure_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Bilbo Baggins, Burglar // Take a Glance','Frolicking Familiar // Blow Off Steam','Galvanic Giant // Storm Reading','Gingerbread Hunter // Puny Snack','Glóin the Mighty // Easy Pickings','Gollum, Silent Slinker // Meager Meal','Grabby Giant // That\'s Mine','Hollow Scavenger // Bakery Raid','Imodane\'s Recruiter // Train Troops','Midgar, City of Mako // Reactor Raid','Minecart Daredevil // Ride the Rails','Obyra\'s Attendants // Desperate Parry','Ratcatcher Trainee // Pest Problem','Shrouded Shepherd // Cleave Shadows','Smaug, the Great Calamity // Spew Flame','Stormkeld Vanguard // Bear Down','Tempest Hart // Scan the Clouds','Threadbind Clique // Rip the Seams','Two-Headed Hunter // Twice the Rage']
NAMES += ['Beanstalk Wurm // Plant Beans', 'Beorn, Reluctant Host // Till and Tend', 'Bellowing Bruiser // Beat a Path', "Beluna's Gatekeeper // Entry Denied", 'Bofur, Reliable Guardian // Concerted Care', 'Ishgard, the Holy See // Faith & Grief', 'Velvetwing Butterflies // Gaze in Wonder', 'Woodland Acolyte // Mend the Wilds']
NAMES += ['Besotted Knight // Betroth the Beast','Conceited Witch // Price of Beauty','Ferocious Werefox // Guard Change','Vantress Transmuter // Croaking Curse']
NAMES += ['Bramble Familiar // Fetch Quest', 'Most Decrepit Old Bird // Speak Secrets', 'Picklock Prankster // Free the Fae', 'Thranduil, Sindarin Liege // Silvan Rally', 'Beluna Grandsquall // Seek Thrills', 'Glamdring, Foe-hammer // Gleam of Death']
class AdventureCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.rows={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  a=[c];return g.compile_catalogue(dict(cards=a,card_count=1,cards_sha256=hashlib.sha256(json.dumps(a,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()),dict(banned_card_names=[]))
 def admitted(self,c):return next(r['implemented']for r in self.compile(c)[1]['cards']if r['name']==c['name'])
 def test_all_thirty_seven_whole_pairs_and_both_immutable_links(self):
  for name in NAMES:
   with self.subTest(name=name):
    rust,report=self.compile(self.rows[name]);self.assertTrue(self.admitted(self.rows[name]));self.assertEqual(report['adventure_definitions'],1)
    self.assertIn('fn adventure_spell',rust);self.assertIn('fn adventure_parent',rust)
    self.assertFalse(any(r['name'].startswith('@adventure/')for r in report['cards']))
    for f in self.rows[name]['card_faces']:self.assertIn(json.dumps(f['name'],ensure_ascii=False),rust)
 def test_each_face_must_compile_all_of_its_residual_text(self):
  for name in NAMES:
   for face in [0,1]:
    c=copy.deepcopy(self.rows[name]);c['card_faces'][face]['oracle_text']+='\nDiscover 3.'
    with self.subTest(name=name,face=face):self.assertFalse(self.admitted(c))
 def test_unknown_parentheticals_on_either_face_fail_closed(self):
  for name in NAMES:
   self.assertTrue(self.admitted(self.rows[name]))
   for face in [0,1]:
    for suffix in [' (You win the game.)',' (Draw a card.)',' (Unknown reminder.)']:
     c=copy.deepcopy(self.rows[name]);c['card_faces'][face]['oracle_text']+=suffix
     with self.subTest(name=name,face=face,suffix=suffix):self.assertFalse(self.admitted(c))
  for name in NAMES:
   for face,f in enumerate(self.rows[name]['card_faces']):
    if 'Sacrifice this token: You gain 3 life.' not in f['oracle_text'] and 'Menace (' not in f['oracle_text']:continue
    c=copy.deepcopy(self.rows[name]);c['card_faces'][face]['oracle_text']=f['oracle_text'].replace('You gain 3 life.', 'You win the game.').replace("two or more creatures.","two or more creatures. You win the game.")
    with self.subTest(name=name,face=face):self.assertFalse(self.admitted(c))
 def test_real_omen_cards_never_get_adventure_definitions(self):
  cards=[c for c in self.rows.values()if c.get('layout')=='adventure'and 'Omen'in c['card_faces'][1]['type_line']]
  self.assertEqual(len(cards),13)
  for c in cards:
   with self.subTest(name=c['name']):
    self.assertIsNone(g.adventure_faces(c));self.assertEqual(self.compile(c)[1]['adventure_definitions'],0)
 def test_all_other_thirty_eight_adventure_parents_remain_wholly_gated(self):
  cards=[c for c in self.rows.values()if c.get('layout')=='adventure'and 'Adventure'in c['card_faces'][1]['type_line']and c['name']not in NAMES]
  self.assertEqual(len(cards),38)
  for c in cards:
   with self.subTest(name=c['name']):self.assertFalse(self.admitted(c))
 def test_malformed_variable_costs_completion_and_parent_metadata_fail_closed(self):
  for name in NAMES:
   for cost in ['{X}{R}','{Y}','{W/P}','{4294967296}','{4294967295}{1}','{R}junk','']:
    c=copy.deepcopy(self.rows[name]);c['card_faces'][1]['mana_cost']=cost;c['mana_cost']=' // '.join(f['mana_cost']for f in c['card_faces']if f['mana_cost'])
    with self.subTest(name=name,cost=cost):self.assertFalse(self.admitted(c))
  original=self.rows[NAMES[0]]
  for field,value in [('name','Wrong metadata'),('mana_cost','{0}'),('type_line','Artifact'),('oracle_text','Draw twenty cards.')]:
   self.assertFalse(self.admitted(dict(original,**{field:value})))
  for body in ['Scry 2. (Then draw fifty cards.)','Scry 2. (Then exile this card. You may cast it without paying its mana cost.)']:
   c=copy.deepcopy(original);c['card_faces'][1]['oracle_text']=body;self.assertFalse(self.admitted(c))
 def test_inset_cost_and_both_programs_change_fingerprint(self):
  original=self.rows[NAMES[0]];a=self.compile(original)[1]['catalogue_version']
  for face,field,value in [(0,'oracle_text','When Bilbo Baggins enters, draw two cards.'),(1,'oracle_text','Scry 1.'),(1,'mana_cost','{2}{U}')]:
   c=copy.deepcopy(original);c['card_faces'][face][field]=value;c['mana_cost']=' // '.join(f['mana_cost']for f in c['card_faces']if f['mana_cost'])
   rust,report=self.compile(c);self.assertTrue(self.admitted(c));self.assertNotEqual(a,report['catalogue_version'])
 def test_recognized_alternative_selectors_do_not_revive_partial_faces(self):
  clauses=[self.rows['Thrill of Possibility']['oracle_text'].splitlines()[0],self.rows['Candy Grapple']['oracle_text'].splitlines()[0],self.rows['Appeal to Eirdu']['oracle_text'].splitlines()[0],self.rows['Analyze the Pollen']['oracle_text'].splitlines()[0],self.rows['Demonic Ruckus']['oracle_text'].splitlines()[-1], 'Kicker {1}', 'Offspring {1}', 'Warp {1}', 'Flashback {1}']
  for face in [0,1]:
   for clause in clauses:
    
    if face==0 and clause==self.rows['Thrill of Possibility']['oracle_text'].splitlines()[0]:continue
    c=copy.deepcopy(self.rows[NAMES[0]]);c['card_faces'][face]['oracle_text']=clause+'\n'+c['card_faces'][face]['oracle_text']
    with self.subTest(face=face,clause=clause):self.assertFalse(self.admitted(c))
if __name__=='__main__':unittest.main()
