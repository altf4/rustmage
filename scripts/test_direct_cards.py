"""Direct Rust admissions bypass the English compiler; provenance remains automated."""
import hashlib,json,subprocess,unittest
from unittest.mock import patch
from scripts import direct_cards,generate_catalogue as g
class DirectCardTests(unittest.TestCase):
 def test_authored_cards_bypass_all_english_body_lowering(self):
  records,_=direct_cards.load(g.ROOT)
  facts=json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']
  cards=[c for c in facts if c['oracle_id']in records]
  self.assertGreater(len(cards),0)
  with patch.object(g,'modal_reminder_text',wraps=g.modal_reminder_text) as parser, patch.object(g,'adventure_faces',wraps=g.adventure_faces) as faces:
   payload=json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
   rust,coverage=g.compile_catalogue(dict(cards=cards,card_count=len(cards),cards_sha256=hashlib.sha256(payload).hexdigest()),dict(banned_card_names=[]))
  self.assertTrue(all(call.args[0].get('oracle_id') not in records for call in parser.call_args_list + faces.call_args_list))
  self.assertEqual(sum(r['implementation']=='authored_rust'for r in coverage['cards']),len(records))
  self.assertTrue(all(r['implemented']for r in coverage['cards']if r['standard_legal']))
 def test_authored_java_provenance(self):
  records,_=direct_cards.load(g.ROOT)
  for r in records.values():
   raw=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',r['java_revision']+':'+r['java_source']])
   self.assertEqual(hashlib.sha256(raw).hexdigest(),r['java_sha256'])
if __name__=='__main__':unittest.main()
