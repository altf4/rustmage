"""Strict whole-pair admission for historical cast and declaration events."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('adventure_land_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Aquatic Alchemist // Bubble Up','Scalding Viper // Steam Clean','Intrepid Trufflesnout // Go Hog Wild']
class AdventureEventCompilerTests(unittest.TestCase):
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
 def test_qualified_events_reject_unknown_and_inverted_clauses(self):
  controls=[(0,'first instant or sorcery spell','first artifact spell'),(0,'first instant or sorcery spell','first instant or sorcery permanent'),(1,'an opponent casts','another opponent casts'),(1,'that player','another player'),(2,'attacks alone','attacks alone or with a friend')]
  for i,old,new in controls:
   c=copy.deepcopy(self.rows[NAMES[i]]);c['card_faces'][0]['oracle_text']=c['card_faces'][0]['oracle_text'].replace(old,new);self.assertFalse(self.admitted(c),(i,old,new))
 def test_event_predicates_bind_typed_history_caster_and_declaration(self):
  for i,expected in enumerate(['InstantSorceryOrdinal(1)','CastingPlayer','SelfAttacksAlone']):
   c=self.rows[NAMES[i]];report=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=hashlib.sha256(json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()),dict(banned_card_names=[]));self.assertIn(expected,report[0])
