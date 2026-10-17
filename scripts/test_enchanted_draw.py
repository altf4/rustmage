"""Complete enchanted draw-step admission, authenticated raw text and immutable parent."""
import copy,hashlib,json,os,re,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
NAME="Lord Skitter's Blessing"
class EnchantedDrawCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.card=next(c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name']==NAME)
 def compiled(self,c):
  rust,cov=metadata.LegacyMetadataTests.compile([c]);return rust,next(r['implemented'] for r in cov['cards'] if r['name']==c['name'])
 def test_complete_body_and_every_renamed_fixture_are_identical(self):
  def body(rust,name):
   rows=[l for l in rust.splitlines() if l.lstrip().startswith('Definition {')]
   names=[json.loads(re.search(r'name: ("(?:[^"\\]|\\.)*")',r)[1]) for r in rows]
   row=next(r for r in rows if 'name: '+json.dumps(name)+',' in r)
   return re.sub(r'Card\((\d+)\)',lambda m:'Card('+json.dumps(names[int(m[1])])+')',row)
  before,ok=self.compiled(self.card);self.assertTrue(ok)
  original=body(before,NAME)
  for name,*_ in g.LEGACY:
   rust,ok=self.compiled(dict(self.card,name=name));self.assertTrue(ok,name)
   self.assertEqual(body(rust,name),original.replace(json.dumps(NAME),json.dumps(name)),name)
  for token in ['CreateRole','StepValue::EnchantedCreatures','TriggerStep::Draw','LoseLife(PlayerSet::Controller, 1)','Draw(PlayerSet::Controller, 1)']:self.assertIn(token,original)
 def test_all_raw_boundaries_and_new_generic_routes_fail_closed(self):
  for line in range(2):
   for suffix in [' Discover 4.',' (You win the game.)',' ((You win the game.))']:
    lines=self.card['oracle_text'].splitlines();lines[line]+=suffix
    self.assertFalse(self.compiled(dict(self.card,oracle_text='\n'.join(lines)))[1],(line,suffix))
  for old in ['Role token','enchanted creature','additional card']:
   for extra in ['(You win the game.)',' (You win the game.) ','((You win the game.))']:
    text=self.card['oracle_text'].replace(old,old.split()[0]+extra+' '+' '.join(old.split()[1:]))
    self.assertFalse(self.compiled(dict(self.card,oracle_text=text))[1],(old,extra))
  for text in ['At the beginning of your draw step, if you control an enchanted creature, draw a card.','At the beginning of your draw step, you lose 1 life and you draw an additional card.']:
   c=dict(self.card,oracle_text=text);self.assertTrue(self.compiled(c)[1])
   self.assertFalse(self.compiled(dict(c,oracle_text=text+' (You win the game.)'))[1])
  self.assertIsNone(g.statement_program('you lose 65536 life and you draw an additional card.'))
 def test_exact_parent_generated_code_changes_only_one_definition(self):
  old=subprocess.check_output(['git','-C',str(g.ROOT),'show','ce9a90a:src/generated/catalogue.rs'],text=True)
  new=(g.ROOT/'src/generated/catalogue.rs').read_text()
  rows=lambda s:[l for l in s.splitlines() if l.lstrip().startswith('Definition {')]
  a,b=rows(old),rows(new);self.assertEqual(len(a),5328);self.assertEqual(len(a),len(b))
  changed=[(x,y) for x,y in zip(a,b) if x!=y];self.assertEqual(len(changed),1)
  self.assertIn('name: '+json.dumps(NAME)+',',changed[0][0]);self.assertIn('implemented: false',changed[0][0]);self.assertIn('implemented: true',changed[0][1])
  def unrelated(s):return '\n'.join(l for l in s.splitlines() if not (l.startswith('pub const CATALOGUE_VERSION:') or 'pub const ALL:' in l or l in [v for pair in changed for v in pair]))
  self.assertEqual(unrelated(old),unrelated(new))
 def test_complete_pinned_record_and_every_direct_java_import(self):
  manifest=json.loads((g.ROOT/'docs/enchanted-draw-admissions.json').read_text());row=manifest['cards'][0]
  for key in ['name','oracle_id','mana_cost','type_line','oracle_text']:self.assertEqual(row[key],self.card[key])
  root=Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE','/home/altf4/Code/mage'))
  self.assertEqual(len(row['sources']),20)
  for record in row['sources']:
   blob=subprocess.check_output(['git','-C',str(root),'show',manifest['java_revision']+':'+record['java_source']]);self.assertEqual((root/record['java_source']).read_bytes(),blob);self.assertEqual(hashlib.sha256(blob).hexdigest(),record['java_sha256'])
if __name__=='__main__':unittest.main()
