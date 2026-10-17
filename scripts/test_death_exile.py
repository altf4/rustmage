"""Complete until-cleanup death-exile consumers, typed bindings and raw closure."""
import copy,hashlib,json,os,re,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
from scripts.catalogue_preservation import canonical,assert_union
PARENT='b39084973f5e4d1d347f99322080c315df2fc7bc'
class DeathExileTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
  cls.audit=json.loads((g.ROOT/'docs/death-exile-admissions.json').read_text())
 def compile(self,c):
  rust,coverage=metadata.LegacyMetadataTests.compile([c]);return rust,next(row['implemented'] for row in coverage['cards'] if row['name']==c['name'])
 def test_eleven_complete_programs_are_name_independent_and_cost_typed(self):
  for name in self.audit['proposed_complete_batch']:
   old=self.cards[name];baseline,ok=self.compile(old);self.assertTrue(ok,name)
   self.assertIn('Instruction::ExileIfDiesThisTurn',baseline)
   for identity in ['Independent Exact Damage',*[x[0]for x in g.LEGACY]]:
    c=dict(old,name=identity,oracle_text=old['oracle_text'].replace(name,identity));rust,ok=self.compile(c);self.assertTrue(ok,(name,identity))
    def program(s,key):return next(l for l in s.splitlines()if l.lstrip().startswith('Definition {') and f'name: {json.dumps(key)},' in l).replace(key,'Independent Exact Damage')
    self.assertEqual(program(baseline,name),program(rust,identity))
    for cost in ['{Y}','{4294967296}','{R}{']:
     self.assertFalse(self.compile(dict(c,mana_cost=cost))[1],(name,identity,cost))
    self.assertFalse(self.compile(dict(c,type_line='Unknown'))[1])
 def test_seven_incomplete_whole_bodies_stay_gated(self):
  for name in self.audit['remaining_full_body_dependencies']:
   self.assertFalse(self.compile(self.cards[name])[1],name)
   renamed=dict(self.cards[name],name='Independent Remaining',oracle_text=self.cards[name]['oracle_text'].replace(name,'Independent Remaining'))
   self.assertFalse(self.compile(renamed)[1],name)
 def test_exact_damage_subject_and_scaled_semantics(self):
  for name,fragment in [('Essence Burn','TargetPredicate::Colors(20)'),('Combustion Technique','StatValue::Fixed(2)'),('Pinecone Strike','TargetPredicate::Token'),("Narset's Rebuke",'Instruction::AddMana')]:
   rust,ok=self.compile(self.cards[name]);self.assertTrue(ok);self.assertIn(fragment,rust)
  rust,_=self.compile(self.cards['Pinecone Strike']);self.assertIn('Choose both',rust);self.assertIn('Instruction::SelectGroup(1)',rust)
  for text in ['This spell deals 3 damage to target player. If that creature would die this turn, exile it instead.','Return target creature to its owner\'s hand. If that creature would die this turn, exile it instead.','This spell deals 3 damage to target creature or planeswalker. If that creature would die this turn, exile it instead.','This spell deals 3 damage to target creature. If a permanent dealt damage by this spell would die this turn, exile it instead.']:
   self.assertFalse(self.compile(dict(self.cards['Feed the Flames'],oracle_text=text))[1],text)
 def test_all_new_generic_callers_authenticate_unerased_raw(self):
  base=self.cards['Feed the Flames']
  bodies=['This spell deals 3 damage to target creature. If that creature would die this turn, exile it instead.','This spell deals 5 damage to target black or green creature or planeswalker.','Destroy target artifact token.','This spell deals damage equal to 2 plus the number of Lesson cards in your graveyard to target creature.']
  for body in bodies:
   c=dict(base,name='Independent Generic Route',oracle_text=body);self.assertTrue(self.compile(c)[1],body)
   variants=[body+' (You win the game.)',body+' ((You win the game.))',body+'\n(Draw seven cards.)',body+' (This creature can attack and {T} as soon as it comes under your control.)']
   variants += [body[:i]+'(You win the game.)'+body[i:] for i in range(1,len(body)) if body[i-1].isalpha() and body[i].isalpha()]
   variants += [body[:m.end()]+'((Draw seven cards.)) '+body[m.end():]for m in re.finditer(r'\s+',body)]
   for raw in variants:
    self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],raw)
   for identity in [x[0]for x in g.LEGACY]:
    for raw in variants[:4]:self.assertFalse(self.compile(dict(c,name=identity,oracle_text=raw))[1],(identity,raw))
 def test_complete_modes_and_intrinsic_reminder_context(self):
  old=self.cards['Pinecone Strike']
  for raw in [old['oracle_text']+' (You win the game.)',old['oracle_text'].replace('Destroy target artifact token.','Destroy target artifact token. Discover 4.'),old['oracle_text'].replace('If that creature','(Draw seven cards.) If that creature')]:self.assertFalse(self.compile(dict(old,oracle_text=raw))[1],raw)
  body='When this creature enters, this creature deals 3 damage to target creature. If that creature would die this turn, exile it instead.'
  c=dict(self.cards['Bot Bashing Time'],type_line='Creature',power='2',toughness='2',oracle_text='Flying\n'+body)
  _,ok=self.compile(c);self.assertTrue(ok)
  rust,ok=self.compile(dict(c,oracle_text='Flying (This creature can\'t be blocked except by creatures with flying or reach.)\n'+body));self.assertTrue(ok)
  self.assertIn('Instruction::ExileIfDiesThisTurn',rust)
  self.assertFalse(self.compile(dict(c,oracle_text='Flying\n'+body+' (This creature can\'t be blocked except by creatures with flying or reach.)'))[1])
 def test_strict_current_parent_preservation(self):
  before=subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True)
  physical={c['oracle_id']:c['name']for c in self.cards.values()}
  proof=assert_union(canonical(before,physical),canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical),changed={name+'#0'for name in self.audit['proposed_complete_batch']},added=set(),helpers=set(),aliases={})
  self.assertEqual(proof['preserved_definitions'],5320)
 def test_entire_pinned_java_source_and_full_metadata_audit(self):
  root=Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE','/home/altf4/Code/mage'))
  if not root.exists():self.skipTest('set RUSTMAGE_XMAGE_SOURCE for complete pinned source audit')
  self.assertEqual(self.audit['source_records'],242);self.assertEqual(len(self.audit['unique_sources']),88)
  for row in self.audit['cards']:self.assertEqual(row['facts'],self.cards[row['name']])
  for row in self.audit['unique_sources']:
   blob=(root/row['java_source']).read_bytes();self.assertEqual(hashlib.sha256(blob).hexdigest(),row['java_sha256'])
   self.assertEqual(blob,subprocess.check_output(['git','-C',str(root),'show',self.audit['java_revision']+':'+row['java_source']]))
if __name__=='__main__':unittest.main()
