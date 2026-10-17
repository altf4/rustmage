"""Full physical Tiered inventory; never an execution whitelist."""
import hashlib,json,subprocess,unittest
from scripts import generate_catalogue as g
class TieredTests(unittest.TestCase):
 def test_six_physical_bodies_all_eighteen_modes_and_full_java_are_pinned(self):
  manifest=json.loads((g.ROOT/'docs/tiered-admissions.json').read_text());cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']if 'Tiered' in c.get('oracle_text','')}
  self.assertEqual(len(cards),6);self.assertEqual({r['facts']['name']for r in manifest['cards']},set(cards));self.assertEqual(sum(len(r['tiers'])for r in manifest['cards']),18)
  records=[]
  for row in manifest['cards']:
   card=cards[row['facts']['name']];self.assertEqual(row['facts'],{k:card.get(k)for k in row['facts']});self.assertEqual(card['layout'],'normal');self.assertFalse(card.get('card_faces'));self.assertEqual(len(row['tiers']),3);self.assertEqual(card['oracle_text'].splitlines()[0],'Tiered (Choose one additional cost.)');records.extend(row['sources'])
   for tier in row['tiers']:self.assertIn('• '+tier['label']+' — '+tier['additional_cost']+' — '+tier['body'],card['oracle_text'])
  records.extend(manifest['shared_sources']);self.assertEqual(len(records),140);self.assertEqual(manifest['source_records'],140)
  unique={r['java_source']:r['java_sha256']for r in records};self.assertEqual(len(unique),87);self.assertEqual(unique,{r['java_source']:r['java_sha256']for r in manifest['unique_sources']})
  for path,sha in unique.items():
   raw=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',manifest['java_revision']+':'+path]);self.assertEqual(hashlib.sha256(raw).hexdigest(),sha)
if __name__=='__main__':unittest.main()
