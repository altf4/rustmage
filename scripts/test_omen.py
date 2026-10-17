"""Strict whole Omen pairs: chosen face parsing and no erased residuals."""
import copy,hashlib,importlib.util,json,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('omen_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Dirgur Island Dragon // Skimming Strike','Disruptive Stormbrood // Petty Revenge','Riling Dawnbreaker // Signaling Roar','Sagu Wildling // Roost Seek','Twinmaw Stormbrood // Charring Bite','Marang River Regent // Coil and Catch','Purging Stormbrood // Absorb Essence','Runescale Stormbrood // Chilling Screech','Stormshriek Feral // Flush Out']
class OmenCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  a=[c];return g.compile_catalogue(dict(cards=a,card_count=1,cards_sha256=hashlib.sha256(json.dumps(a,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()),dict(banned_card_names=[]))
 def admitted(self,c):return next(v['implemented'] for v in self.compile(c)[1]['cards'] if v['name']==c['name'])
 def test_nine_complete_pairs_have_distinct_omen_not_adventure_definitions(self):
  for n in NAMES:
   with self.subTest(card=n):
    rust,m=self.compile(self.cards[n]);self.assertTrue(self.admitted(self.cards[n]));self.assertEqual(m['omen_definitions'],1);self.assertEqual(m['adventure_definitions'],0);self.assertIn('fn omen_spell',rust)
 def test_every_face_rejects_unknown_residuals_and_parentheses(self):
  for n in NAMES:
   for face in [0,1]:
    for clause in [' Discover 3.',' (Draw twenty cards.)',' (Then exile this card.)',' (Also shuffle another card.)']:
     c=copy.deepcopy(self.cards[n]);c['card_faces'][face]['oracle_text']+=clause
     with self.subTest(card=n,face=face,clause=clause):self.assertFalse(self.admitted(c))
 def test_four_remaining_omen_bodies_remain_tracked_and_wholly_gated(self):
  remaining=[c for c in self.cards.values() if c.get('card_faces') and 'Omen' in c['card_faces'][-1]['type_line'] and c['name'] not in NAMES];self.assertEqual(len(remaining),4)
  for c in remaining:self.assertFalse(self.admitted(c),c['name'])
 def test_variable_malformed_and_wrong_reminder_cannot_revive_face(self):
  for n in NAMES:
   for cost in ['{X}{U}','{Y}','{W/P}','{4294967296}','']:
    c=copy.deepcopy(self.cards[n]);c['card_faces'][1]['mana_cost']=cost;c['mana_cost']=' // '.join(f['mana_cost'] for f in c['card_faces'] if f['mana_cost']);self.assertFalse(self.admitted(c))
  c=copy.deepcopy(self.cards[NAMES[0]]);c['card_faces'][0]['oracle_text']=c['card_faces'][0]['oracle_text'].replace('pays {2}.','pays {3}.');self.assertFalse(self.admitted(c))
if __name__=='__main__':unittest.main()
