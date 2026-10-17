"""Strict whole-pair admission for reusable land allowances and target movement."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('adventure_land_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Bramble Familiar // Fetch Quest', 'Most Decrepit Old Bird // Speak Secrets', 'Picklock Prankster // Free the Fae', 'Thranduil, Sindarin Liege // Silvan Rally', 'Beluna Grandsquall // Seek Thrills', 'Glamdring, Foe-hammer // Gleam of Death']
class AdventureMilledSelectionCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.rows={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def admitted(self,c):
  return next(r['implemented'] for r in g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=hashlib.sha256(json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()),dict(banned_card_names=[]))[1]['cards'] if r['name']==c['name'])
 def test_complete_pairs_and_renamed_identities(self):
  for name in NAMES:
   self.assertTrue(self.admitted(self.rows[name]),name)
   c=copy.deepcopy(self.rows[name]);c['card_faces'][0]['name']='Fresh creature';c['card_faces'][1]['name']='Fresh adventure';c['name']='Fresh creature // Fresh adventure'
   self.assertTrue(self.admitted(c),name)
 def test_each_face_unknown_suffixes_and_parentheticals_fail_closed(self):
  for name in NAMES:
   for face in [0,1]:
    for suffix in [' Discover 3.',' (You win the game.)',' ((You win the game.))',' (Unknown reminder.)']:
     c=copy.deepcopy(self.rows[name]);c['card_faces'][face]['oracle_text']+=suffix
     self.assertFalse(self.admitted(c),(name,face,suffix))
 def test_filter_cardinality_destination_and_residuals_are_not_erased(self):
  for name in NAMES:
   for suffix in [' Draw fifty cards.',' You may cast any card from your graveyard.',' (Unknown reminder.)']:
    c=copy.deepcopy(self.rows[name]);c['card_faces'][1]['oracle_text']+=suffix;self.assertFalse(self.admitted(c))
  for name in NAMES:
   body=self.rows[name]['card_faces'][1]['oracle_text']
   for old,new in [('from among them','from your graveyard'),('from among the milled cards','from your graveyard'),('up to two','up to twenty'),('instant or sorcery','artifact or sorcery'),('Adventure','Omen')]:
    if old not in body:continue
    c=copy.deepcopy(self.rows[name]);c['card_faces'][1]['oracle_text']=body.replace(old,new);self.assertFalse(self.admitted(c),(name,old,new))
 def test_reducers_reject_unknown_relations_and_additional_qualifiers(self):
  for name in [NAMES[4],NAMES[5]]:
   for old,new in [('you cast','an opponent casts'),('less to cast.','less to cast. Discover 3.'),("equipped creature's power","equipped creature's toughness"),('that have an Adventure','that have an Omen')]:
    body=self.rows[name]['card_faces'][0]['oracle_text']
    if old not in body:continue
    c=copy.deepcopy(self.rows[name]);c['card_faces'][0]['oracle_text']=body.replace(old,new);self.assertFalse(self.admitted(c),(name,old,new))
if __name__=='__main__':unittest.main()
