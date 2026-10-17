"""Whole prospective entry replacements and result-bound land placement."""
import copy,json,subprocess,unittest
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
from scripts.catalogue_preservation import canonical,assert_union
PARENT='2812c749b63b38d10572bf96baacd527464852de'
class UntappedEntryTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  rust,cov=metadata.LegacyMetadataTests.compile([c]);return rust,next(v['implemented']for v in cov['cards']if v['name']==c['name'])
 def test_both_whole_bodies_costs_types_names_and_raw(self):
  for origin in ['Spelunking','The Wandering Minstrel']:
   base=self.cards[origin]
   rust,ok=self.compile(base);self.assertTrue(ok,origin)
   self.assertIn('controlled_entry_untapped',rust)
   if origin=='Spelunking':self.assertIn('PutHandLandWithLife { tapped: false, subtype: "Cave", life: 4 }',rust)
   else:
    self.assertIn('minimum: Some(5)',rust);self.assertIn('GrowCreaturesScaled',rust);self.assertIn('colors: 31',rust)
   for name in ['Independent Entering Land',*[c[0]for c in g.LEGACY]]:
    c=dict(base,name=name);self.assertTrue(self.compile(c)[1],(origin,name))
    raw=c['oracle_text']
    for tail in [' (You win the game.)',' ((You win the game.))','\n(You win the game.)',' You win the game.']:
     self.assertFalse(self.compile(dict(c,oracle_text=raw+tail))[1],(origin,name,tail))
    for bad in [raw.replace('untapped','un(You win the game.)tapped'),raw.replace('Lands you control','Lands you (You win the game.)control')]:
     self.assertFalse(self.compile(dict(c,oracle_text=bad))[1],bad)
    for cost in ['{Q}','{3}{','{X}{G}']:
     self.assertFalse(self.compile(dict(c,mana_cost=cost))[1],(name,cost))
    self.assertFalse(self.compile(dict(c,type_line='Sorcery Creature'))[1])
 def test_whole_draw_then_land_unsupported_tail_is_gated_not_an_exception(self):
  base=self.cards['Spelunking']
  for bad in [base['oracle_text'].replace('a Cave','a Imaginary'),base['oracle_text'].replace('gain 4 life','gain 65536 life')]:
   for name in ['Unsupported Draw Tail',*[c[0]for c in g.LEGACY]]:
    self.assertFalse(self.compile(dict(base,name=name,oracle_text=bad))[1],(name,bad))
 def test_each_new_generic_production_authenticates_original_raw(self):
  base=metadata.LegacyMetadataTests.creature('Generic Replacement')
  routes=[
   'Lands you control enter untapped.',
   'When this creature enters, you may put a land card from your hand onto the battlefield. If you put a Forest onto the battlefield this way, you gain 2 life.',
   "When this creature enters, create a 3/4 Elemental creature token that's all colors.",
   '{2}{G}: Other creatures you control get +X/+X until end of turn, where X is the number of Towns you control.',
  ]
  for raw in routes:
   for name in ['Fresh Generic',*[c[0]for c in g.LEGACY]]:
    c=dict(base,name=name,oracle_text=raw);self.assertTrue(self.compile(c)[1],(name,raw))
    for tail in [' (You win the game.)',' ((You win the game.))',' (You gain 2 life.)',' (A token is a permanent.)']:
     self.assertFalse(self.compile(dict(c,oracle_text=raw+tail))[1],(name,raw,tail))
    for word in ['enter','creature','control']:
     if word in raw:self.assertFalse(self.compile(dict(c,oracle_text=raw.replace(word,word[:2]+'(You win the game.)'+word[2:],1)))[1],(name,word))
  for bad in [routes[1].replace('Forest','Imaginary'),routes[1].replace('this way','this or another way'),routes[2].replace('all colors','any color'),routes[3].replace('Towns you control','Towns everyone controls')]:
   self.assertFalse(self.compile(dict(base,oracle_text=bad))[1],bad)
 def test_exact_actual_parent_preservation(self):
  physical={c['oracle_id']:c['name']for c in self.cards.values()}
  before=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True),physical)
  after=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  added=set(after['definitions'])-set(before['definitions']);self.assertEqual(len(added),1)
  token=next(iter(added));recipe=token.removesuffix('#0')
  self.assertEqual(set(after['tokens'])-set(before['tokens']),{recipe})
  self.assertIn('name: "Elemental Token", subtypes: "Elemental", colors: 31',after['tokens'][recipe])
  self.assertTrue(all(after['tokens'].get(k)==v for k,v in before['tokens'].items()))
  after=copy.deepcopy(after);after['tokens']=copy.deepcopy(before['tokens'])
  allowed=dict(changed={'Spelunking#0','The Wandering Minstrel#0'},added=added,helpers={('controlled_entry_untapped','Spelunking#0'),('controlled_entry_untapped','The Wandering Minstrel#0')},aliases={recipe:token})
  proof=assert_union(before,after,**allowed);self.assertEqual(proof['prior_helpers'],928)
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,**allowed)
 def test_pinned_full_java_audit(self):
  audit=json.loads((g.ROOT/'docs/untapped-entry-admissions.json').read_text());self.assertEqual(audit['source_records'],88);self.assertEqual(audit['unique_sources'],52)
  self.assertEqual([c['name']for c in audit['cards']if c['current_standard']],['Spelunking','The Wandering Minstrel'])
  import hashlib
  for record in [s for c in audit['cards']for s in c['sources']]+audit['additional_sources']:
   raw=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',audit['java_revision']+':'+record['java_source']]);self.assertEqual(hashlib.sha256(raw).hexdigest(),record['java_sha256'])
if __name__=='__main__':unittest.main()
