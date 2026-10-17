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
if __name__=='__main__':unittest.main()
