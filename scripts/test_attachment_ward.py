"""Whole attachment Ward grants and typed enchant predicate boundaries."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('attachment_ward_generate',Path(__file__).with_name('generate_catalogue.py'))
g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Dusk Rose Reliquary','Trapped in the Screen','Dwarven Mattock',"Falcon's Wing Harness",'Lavaspur Boots','Hardlight Containment','Sheltered by Ghosts','Super Strength','Detention Chariot','Static Snare']
class AttachmentWardCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.snapshot=json.loads((g.ROOT/'data/standard/cards.json').read_text());cls.rows={c['name']:c for c in cls.snapshot['cards']}
 def admitted(self,card):
  snapshot=dict(cards=[card],card_count=1,cards_sha256=hashlib.sha256(json.dumps([card],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest())
  _,coverage=g.compile_catalogue(snapshot,dict(banned_card_names=[]))
  return next(c for c in coverage['cards'] if c['name']==card['name'])['implemented']
 def test_all_ten_complete_bodies_and_unknown_residuals(self):
  for n in NAMES:
   c=self.rows[n]
   with self.subTest(card=n):
    self.assertTrue(self.admitted(c));self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')))
 def test_unknown_parenthetical_residuals_and_mismatched_reminders_stay_closed(self):
  for n in NAMES:
   c=self.rows[n]
   for residual in [' (You win the game.)',' (Discover 2.)',' (Whenever it becomes targeted, draw a card.)',' (unclosed',')']:
    self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text']+residual)),(n,residual))
   self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text']+'\n(You win the game.)')),n)
   self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace('exile target','Exile target')+'\n(You win the game.)')),n)
   self.assertFalse(self.admitted(dict(c,oracle_text='\n'.join(' '+line for line in c['oracle_text'].splitlines())+'\n (You win the game.)')),n)
   changed=c['oracle_text'].replace('that player pays {1}.','that player pays {9}.').replace('that player pays {2}.','that player pays {9}.')
   if changed!=c['oracle_text']:self.assertFalse(self.admitted(dict(c,oracle_text=changed)),n)
  # Existing grammar consumers retain their audited Job select/equip reminders.
  self.assertTrue(self.admitted(self.rows["Paladin's Arms"]))
 def test_grants_are_per_instance_and_reject_unsupported_costs(self):
  k,triggers=g.attachment_grants('flying and ward {1} and ward {1}')
  self.assertEqual(k,1);self.assertEqual(len(triggers),2);self.assertEqual(triggers[0],triggers[1])
  for text in ['ward {X}','ward {4294967296}','ward {W/P}','ward {Y}','ward—pay 2 life','ward {1} until end of turn','ward {1} and draw a card','flying, nonsense and ward {1}']:
   self.assertIsNone(g.attachment_grants(text),text)
 def test_type_cost_and_enchant_scope_remain_closed(self):
  for n in NAMES:
   c=self.rows[n]
   for cost in ['{X}','{Y}','{W/P}','{4294967296}','{4294967295}{1}']:
    self.assertFalse(self.admitted(dict(c,mana_cost=cost)),(n,cost))
   self.assertFalse(self.admitted(dict(c,layout='transform')))
  hard=self.rows['Hardlight Containment']
  for clause in ['Enchant land','Enchant artifact an opponent controls','Enchant artifact with mana value 2 or less','Enchant artifact or creature you control']:
   self.assertFalse(self.admitted(dict(hard,oracle_text=hard['oracle_text'].replace('Enchant artifact you control',clause))))
  for line in ['Enchanted creature has ward {1}.','Enchanted creature gets +1/+1 and has ward {1}.','Enchanted permanent has flying.']:
   for text in ['Enchant artifact you control\n'+line,line+'\nEnchant artifact you control']:
    self.assertFalse(self.admitted(dict(hard,oracle_text=text)))
 def test_static_snare_unqualified_attacker_count_is_distinct_from_owned_counts(self):
  c,discount=g.extract_cost_reductions(self.rows['Static Snare'])
  self.assertEqual(len(discount),1);self.assertIn('StatValue::AllAttackingCreatures',discount[0])
  self.assertEqual(g.stat_value('attacking creatures'),'StatValue::AttackingCreatures')
  self.assertNotIn('costs',c['oracle_text'])
 def test_exact_targets_and_live_subtype_ir(self):
  for text,kinds in [('target artifact or creature an opponent controls',17),('target artifact, creature, or enchantment an opponent controls',49)]:
   rule,program=g.targeted_program('exile '+text+' until this enchantment leaves the battlefield.')
   self.assertIn('kinds: '+str(kinds),g.TARGET_RULES[rule]);self.assertIn('relation: 1',g.TARGET_RULES[rule]);self.assertEqual(program,['ExileUntilSourceLeaves'])
  rule,program=g.targeted_program('attach this Equipment to target Dwarf you control.')
  self.assertIn('Subtype("Dwarf")',g.TARGET_RULES[rule]);self.assertIn('kinds: 0',g.TARGET_RULES[rule]);self.assertEqual(program,['AttachSource'])
  result=g.aura_template(self.rows['Hardlight Containment']);self.assertEqual(result[-5:-3],(16,0))
  for n in ['Royal Treatment','Redtooth Genealogist','Charmed Clothier','Secret Invasion','Nowhere to Run','Titania, Rugged Rumbler']:
   self.assertFalse(self.admitted(self.rows[n]),n)
if __name__=='__main__':unittest.main()
