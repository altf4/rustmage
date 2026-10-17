"""Complete Gift keyword/body contracts, generic raw closure and pinned inventory."""
import hashlib,json,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
from scripts.catalogue_preservation import canonical,assert_union
PARENT='8dd395286661963d1d1d98a2d384f2f5ab62d620'
# Explicit prototype inventory; expand only after the corresponding whole kernel exists.
NAMES={'Blooming Blast','Consumed by Greed','Crumb and Get It','Dewdrop Cure','Into the Flood Maw',"Long River's Pull",'Longstalk Brawl','Mind Spiral','Nocturnal Hunger','Peerless Recycling',"Sazacap's Brew",'Scrapshooter','Wear Down',"Bilbo's Gambit","Dawn's Truce",'Valley Rally','Wildfire Howl'}
class GiftTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards=json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']
 def implemented(self,c):
  _,coverage=metadata.LegacyMetadataTests.compile([c]);return next(r['implemented']for r in coverage['cards']if r['name']==c['name'])
 def test_complete_prototype_bodies_and_every_raw_boundary_are_name_independent(self):
  for card in self.cards:
   if card['name']not in NAMES:continue
   for name in ['Independent Gift Body','Forest',card['name']]:
    c=dict(card,name=name,oracle_text=card['oracle_text'].replace(card['name'],name));raw=c['oracle_text']
    self.assertTrue(self.implemented(c),(card['name'],name))
    for bad in [raw+' (You win the game.)',raw+'\nDiscover 4.',raw.replace('Gift ','Gi(You win the game.)ft ',1),raw.replace('Gift ','(Gift ',1)+')',raw.replace(' as you cast this spell.',' as you cast this spell. Draw a card.',1),raw.replace('\n','\n(You win the game.)\n',1),raw+' ((Draw a card.))',raw.replace('target','tar(Draw a card.)get',1)]:
     if bad!=raw:self.assertFalse(self.implemented(dict(c,oracle_text=bad)),(card['name'],name,bad))
 def test_current_candidate_changes_only_declared_complete_bodies(self):
  before=subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True)
  after=subprocess.check_output(['git','-C',str(g.ROOT),'show','02d83521879d80772374b1c50d66c6356ea7325e:src/generated/catalogue.rs'],text=True);physical={c['oracle_id']:c['name']for c in self.cards}
  a,b=canonical(before,physical),canonical(after,physical)
  allowed={('gift',n+'#0')for n in NAMES}|{('additional_cost',"Sazacap's Brew#0")}
  proof=assert_union(a,b,changed={n+'#0'for n in NAMES},added=set(),helpers=allowed,aliases={})
  self.assertEqual(proof['preserved_definitions'],5314)
  current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  for name in NAMES:self.assertEqual(current['definitions'][name+'#0'],b['definitions'][name+'#0'])
  for key in allowed:self.assertEqual(current['helpers'][key],b['helpers'][key])
  import copy
  mutant=copy.deepcopy(b);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(a,mutant,changed={n+'#0'for n in NAMES},added=set(),helpers=allowed,aliases={})
 def test_malformed_cost_layout_type_and_out_of_range_stats_never_dispatch(self):
  for c in self.cards:
   if c['name']not in NAMES:continue
   for name in ['Independent Gift Shape','Forest']:
    card=dict(c,name=name,oracle_text=c['oracle_text'].replace(c['name'],name))
    for cost in ['', '{X}{G}', '{Y}', '{4294967296}', '{G}{']:
     self.assertFalse(self.implemented(dict(card,mana_cost=cost)),(c['name'],name,cost))
    for typ in ['Unknown','Creature Instant','Artifact Sorcery']:
     self.assertFalse(self.implemented(dict(card,type_line=typ,power='2',toughness='2')),(c['name'],name,typ))
    for layout in ['unknown','adventure','saga']:
     self.assertFalse(self.implemented(dict(card,layout=layout)),(c['name'],name,layout))
    if c['name']=='Scrapshooter':
     for axis in ['power','toughness']:
      for value in ['2147483648','-1','*','unknown']:
       self.assertFalse(self.implemented(dict(card,**{axis:value})),(name,axis,value))
 def test_all23_full_java_bodies_and_direct_imports_are_pinned(self):
  manifest=json.loads((g.ROOT/'docs/gift-admissions.json').read_text());gifts={c['name']:c for c in self.cards if c.get('oracle_text','').startswith('Gift ')}
  self.assertEqual(len(gifts),23);self.assertEqual({c['name']for c in manifest['cards']},set(gifts))
  root=Path('/home/altf4/Code/mage');unique={}
  for card in manifest['cards']:
   self.assertEqual(card['facts'],{k:v for k,v in gifts[card['name']].items()if k in ['mana_cost','type_line','oracle_text','power','toughness','colors','keywords','layout']})
   for source in card['sources']:unique[source['java_source']]=source['java_sha256']
  unique.update({s['java_source']:s['java_sha256']for s in manifest['shared_sources']})
  for path,sha in unique.items():
   raw=(root/path).read_bytes();self.assertEqual(hashlib.sha256(raw).hexdigest(),sha)
   self.assertEqual(raw,subprocess.check_output(['git','-C',str(root),'show',manifest['java_revision']+':'+path]))
if __name__=='__main__':unittest.main()
