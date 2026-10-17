"""Complete private hand-land consumers, original-raw closure and source provenance."""
import hashlib,json,re,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
from scripts.catalogue_preservation import canonical,assert_union
PARENT='19e4602b436b7aa8c9708f28f87fe288985c9377'
class HandLandTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
  cls.audit=json.loads((g.ROOT/'docs/hand-land-admissions.json').read_text())
 def compile(self,c):
  rust,cov=metadata.LegacyMetadataTests.compile([c]);return rust,next(x['implemented']for x in cov['cards']if x['name']==c['name'])
 def test_five_complete_bodies_are_name_independent_with_typed_costs(self):
  for n in self.audit['proposed_complete']:
   c=self.cards[n];r,ok=self.compile(c);self.assertTrue(ok,n);self.assertIn('Instruction::PutHandLand { tapped: true }',r)
   for name in ['Independent Private Entry',*[x[0]for x in g.LEGACY]]:
    raw=c['oracle_text'].replace(n,name)
    if c['type_line'].startswith('Legendary '):raw=raw.replace(n.split(',')[0],name)
    d=dict(c,name=name,oracle_text=raw);self.assertTrue(self.compile(d)[1],(n,name))
    for cost in ['{Y}','{R}{','{4294967296}']:
     self.assertFalse(self.compile(dict(d,mana_cost=cost))[1],(n,name,cost))
    self.assertFalse(self.compile(dict(d,type_line='Unknown'))[1])
 def test_three_remaining_and_later_complete_bodies_keep_full_body_guards(self):
  for n in ['PuPu UFO',"Worldsoul's Rage",'Bonny Pall, Clearcutter','Spelunking']:
   c=self.cards[n];self.assertTrue(self.compile(c)[1]);self.assertFalse(self.compile(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.'))[1])
  for n in self.audit['remaining_dependencies']:
   c=self.cards[n];self.assertFalse(self.compile(c)[1],n)
   self.assertFalse(self.compile(dict(c,name='Independent Remaining',oracle_text=c['oracle_text'].replace(n,'Independent Remaining')))[1],n)
 def test_all_new_generic_routes_authenticate_original_raw_and_context(self):
  base=self.cards['H.E.R.B.I.E. Scout Unit']
  bodies=['You may put a land card from your hand onto the battlefield.','When this creature enters, draw a card, then you may put a land card from your hand onto the battlefield tapped.','Whenever a creature you control explores a nonland card, draw a card.','Whenever a creature you control explores a land card, you gain 1 life.','{2}: Choose one —\n• Draw a card.\n• You gain 1 life.']
  for body in bodies:
   c=dict(base,name='Independent Generic Entry',oracle_text=body)
   if body.startswith('You may'):c=dict(c,type_line='Instant')
   self.assertTrue(self.compile(c)[1],body)
   variants=[body+' (You win the game.)',body+' ((Draw seven cards.))',body+'\n(Draw seven cards.)',body+'\n(This creature can attack and {T} as soon as it comes under your control.)']
   variants += [body[:i]+'(You win the game.)'+body[i:]for i in range(1,len(body))if body[i-1].isalpha()and body[i].isalpha()]
   for text in variants:self.assertFalse(self.compile(dict(c,oracle_text=text))[1],text)
   for name in [x[0]for x in g.LEGACY]:
    for text in variants[:4]:self.assertFalse(self.compile(dict(c,name=name,oracle_text=text))[1],(name,text))
  valid='Flying (This creature can\'t be blocked except by creatures with flying or reach.)\n'+bodies[1]
  self.assertTrue(self.compile(dict(base,oracle_text=valid))[1])
 def test_modal_activation_is_single_ability_complete_and_not_a_modal_mana_shortcut(self):
  c=self.cards['Insidious Fungus'];r,ok=self.compile(c);self.assertTrue(ok)
  line=next(x for x in r.splitlines()if 'name: "Insidious Fungus"'in x and x.lstrip().startswith('Definition {'))
  self.assertEqual(line.count('ActivatedAbility {'),1);self.assertIn('Instruction::Modal(',line);self.assertEqual(line.count('SpellMode {'),3)
  for text in [c['oracle_text']+'\n• Discover 4.',c['oracle_text'].replace('• Destroy target enchantment.','• You win the game.'),'{2}: Choose one —\n• Add {G}.\n• Draw a card.','{2}, Discard a card: Choose one —\n• Draw a card.\n• You gain 1 life.','{2}, Sacrifice a creature: Choose one —\n• Destroy target artifact.\n• You gain 1 life.']:
   self.assertFalse(self.compile(dict(c,oracle_text=text))[1],text)
 def test_internal_modal_lowering_cannot_be_forged_by_printed_text(self):
  base=self.cards['Insidious Fungus']
  modal='{2}: Choose one —\n• Draw a card.\n• You gain 1 life.'
  for name in ['Independent Forged Mode',*[x[0]for x in g.LEGACY]]:
   c=dict(base,name=name,oracle_text=modal)
   self.assertTrue(self.compile(c)[1],name)
   for marker in ['__rustmage_modal_activation_0__','__rustmage_modal_activation_1__','Choose one —']:
    fake='{2}: '+marker
    for text in [fake+'\n'+modal,modal+'\n'+fake]:
     self.assertFalse(self.compile(dict(c,oracle_text=text))[1],(name,text))
   two=modal+'\n{3}: Choose one —\n• You gain 2 life.\n• Draw a card.'
   rust,ok=self.compile(dict(c,oracle_text=two));self.assertTrue(ok,name)
   line=next(x for x in rust.splitlines()if 'name: '+json.dumps(name)in x and x.lstrip().startswith('Definition {'))
   self.assertEqual(line.count('ActivatedAbility {'),2)
   self.assertEqual(line.count('Instruction::Modal('),2)
 def test_trigger_modal_semantics_are_owned_by_parsed_header_only(self):
  base=self.cards['Insidious Fungus']
  modal='When this creature enters, choose one —\n• Draw a card.\n• You gain 1 life.'
  for name in ['Independent Trigger',*[x[0]for x in g.LEGACY]]:
   c=dict(base,name=name,oracle_text=modal)
   self.assertTrue(self.compile(c)[1],name)
   for marker in ['__rustmage_modal_trigger_0__','__rustmage_modal_trigger_1__','__rustmage_modal_activation_0__']:
    for text in [marker+'\n'+modal,modal+'\n'+marker]:
     self.assertFalse(self.compile(dict(c,oracle_text=text))[1],(name,text))
   for prefix in ['Flying\n','{2}: You gain 1 life.\n']:
    self.assertTrue(self.compile(dict(c,oracle_text=prefix+modal))[1],(name,prefix))
 def test_explored_kinds_are_exact_and_empty_has_no_card_kind(self):
  r,ok=self.compile(self.cards['Nicanzil, Current Conductor']);self.assertTrue(ok);self.assertIn('CreatureExploresCardYou { land: true }',r);self.assertIn('CreatureExploresCardYou { land: false }',r)
  for phrase in ['an artifact card','a creature card','a card','another land card']:
   c=self.cards['Nicanzil, Current Conductor'];self.assertFalse(self.compile(dict(c,oracle_text=c['oracle_text'].replace('a land card,',phrase+',')))[1],phrase)
 def test_exact_current_parent_preservation(self):
  before=subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True);delivered=subprocess.check_output(['git','-C',str(g.ROOT),'show','a9380ec478646c42308c7284de197fa466db24d2:src/generated/catalogue.rs'],text=True);physical={c['oracle_id']:c['name']for c in self.cards.values()}
  old=canonical(before,physical);final=canonical(delivered,physical);current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  proof=assert_union(old,final,changed={n+'#0'for n in self.audit['proposed_complete']},added=set(),helpers=set(),aliases={});self.assertEqual(proof['preserved_definitions'],5327);self.assertEqual(proof['prior_helpers'],911)
  for name in self.audit['proposed_complete']:self.assertEqual(current['definitions'][name+'#0'],final['definitions'][name+'#0'])
  for key,value in final['helpers'].items():self.assertEqual(current['helpers'][key],value)
  import copy
  mutant=copy.deepcopy(final);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(old,mutant,changed={n+'#0'for n in self.audit['proposed_complete']},added=set(),helpers=set(),aliases={})
 def test_complete_twelve_body_metadata_and_all_pinned_java_files(self):
  self.assertEqual(len(self.audit['cards']),12);self.assertEqual(self.audit['source_records'],187);self.assertEqual(self.audit['unique_source_count'],86)
  root=Path('/home/altf4/Code/mage')
  for row in self.audit['cards']:self.assertEqual(row['facts'],self.cards[row['name']])
  for row in self.audit['unique_sources']:
   b=(root/row['java_source']).read_bytes();self.assertEqual(hashlib.sha256(b).hexdigest(),row['java_sha256']);self.assertEqual(b,subprocess.check_output(['git','-C',str(root),'show',self.audit['java_revision']+':'+row['java_source']]))
if __name__=='__main__':unittest.main()
