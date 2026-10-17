"""Whole Job Select bodies, exact quoted recipient events and closed residual gates."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('job_generator',Path(__file__).with_name('generate_catalogue.py'))
g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=["Astrologian's Planisphere","Bard's Bow","Black Mage's Rod","Dark Knight's Greatsword","Dragoon's Lance","Machinist's Arsenal","Monk's Fist","Ninja's Blades","Paladin's Arms","Red Mage's Rapier","Sage's Nouliths","Samurai's Katana","Summoner's Grimoire","Thief's Knife","Warrior's Sword","White Mage's Staff"]
class JobSelectCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.snapshot=json.loads((g.ROOT/'data/standard/cards.json').read_text());cls.facts={c['name']:c for c in cls.snapshot['cards']}
 def compile(self,cards):
  snapshot=dict(cards=cards,card_count=len(cards));snapshot['cards_sha256']=hashlib.sha256(json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
  return g.compile_catalogue(snapshot,dict(banned_card_names=[]))
 def rejected(self,cards):
  _,coverage=self.compile(cards);self.assertFalse(any(c['implemented'] for c in coverage['cards'] if c['name'] in NAMES))
 def test_sixteen_complete_bodies_and_immutable_provenance(self):
  for name in NAMES:
   result=g.job_select_template(self.facts[name]);self.assertIsNotNone(result,name)
   self.assertIn('JobSelect',result[0][2][0][1][0]);self.assertIn(result[1],g.JOB_TYPES)
  source,coverage=self.compile([self.facts[n] for n in NAMES]);self.assertEqual(sum(c['implemented'] for c in coverage['cards'] if c['name'] in NAMES),16)
  self.assertIn('Granted(&TriggerCondition::CardDrawn { relation: CastRelation::You, ordinal: Some(3) })',source)
  self.assertIn('DiscardManaValueLifeLoss',source);self.assertIn('PutHandCreature',source);self.assertIn('TargetPredicate::Attacking',source)
  self.assertIn('Book Equipment',source);self.assertIn('once_per_turn: true',source)
 def test_unknown_whole_clauses_and_unimplemented_granted_events_remain_gated(self):
  for suffix in ['\nDiscover 3.','\nEquipped creature has "Whenever this creature dies, draw a card."','\nYou may play an additional land on each of your turns.']:
   self.rejected([dict(self.facts[n],oracle_text=self.facts[n]['oracle_text']+suffix) for n in NAMES])
  card=self.facts["White Mage's Staff"]
  for event in ['dies','enters','becomes tapped']:
   self.assertIsNone(g.job_select_template(dict(card,oracle_text=card['oracle_text'].replace('this creature attacks','this creature '+event))))
  self.assertIsNone(g.job_select_template(dict(card,oracle_text=card['oracle_text'].replace('Job select (','Job select twice ('))))
  self.assertIsNone(g.job_select_template(dict(card,oracle_text=card['oracle_text']+'\n(Whenever you cast a spell, draw a card.)')))
 def test_printed_costs_subtypes_reminders_and_full_selector_composites_are_closed(self):
  for cost in ['{X}{R}','{Y}{R}','{U/P}','{4294967296}','{4294967295}{1}']:
   self.rejected([dict(self.facts[n],mana_cost=cost) for n in NAMES])
  for name in NAMES:
   card=self.facts[name];self.assertIsNone(g.job_select_template(dict(card,oracle_text=card['oracle_text'].replace('in addition to its other types','instead of its other types'))))
  declaration=next(line for line in self.facts['Appeal to Eirdu']['oracle_text'].splitlines() if line.startswith('Convoke'))
  for n in NAMES:
   mutant=dict(self.facts[n],oracle_text=declaration+'\n'+self.facts[n]['oracle_text']);self.assertTrue(g.extract_convoke(mutant)[1])
  self.rejected([dict(self.facts[n],oracle_text=declaration+'\n'+self.facts[n]['oracle_text']) for n in NAMES])
  for extra in ['Kicker {1}','Offspring {1}','Flashback {1}','Collect evidence 4','Cycling {1}']:
   self.rejected([dict(self.facts[n],oracle_text=self.facts[n]['oracle_text']+'\n'+extra) for n in NAMES])
 def test_mana_bounds_and_semantic_metadata_change_fingerprint(self):
  card=self.facts["Monk's Fist"]
  source,coverage=self.compile([dict(card,mana_cost='{4294967295}{W}')]);self.assertTrue(next(c for c in coverage['cards'] if c['name']==card['name'])['implemented'])
  first,_=self.compile([card]);changed,_=self.compile([dict(card,oracle_text=card['oracle_text'].replace('a Monk','a Wizard'))]);self.assertNotEqual(first.splitlines()[1],changed.splitlines()[1])
if __name__=='__main__':unittest.main()
