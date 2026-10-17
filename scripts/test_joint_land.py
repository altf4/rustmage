"""Complete owned joint land selection and dynamically defined named tokens."""
import copy,hashlib,json,subprocess,unittest
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
from scripts.catalogue_preservation import canonical,assert_union
PARENT='dffafa6caaf7288ae8bbb8fa4c90b206e5960b94'
class JointLandTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  rust,cov=metadata.LegacyMetadataTests.compile([c]);return rust,next(v['implemented']for v in cov['cards']if v['name']==c['name'])
 def test_both_complete_bodies_are_semantic_and_name_independent(self):
  for original in ["Worldsoul's Rage",'Bonny Pall, Clearcutter']:
   base=self.cards[original];a,ok=self.compile(base);self.assertTrue(ok)
   for name in ['Independent Joint Land',*[c[0]for c in g.LEGACY]]:
    raw=base['oracle_text'].replace(original,name)
    if original.startswith('Bonny'):raw=raw.replace('Bonny Pall',name)
    c=dict(base,name=name,oracle_text=raw);rust,ok=self.compile(c);self.assertTrue(ok,(original,name))
    for tail in [' (You win the game.)',' ((You win the game.))','\n(You win the game.)']:
     self.assertFalse(self.compile(dict(c,oracle_text=raw+tail))[1],(original,name,tail))
    for bad in [raw.replace('graveyard','grave(You win the game.)yard'),raw.replace('from your hand','from(You win the game.) your hand')]:
     self.assertFalse(self.compile(dict(c,oracle_text=bad))[1],bad)
    self.assertFalse(self.compile(dict(c,mana_cost='{Q}'))[1]);self.assertFalse(self.compile(dict(c,type_line='Sorcery Creature'))[1])
   if original.startswith('World'):
    self.assertIn('DamageScaled(AmountValue::ChosenX)',a);self.assertIn('PutLandCards { maximum: AmountValue::ChosenX, tapped: true }',a)
   else:
    self.assertIn('DynamicStats',a);self.assertIn('legendary: true',a);self.assertIn('StatValue::Count(',a)
    self.assertIn('PutLandCards { maximum: AmountValue::Fixed(1), tapped: false }',a)
 def test_generic_new_routes_and_token_cda_authenticate_whole_raw(self):
  for origin in ['Instant','Sorcery']:
   c=dict(self.cards["Worldsoul's Rage"],name='Fresh Land Damage',type_line=origin,mana_cost='{X}{1}{U}',oracle_text='Fresh Land Damage deals X damage to any target. Put up to X land cards from your hand and/or graveyard onto the battlefield tapped.')
   self.assertTrue(self.compile(c)[1])
   for tail in [' Put a +1/+1 counter on it.',' You win the game.',' (This creature has flying.)']:
    self.assertFalse(self.compile(dict(c,oracle_text=c['oracle_text']+tail))[1])
  base=metadata.LegacyMetadataTests.creature('Fresh Factory')
  raw='When this creature enters, create Example, a legendary green Ox creature token with "Example\'s power and toughness are each equal to the number of lands you control."'
  self.assertTrue(self.compile(dict(base,oracle_text=raw))[1])
  for bad in [raw.replace("Example's power","Unrelated's power"),raw.replace('lands you control','imaginary things'),raw+' (You win the game.)',raw.replace('legendary','legen(You win the game.)dary'),raw.replace('control.','control. You win the game.')]:
   self.assertFalse(self.compile(dict(base,oracle_text=bad))[1],bad)
 def test_exact_parent_preservation_and_only_two_whole_admissions(self):
  physical={c['oracle_id']:c['name']for c in self.cards.values()}
  before=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True),physical)
  after=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show','f35e95302aad7924be38def41598dcf9013c0fb7:src/generated/catalogue.rs'],text=True),physical)
  current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  new=set(after['definitions'])-set(before['definitions']);self.assertEqual(len(new),1)
  key=next(iter(new));self.assertTrue(key.startswith('@token/'))
  new_recipes=set(after['tokens'])-set(before['tokens']);self.assertEqual(new_recipes,{key.removesuffix('#0')})
  self.assertIn('name: "Beau", subtypes: "Ox", colors: 2',after['tokens'][key.removesuffix('#0')])
  self.assertTrue(all(after['tokens'].get(k)==v for k,v in before['tokens'].items()))
  delivered_tokens=copy.deepcopy(after['tokens'])
  after=copy.deepcopy(after);after['tokens']=copy.deepcopy(before['tokens'])
  proof=assert_union(before,after,changed={"Worldsoul's Rage#0",'Bonny Pall, Clearcutter#0'},added=new,helpers={('dynamic_stats',key),('x_symbols',"Worldsoul's Rage#0")},aliases={key.removesuffix('#0'):key})
  self.assertEqual(proof['prior_helpers'],924)
  for name in ["Worldsoul's Rage#0",'Bonny Pall, Clearcutter#0',key]:self.assertEqual(current['definitions'][name],after['definitions'][name])
  for helper in [('dynamic_stats',key),('x_symbols',"Worldsoul's Rage#0")]:self.assertEqual(current['helpers'][helper],after['helpers'][helper])
  self.assertEqual(current['tokens'],delivered_tokens)
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed={"Worldsoul's Rage#0",'Bonny Pall, Clearcutter#0'},added=new,helpers={('dynamic_stats',key),('x_symbols',"Worldsoul's Rage#0")},aliases={key.removesuffix('#0'):key})
 def test_remaining_four_full_bodies_stay_gated(self):
  for name in ['Kellan, the Kid','Michelangelo, Improviser','Spelunking','The Gitrog, Ravenous Ride']:
   self.assertFalse(self.compile(self.cards[name])[1],name)
if __name__=='__main__':unittest.main()
