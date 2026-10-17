"""Complete temporary-grant/Suspect programs, strict raw contexts and provenance."""
import copy,hashlib,json,re,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
NAMES={"Galuf's Final Act",'Rabid Attack','Perigee Beckoner','Presumed Dead','Person of Interest',"It Doesn't Add Up",'Absolving Lammasu','Reasonable Doubt','Convenient Target'}
class TemporaryDeathConsumerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  b=json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode();r,v=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=hashlib.sha256(b).hexdigest()),dict(banned_card_names=[]));return r,next(x['implemented'] for x in v['cards'] if x['name']==c['name'])
 def test_whole_bodies_generic_renamed_and_reserved_names(self):
  for name in NAMES:
   c=self.cards[name];self.assertTrue(self.compile(c)[1],name)
   for alias in ['Independent Suspect Subject','Lightning Bolt','Grizzly Bears']:
    d=dict(c,name=alias,oracle_text=c['oracle_text'].replace(name,alias));self.assertTrue(self.compile(d)[1],(name,alias))
 def test_original_raw_not_erased_or_stolen_contexts(self):
  for name in NAMES:
   c=self.cards[name]
   for alias in [name,'Independent Suspect Subject','Lightning Bolt']:
    raw=c['oracle_text'].replace(name,alias);d=dict(c,name=alias)
    words=re.findall(r'[A-Za-z]{4,}',raw)
    cases=[raw+' (You win the game.)',raw+'\n((Draw seven cards.))',raw+'\nYou win the game.',raw+'\n(It has menace and can\'t block.)']
    for word in words[:8]:cases += [raw.replace(word,word[:2]+'(You win the game.)'+word[2:],1),raw.replace(word,word[:2]+'((Draw seven cards.))'+word[2:],1)]
    for text in cases:self.assertFalse(self.compile(dict(d,oracle_text=text))[1],(name,alias,text))
   for bad in ['{X}{B}','{1}{Q}','{999999999999999999999}{B}']:
    self.assertFalse(self.compile(dict(c,mana_cost=bad))[1],(name,bad))
 def test_no_partial_suspect_or_tiered_admission(self):
  m=json.loads((g.ROOT/'docs/temporary-death-consumers.json').read_text())
  for row in m['cards']:
   if row['name'] not in NAMES:self.assertFalse(self.compile(self.cards[row['name']])[1],row['name'])
 def test_historical_nine_delivery_and_current_owned_programs(self):
  from scripts.catalogue_preservation import canonical,assert_union
  physical={c['oracle_id']:c['name'] for c in self.cards.values()}
  load=lambda ref:canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',ref+':src/generated/catalogue.rs'],text=True),physical)
  prior=load('5d103e7a3172e4742687d3f77f3ed814264b1532');delivered=load('471a7169e6ab1e00d6eeb35934ae4343a3bd6c67')
  token='@token/f144ea316833bbffad6fa831';helpers={('warp_cost','Perigee Beckoner#0'),('aura','Convenient Target#0')}
  self.assertEqual({k:v for k,v in delivered['tokens'].items() if k not in prior['tokens']},{token:'TokenInfo { factory_identity: "@token/f144ea316833bbffad6fa831", name: "Detective Token", subtypes: "Detective", colors: 3 },'})
  self.assertTrue(all(delivered['tokens'].get(k)==v for k,v in prior['tokens'].items()))
  checked=copy.deepcopy(delivered);checked['tokens']=prior['tokens']
  options=dict(changed={n+'#0' for n in NAMES},added={token+'#0'},helpers=helpers,aliases={token:token+'#0'})
  proof=assert_union(prior,checked,**options);self.assertEqual(proof['preserved_definitions'],5322)
  current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  for key in {n+'#0' for n in NAMES}|{token+'#0'}:
   self.assertEqual(current['definitions'][key],delivered['definitions'][key]);self.assertEqual(current['colors'][key],delivered['colors'][key])
  for key in helpers:self.assertEqual(current['helpers'][key],delivered['helpers'][key])
  self.assertEqual(current['tokens'][token],delivered['tokens'][token]);self.assertEqual(current['aliases'][token],token+'#0')
  mutant=copy.deepcopy(checked);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(prior,mutant,**options)
 def test_full_inventory_source_pin_and_actual_card_facts(self):
  m=json.loads((g.ROOT/'docs/temporary-death-consumers.json').read_text());self.assertEqual(len(m['cards']),24);self.assertEqual(m['java_revision'],'492369c7844b8d76479f0393ff9eb6b0b0cf30d7')
  seen=set()
  for row in m['cards']:
   for key in ['name','oracle_id','mana_cost','type_line','oracle_text']:self.assertEqual(row[key],self.cards[row['name']][key])
   for source in row['sources']:
    path=source['java_source']
    if path in seen:continue
    b=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',m['java_revision']+':'+path]);self.assertEqual(hashlib.sha256(b).hexdigest(),source['java_sha256']);seen.add(path)
  self.assertEqual(len(seen),140)
if __name__=='__main__':unittest.main()
