"""Whole split-destination search consumers, private choice grammar and strict residuals."""
import copy,hashlib,importlib.util,json,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('split_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Bloomvine Regent // Claim Territory','Troop of Ponies']
class SplitSearchCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  a=[c];return g.compile_catalogue(dict(cards=a,card_count=1,cards_sha256=hashlib.sha256(json.dumps(a,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()),dict(banned_card_names=[]))
 def admitted(self,c):return next(v['implemented']for v in self.compile(c)[1]['cards']if v['name']==c['name'])
 def test_two_complete_consumers_and_typed_full_body_instructions(self):
  for n in NAMES:
   c=self.cards[n];rust,_=self.compile(c);self.assertTrue(self.admitted(c));self.assertIn('BattlefieldThenHand { battlefield: 1 }',rust)
  rust,_=self.compile(self.cards[NAMES[0]]);self.assertIn('SearchFilter::AllOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Forest")])',rust);self.assertIn('include_source: true',rust);self.assertIn('GainLife(PlayerSet::Controller, 3)',rust)
  rust,_=self.compile(self.cards[NAMES[1]]);self.assertIn('tap: true, sacrifice: true',rust)
 def test_all_unknown_residuals_and_parentheses_stay_closed(self):
  for n in NAMES:
   c=self.cards[n]
   for face in range(2)if n==NAMES[0]else[None]:
    for clause in ['\nDiscover 3.',' (Draw twenty cards.)',' (Unknown reminder.)',' (unclosed',')']:
     t=copy.deepcopy(c);body=t if face is None else t['card_faces'][face];body['oracle_text']+=clause
     with self.subTest(card=n,face=face,clause=clause):self.assertFalse(self.admitted(t))
  c=self.cards[NAMES[1]]
  for text in [c['oracle_text'].replace('Search','SEARCH')+'\n(Draw a card.)','\n'.join('\u2003'+line for line in c['oracle_text'].splitlines())+'\n (You win the game.)']:
   self.assertFalse(self.admitted(dict(c,oracle_text=text)))
 def test_nested_or_token_splitting_parentheticals_cannot_hide_new_routes(self):
  c=self.cards[NAMES[1]];text=c['oracle_text']
  for word in ['Search','library','put','one','onto','battlefield','tapped','other','hand']:
   mutated=text.replace(word,word[:1]+'(You win the game.)'+word[1:],1)
   for body in [mutated,'('+mutated+')']:
    with self.subTest(word=word,body=body):self.assertFalse(self.admitted(dict(c,oracle_text=body)))
 def test_exact_count_reveal_destination_and_cost_boundaries(self):
  for n in NAMES:
   c=self.cards[n]
   for old,new in [('up to two','up to three'),('reveal them, ',''),('battlefield tapped','battlefield'),('the other into your hand','the other into your graveyard')]:
    t=copy.deepcopy(c);body=t if n==NAMES[1]else t['card_faces'][1];body['oracle_text']=body['oracle_text'].replace(old,new);self.assertFalse(self.admitted(t),(n,old))
   for cost in ['{X}','{Y}','{4294967296}','{W/P}']:
    t=copy.deepcopy(c)
    if n==NAMES[0]:t['card_faces'][1]['mana_cost']=cost;t['mana_cost']=' // '.join(f['mana_cost']for f in t['card_faces'])
    else:t['mana_cost']=cost
    self.assertFalse(self.admitted(t))
 def test_admission_is_semantic_not_a_name_whitelist(self):
  for n in NAMES:
   c=copy.deepcopy(self.cards[n])
   if n==NAMES[0]:c['card_faces'][0]['name']='Renamed Dragon';c['name']=' // '.join(f['name']for f in c['card_faces'])
   else:c['name']='Renamed Horse'
   self.assertTrue(self.admitted(c))
if __name__=='__main__':unittest.main()
