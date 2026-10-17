"""Complete semantic Role consumers, predefined factories and raw erasure boundaries."""
import copy,json,re,unittest
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
NAMES=['Besotted Knight // Betroth the Beast','Conceited Witch // Price of Beauty','Ferocious Werefox // Guard Change','Vantress Transmuter // Croaking Curse','Charmed Clothier','Charming Scoundrel','Embereth Veteran','Redtooth Genealogist','Living Lectern','Royal Treatment','Spiteful Hexmage','Splashy Spellcaster']
class RoleCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def test_twelve_whole_consumers_six_factories_and_complete_renamed_programs(self):
  for name in NAMES:
   c=self.cards[name];before,cov=metadata.LegacyMetadataTests.compile([c]);self.assertTrue(next(r for r in cov['cards'] if r['name']==name)['implemented'],name)
   new='Independent Role Consumer';d=copy.deepcopy(c); replacements={name:new}
   if d.get('card_faces'):
    for i,f in enumerate(d['card_faces']):
     old=f['name'];f['name']=new+(' Face' if i==0 else ' Inset');replacements[old]=f['name']
    new=' // '.join(f['name'] for f in d['card_faces']);replacements[name]=new
   d['name']=new
   after,cov=metadata.LegacyMetadataTests.compile([d]);self.assertTrue(next(r for r in cov['cards'] if r['name']==new)['implemented'],name)
   semantics=lambda t: sorted(l for l in t.splitlines() if not l.startswith('pub const CATALOGUE_VERSION:'))
   for old,renamed in sorted(replacements.items(),key=lambda item:-len(item[0])):before=before.replace(json.dumps(old),json.dumps(renamed))
   self.assertEqual(semantics(before),semantics(after))
  _,cov=metadata.LegacyMetadataTests.compile([self.cards[n] for n in NAMES]);self.assertEqual(cov['token_definitions'],7) # Six Roles plus Scoundrel's Treasure.
 def test_every_role_word_boundary_rejects_erased_unknown_raw_prose(self):
  mutants=[]
  for name in NAMES:
   base=self.cards[name]
   fields=[None] if not base.get('card_faces') else [0,1]
   for face in fields:
    text=base.get('oracle_text','') if face is None else base['card_faces'][face].get('oracle_text','')
    for tail in ['\nDiscover 4.',' (Draw seven cards.)',' ((Draw seven cards.))']:
     d=copy.deepcopy(base)
     if face is None:d['oracle_text']=text+tail
     else:d['card_faces'][face]['oracle_text']=text+tail
     d['oracle_id']='role-mutation-'+str(len(mutants));mutants.append(d)
    for m in re.finditer(r'\bRole token\b',text):
     for insertion in ['(Draw seven cards.)',' (Draw seven cards.) ','((Draw seven cards.))','\t(Draw seven cards.)\t']:
      d=copy.deepcopy(base);changed=text[:m.start()+4]+insertion+text[m.start()+4:]
      if face is None:d['oracle_text']=changed
      else:d['card_faces'][face]['oracle_text']=changed
      d['oracle_id']='role-mutation-'+str(len(mutants));mutants.append(d)
  for c in mutants:
   _,cov=metadata.LegacyMetadataTests.compile([c]);self.assertFalse(next(r for r in cov['cards'] if r['name']==c['name'])['implemented'],c)
 def test_role_selection_is_semantic_and_all_residuals_remain_closed(self):
  for kind in g.ROLE_BODIES:
   rule,program=g.targeted_program('Create a '+kind+' Role token attached to target creature you control.')
   self.assertEqual(rule,'CreatureYou');self.assertIn('CreateRole',program[0])
   for tail in [' Discover 4.',' Take an extra turn after this one.',' You win the game.']:
    self.assertIsNone(g.targeted_program('Create a '+kind+' Role token attached to target creature you control.'+tail))
   complete=g.targeted_program('Create a '+kind+' Role token attached to target creature you control. Draw a card.')
   self.assertEqual(complete[0],'CreatureYou');self.assertEqual(complete[1][-1],'Draw(PlayerSet::Controller, 1)')
 def test_full_role_inventory_matches_snapshot_and_pinned_source_blobs(self):
  import hashlib,os,subprocess
  from pathlib import Path
  manifest=json.loads((g.ROOT/'docs/role-admissions.json').read_text())
  expected={n for n,c in self.cards.items() if 'Role token' in c.get('oracle_text','')+' '.join(f.get('oracle_text','') for f in c.get('card_faces',[]))}
  self.assertEqual({r['name'] for r in manifest['cards']},expected)
  self.assertEqual({r['name'] for r in manifest['cards'] if r['status']=='implemented'},set(NAMES))
  source=Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE','/home/altf4/Code/mage'))
  records=manifest['shared_sources']+[v for c in manifest['cards'] for v in c['sources']]
  for record in {v['java_source']:v for v in records}.values():
   blob=subprocess.check_output(['git','-C',str(source),'show',manifest['java_revision']+':'+record['java_source']])
   self.assertEqual(hashlib.sha256(blob).hexdigest(),record['java_sha256'])
   self.assertEqual((source/record['java_source']).read_bytes(),blob)
if __name__=='__main__':unittest.main()
