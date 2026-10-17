"""Whole-pair admission and exact one-graveyard grammar, without card-name dispatch."""
import copy, hashlib, json, unittest
import test_omen as omen
g=omen.g
NAME='Feral Deathgorger // Dusk Sight'
class FeralCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 compile=omen.OmenCompilerTests.compile
 admitted=omen.OmenCompilerTests.admitted
 def test_same_graveyard_is_typed_and_both_complete_faces_are_required(self):
  c=copy.deepcopy(self.cards[NAME]);rust,m=self.compile(c)
  self.assertTrue(self.admitted(c));self.assertIn('constraint: TargetGroupConstraint::SameGraveyard',rust)
  self.assertIn('Instruction::PutCounters',rust);self.assertIn('Instruction::Draw',rust);self.assertEqual(m['omen_definitions'],1)
  c['name']='Audited Dragon // Audited Omen'
  c['card_faces'][0]['name']='Audited Dragon';c['card_faces'][1]['name']='Audited Omen'
  self.assertTrue(self.admitted(c))
 def test_unknown_constraints_and_residuals_never_become_unrestricted(self):
  for replacement in ['from two graveyards','from a single hand','from a single graveyard of your choice and draw a card','from a single graveyard (Draw a card.)','from a single graveyard (','from a single graveyard ((Then shuffle this card into its owner\'s library.))']:
   c=copy.deepcopy(self.cards[NAME]);c['card_faces'][0]['oracle_text']=c['card_faces'][0]['oracle_text'].replace('from a single graveyard',replacement)
   with self.subTest(replacement=replacement):self.assertFalse(self.admitted(c))
  for face in [0,1]:
   for residual in [' Discover 4.',' (You win the game.)',' (Also exile all graveyards.)','\nExile all creatures.']:
    c=copy.deepcopy(self.cards[NAME]);c['card_faces'][face]['oracle_text']+=residual
    with self.subTest(face=face,residual=residual):self.assertFalse(self.admitted(c))
 def test_same_graveyard_parenthetical_closure_survives_case_and_edge_whitespace(self):
  for name in [NAME,'Arashin Sunshield','Griffnaut Tracker']:
   for prefix in ['', '  ', '\t', '\u2003']:
    c=copy.deepcopy(self.cards[name]);face=c['card_faces'][0] if c.get('card_faces') else c
    face['oracle_text']='\n'.join(prefix+line for line in face['oracle_text'].splitlines())+' (You win the game.)'
    self.assertFalse(self.admitted(c),(name,prefix))
 def test_unknown_parenthetical_cannot_split_the_constraint_route_literal(self):
  for name in [NAME,'Arashin Sunshield','Griffnaut Tracker']:
   for replacement in ['single(You win the game.) graveyard','single (Draw a card.)graveyard','single(You win the game.)(Draw a card.) graveyard','single(You win the game.) GRAVEYARD']:
    c=copy.deepcopy(self.cards[name]);face=c['card_faces'][0] if c.get('card_faces') else c
    face['oracle_text']=face['oracle_text'].replace('single graveyard',replacement)
    self.assertFalse(self.admitted(c),(name,replacement))
 def test_unconstrained_graveyard_template_retains_its_distinct_semantics(self):
  self.assertEqual(g.targeted_program('exile up to two target cards from graveyards.')[0],('CardGrave',0,2))
  self.assertEqual(g.targeted_program('exile up to two target cards from a single graveyard.')[0],('CardSingleGrave',0,2))
 def test_all_feral_provenance_records_match_checked_out_and_pinned_java(self):
  import subprocess
  m=json.loads((g.ROOT/'docs/omen-admissions.json').read_text());entry=next(c for c in m['cards']if c['name']==NAME)
  self.assertEqual(entry['status'],'complete_pair_admitted');self.assertIsNone(entry['remaining_obligation'])
  self.assertTrue(any(s['java_source'].endswith('/TargetCardInASingleGraveyard.java')for s in entry['sources']))
  for source in entry['sources']:
   p=source['java_source'];expected=source['java_sha256']
   self.assertEqual(hashlib.sha256((__import__('pathlib').Path('/home/altf4/Code/mage')/p).read_bytes()).hexdigest(),expected,p)
   blob=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',m['java_revision']+':'+p])
   self.assertEqual(hashlib.sha256(blob).hexdigest(),expected,p)
 def test_complete_other_bodies_and_full_body_negative_controls(self):
  for name in ['Arashin Sunshield','Griffnaut Tracker']:
   self.assertTrue(self.admitted(self.cards[name]))
   c=copy.deepcopy(self.cards[name]);c['name']='Audited Graveyard Creature';self.assertTrue(self.admitted(c))
   for extra in [' Discover 4.',' (You win the game.)','\nWhenever you draw a card, discover 4.']:
    c=copy.deepcopy(self.cards[name]);c['oracle_text']+=extra
    with self.subTest(name=name,extra=extra):self.assertFalse(self.admitted(c))
if __name__=='__main__':unittest.main()
