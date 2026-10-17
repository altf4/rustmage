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
 def test_exact_merged_parent_union_preserves_every_unrelated_program_and_factory(self):
  import subprocess
  from catalogue_preservation import canonical, assert_union
  parent='ce9a90a1867ce741387d3ed0a1aac0aa1592ca08'
  old=subprocess.check_output(['git','-C',str(g.ROOT),'show',parent+':src/generated/catalogue.rs'],text=True)
  physical={c['oracle_id']:c['name'] for c in self.rows.values()}
  changed={name+'#0' for name in NAMES}
  insets={self.rows[name]['card_faces'][1]['name']+'#0' for name in NAMES}
  helpers={(fn,name+'#0') for fn in ['adventure_identity','adventure_spell'] for name in NAMES}
  helpers|={('adventure_parent',name) for name in insets}
  aliases={'@adventure/'+self.rows[name]['oracle_id']:self.rows[name]['card_faces'][1]['name']+'#0' for name in NAMES}
  aliases.update({self.rows[name]['card_faces'][1]['name']:self.rows[name]['card_faces'][1]['name']+'#0' for name in NAMES})
  aliases.update({self.rows[name]['card_faces'][0]['name']:name+'#0' for name in NAMES})
  before=canonical(old,physical);after=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  report=assert_union(before,after,changed=changed,added=insets,helpers=helpers,aliases=aliases)
  # An unrelated admission, changed old helper, factory or inline flag must fail.
  for section,key in [('definitions','Plains#0'),('helpers',next(iter(before['helpers']))),('tokens',next(iter(before['tokens']))),('flags',next(iter(before['flags'])))]:
   mutant=copy.deepcopy(after);mutant[section][key]='unauthorized change'
   with self.assertRaises(AssertionError):assert_union(before,mutant,changed=changed,added=insets,helpers=helpers,aliases=aliases)
  self.assertEqual(report['prior_definitions'],5328)
  self.assertEqual(report['current_definitions'],5331)
  self.assertEqual(report['token_recipes'],94)
  self.assertEqual(report['inline_flag_tables'],3)
 def normal_faces(self):
  result=[]
  for name in NAMES:
   c=copy.deepcopy(self.rows[name]);c.update(c['card_faces'][0]);c.pop('card_faces',None);c['layout']='normal';c['name']='Independent normal event '+str(len(result));result.append(c)
  second=copy.deepcopy(result[0]);second['name']='Independent second cast event';second['oracle_text']=second['oracle_text'].replace('first instant','second instant');result.append(second)
  return result
 def test_generic_normal_events_authenticate_raw_text_at_every_infix_boundary(self):
  for c in self.normal_faces():
   self.assertTrue(self.admitted(c),c['name'])
   words=c['oracle_text'].split()
   for index in range(len(words)+1):
    for injected in ['(You win the game.)','((You win the game.))']:
     for separator in [' ','\t','\n']:
      mutant=copy.deepcopy(c);mutant['oracle_text']=' '.join(words[:index])+separator+injected+separator+' '.join(words[index:])
      with self.subTest(name=c['name'],index=index,clause=injected,separator=separator):self.assertFalse(self.admitted(mutant),mutant['oracle_text'])
   for word in ['cast','attacks','damage']:
    if word in c['oracle_text']:
     mutant=copy.deepcopy(c);mutant['oracle_text']=mutant['oracle_text'].replace(word,word[:2]+'(You win the game.)'+word[2:],1);self.assertFalse(self.admitted(mutant))
 def test_event_route_allows_only_authenticated_intrinsic_and_food_reminders(self):
  for c in self.normal_faces():
   c['oracle_text']='Flying (This creature can\'t be blocked except by creatures with flying or reach.)\n'+c['oracle_text']
   self.assertTrue(self.admitted(c),c['name'])
   for changed in ['Flying (You win the game.)','Flying ((This creature can\'t be blocked except by creatures with flying or reach.))','Flying\n(This creature can\'t be blocked except by creatures with flying or reach.)']:
    mutant=copy.deepcopy(c);mutant['oracle_text']=changed+'\n'+c['oracle_text'].split('\n',1)[1];self.assertFalse(self.admitted(mutant))
  food=self.normal_faces()[2];reminder=food['oracle_text'][food['oracle_text'].index('('):]
  for changed in [food['oracle_text'].replace('Food token.','Clue token.'),food['oracle_text'].replace(' '+reminder,'\n'+reminder),food['oracle_text']+' '+reminder,food['oracle_text'].replace('gain 3 life','gain 30 life')]:
   mutant=copy.deepcopy(food);mutant['oracle_text']=changed;self.assertFalse(self.admitted(mutant),changed)
