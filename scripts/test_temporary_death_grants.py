"""Complete temporary-grant inventory and pinned Java facts; no admission whitelist."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class TemporaryDeathGrantProvenanceTests(unittest.TestCase):
 def test_complete_eight_card_inventory_and_pinned_java_class_references(self):
  manifest=json.loads((ROOT/'docs/temporary-death-admissions.json').read_text());cards=json.loads((ROOT/'data/standard/cards.json').read_text())['cards']
  expected={c['name']:c for c in cards if 'dies' in c.get('oracle_text','') and re.search(r'gains? "When',c.get('oracle_text',''))}
  self.assertEqual(len(expected),8);self.assertEqual({r['name'] for r in manifest['cards']},set(expected));self.assertEqual(set(manifest['initial_consumers']),{'Not Dead After All','Fake Your Own Death','Undying Malice'})
  java=Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE','/home/altf4/Code/mage'));seen=set()
  for row in manifest['cards']:
   for key in ['name','oracle_id','mana_cost','type_line','oracle_text']:self.assertEqual(row[key],expected[row['name']][key])
   for source in row['sources']:
    path=source['java_source']
    if path in seen:continue
    blob=subprocess.check_output(['git','-C',str(java),'show',manifest['java_revision']+':'+path]);self.assertEqual((java/path).read_bytes(),blob);self.assertEqual(hashlib.sha256(blob).hexdigest(),source['java_sha256']);seen.add(path)
  self.assertGreater(len(seen),30)
class TemporaryDeathGrantCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from scripts import generate_catalogue as g
  cls.g=g;cls.cards={c['name']:c for c in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  from scripts.test_legacy_metadata import LegacyMetadataTests
  text,cov=LegacyMetadataTests.compile([c]);return text,next(r for r in cov['cards'] if r['name']==c['name'])
 def test_three_complete_bodies_and_renamed_generic_programs(self):
  for name in ['Not Dead After All','Fake Your Own Death','Undying Malice']:
   c=self.cards[name];text,row=self.compile(c);self.assertTrue(row['implemented'])
   for fragment in ['Instruction::GrantUntilEndTurn { ability: 0 }','TriggerCondition::Granted(&TriggerCondition::SelfDies)','Instruction::ReturnGrantedSource']:
    self.assertIn(fragment,text)
   renamed=dict(c,name='Independent temporary grant')
   other,row=self.compile(renamed);self.assertTrue(row['implemented'])
   norm=lambda t:sorted(l for l in t.splitlines() if not l.startswith('pub const CATALOGUE_VERSION:'))
   self.assertEqual(norm(text.replace(json.dumps(name),json.dumps(renamed['name']))),norm(other))
   if name=='Undying Malice':self.assertIn('counter: Some((CounterKind::PlusOne, 1))',text)
   if name=='Fake Your Own Death':self.assertRegex(text,r'ReturnGrantedSource[^\n]+CreateTokens')
 def test_every_unknown_raw_boundary_remains_gated(self):
  for name in ['Not Dead After All','Fake Your Own Death','Undying Malice']:
   c=self.cards[name];raw=c['oracle_text']
   variants=[raw+' (You win the game.)',raw+' ((You win the game.))']
   for m in re.finditer(r'\s+|[a-zA-Z]{3,}',raw):
    index=m.end() if m.group().isspace() else m.start()+1
    for extra in ['(Draw seven cards.)','((Draw seven cards.))']:
     variants.append(raw[:index]+extra+raw[index:])
   for text in variants:
    self.assertFalse(self.compile(dict(c,name='Independent raw grant',oracle_text=text))[1]['implemented'],(name,text))
 def test_malformed_metadata_and_remaining_five_whole_bodies_stay_gated(self):
  manifest=json.loads((ROOT/'docs/temporary-death-admissions.json').read_text())
  initial=set(manifest['initial_consumers'])
  for row in manifest['cards']:
   c=self.cards[row['name']]
   if row['name'] not in initial:self.assertFalse(self.compile(c)[1]['implemented'])
   else:
    for update in [dict(mana_cost='{Y}'),dict(mana_cost='{4294967296}'),dict(type_line='Creature'),dict(layout='adventure')]:
     self.assertFalse(self.compile(dict(c,**update))[1]['implemented'],update)
if __name__=='__main__':unittest.main()
