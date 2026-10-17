"""Strict whole-pair admission for reusable land allowances and target movement."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('adventure_land_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Beanstalk Wurm // Plant Beans', 'Beorn, Reluctant Host // Till and Tend', 'Bellowing Bruiser // Beat a Path', "Beluna's Gatekeeper // Entry Denied", 'Bofur, Reliable Guardian // Concerted Care', 'Ishgard, the Holy See // Faith & Grief', 'Velvetwing Butterflies // Gaze in Wonder', 'Woodland Acolyte // Mend the Wilds']
class AdventureLandTargetCompilerTests(unittest.TestCase):
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
 def test_target_bounds_and_relations_are_not_erased(self):
  replacements={NAMES[2]:[("Up to two","Up to twenty"),("can't block","can't attack")],NAMES[3]:[("3 or less","4 or less"),("don't control","control")],NAMES[4]:[("artifact or creature","artifact, creature, or land"),("you control","an opponent controls")],NAMES[5]:[("up to two","up to twenty"),("your graveyard","any graveyard")],NAMES[6]:[("one or two","zero or two")],NAMES[7]:[("your graveyard","a graveyard"),("permanent card","instant card")]}
  for name,changes in replacements.items():
   for old,new in changes:
    c=copy.deepcopy(self.rows[name]);c['card_faces'][1]['oracle_text']=c['card_faces'][1]['oracle_text'].replace(old,new)
    self.assertFalse(self.admitted(c),(name,old,new))
 def test_extra_land_duration_and_mandatory_cost_residuals_fail_closed(self):
  for name in NAMES[:2]:
   for body in ['You may play an additional land each turn.','You may play two additional lands this turn.','You may play an additional land this turn. Draw fifty cards.','You may play an additional land this turn. (You win the game.)']:
    c=copy.deepcopy(self.rows[name]);c['card_faces'][1]['oracle_text']=body;self.assertFalse(self.admitted(c))
 def test_top_library_program_uses_typed_graveyard_targets(self):
  for phrase,rule in [('permanent','PermanentGraveYou'),('instant or sorcery','InstantSorceryGraveYou')]:
   self.assertEqual(g.targeted_program('Put target '+phrase+' card from your graveyard on top of your library.'),(rule,['TopLibrary']))
if __name__=='__main__':unittest.main()
