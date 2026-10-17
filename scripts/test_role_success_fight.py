"""Complete successful-Role creation/reflexive fight, raw closure and provenance."""
import copy, hashlib, json, os, re, subprocess, unittest
from pathlib import Path
from scripts import generate_catalogue as g
from scripts.test_legacy_metadata import LegacyMetadataTests
from scripts.catalogue_preservation import canonical, assert_union
PARENT='ce4b0c104a8e0dbc0a76753a03608eb7bc4618be'
NAME='Curse of the Werefox'
class RoleSuccessFightTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cards=json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'];cls.card=next(c for c in cls.cards if c['name']==NAME)
 def compile(self,c):
  rust,coverage=LegacyMetadataTests.compile([c]);return rust,next(r['implemented'] for r in coverage['cards'] if r['name']==c['name'])
 def test_complete_program_cost_type_and_every_role_are_name_independent(self):
  for kind in g.ROLE_BODIES:
   bare=f"Create a {kind} Role token attached to target creature you control. When you do, that creature fights up to one target creature you don't control."
   for name in ['Independent Role Fight',*[x[0] for x in g.LEGACY]]:
    for typ,cost in [('Sorcery','{2}{G}'),('Instant','{1}{U}{R}')]:
     c=dict(self.card,name=name,type_line=typ,mana_cost=cost,oracle_text=bare)
     rust,ok=self.compile(c);self.assertTrue(ok,(kind,name,typ,cost));self.assertIn('Instruction::CreateRoleReflexive',rust);self.assertIn('Instruction::ReflexiveFight',rust);self.assertIn('min: 0, max: 1',rust);self.assertIn('condition: TriggerCondition::Reflexive',rust)
     row=next(l for l in rust.splitlines() if f'name: {json.dumps(name)},' in l and 'Definition {' in l)
     self.assertIn(f'mana_cost: {json.dumps(cost)}',row);self.assertIn(f'type_line: {json.dumps(typ)}',row)
     self.assertIn('colored: [0, 0, 0, 0, 1, 0], generic: 2' if typ=='Sorcery' else 'colored: [0, 1, 0, 1, 0, 0], generic: 1',row)
 def test_combined_and_standalone_matching_reminders_have_identical_programs(self):
  for kind,body in g.ROLE_BODIES.items():
   bare=f"Create a {kind} Role token attached to target creature you control. When you do, that creature fights up to one target creature you don't control."
   expected,ok=self.compile(dict(self.card,oracle_text=bare));self.assertTrue(ok)
   fight='Creatures that fight each deal damage equal to their power to the other.'
   forms=[fight,body,'If you control another Role on it, put that one into the graveyard. '+body]
   forms += [x+' '+fight for x in forms[1:]]
   for reminder in forms:
    actual,ok=self.compile(dict(self.card,oracle_text=bare+' ('+reminder+')'));self.assertTrue(ok,(kind,reminder))
    def defs(s):return [re.sub(r'oracle_keywords: &\[[^]]*\]','oracle_keywords: &[]',l)for l in s.splitlines()if l.lstrip().startswith('Definition {')]
    self.assertEqual(defs(expected),defs(actual))
 def test_unknown_and_stolen_raw_clauses_remain_gated_in_every_metadata_identity(self):
  bare=g.role_fight_text(self.card['oracle_text'])[1]
  variants=[bare+' Discover 4.',bare+' (You win the game.)',bare+' ((You win the game.))',bare+'\n(Draw seven cards.)',bare+' (Damage and effects that say "destroy" don\'t destroy it.)']
  for i in range(1,len(bare)):
   if bare[i-1].isalpha() and bare[i].isalpha():variants.append(bare[:i]+'(You win the game.)'+bare[i:])
  for m in re.finditer(r'\s+',bare):variants.append(bare[:m.end()]+'((Draw seven cards.)) '+bare[m.end():])
  for raw in variants:
   self.assertFalse(self.compile(dict(self.card,name='Independent Raw Closure',oracle_text=raw))[1],raw)
  for name in [x[0]for x in g.LEGACY]:
   for raw in variants[:5]:self.assertFalse(self.compile(dict(self.card,name=name,oracle_text=raw))[1],(name,raw))
  for changed in [bare.replace('When you do','If you do'),bare.replace('that creature fights','another creature fights'),bare.replace('up to one','up to two'),bare.replace('you don\'t control','you control')]:self.assertFalse(self.compile(dict(self.card,oracle_text=changed))[1],changed)
 def test_malformed_cost_type_and_incomplete_programs_never_dispatch_reserved_names(self):
  for name in [x[0]for x in g.LEGACY]:
   for cost in ['{X}{G}','{Y}','{4294967296}','{G}{', '']:
    self.assertFalse(self.compile(dict(self.card,name=name,mana_cost=cost))[1],(name,cost))
   for typ in ['Creature','Artifact','Instant Sorcery','Legendary Instant','Unknown']:
    self.assertFalse(self.compile(dict(self.card,name=name,type_line=typ,power='2',toughness='2'))[1],(name,typ))
 def test_exact_parent_preservation_has_only_one_complete_admission(self):
  before=subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True);after=(g.ROOT/'src/generated/catalogue.rs').read_text();physical={c['oracle_id']:c['name']for c in self.cards};proof=assert_union(canonical(before,physical),canonical(after,physical),changed={NAME+'#0'},added=set(),helpers=set(),aliases={});self.assertEqual(proof['preserved_definitions'],5330)
 def test_full_metadata_and_unique_pinned_source_provenance(self):
  m=json.loads((g.ROOT/'docs/role-success-fight-provenance.json').read_text());self.assertEqual(m['card'],self.card)
  root=Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE','/home/altf4/Code/mage'))
  if not root.exists():self.skipTest('set RUSTMAGE_XMAGE_SOURCE to verify pinned Java provenance')
  self.assertEqual(len({s['java_source']for s in m['sources']}),len(m['sources']))
  for item in m['sources']:
   b=(root/item['java_source']).read_bytes();self.assertEqual(hashlib.sha256(b).hexdigest(),item['java_sha256']);self.assertEqual(b,subprocess.check_output(['git','-C',str(root),'show',m['java_revision']+':'+item['java_source']]))
if __name__=='__main__':unittest.main()
