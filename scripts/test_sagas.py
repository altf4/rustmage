"""Whole single-face Saga grammar: no unparsed chapter or intrinsic residual."""
import hashlib,importlib.util,json,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('saga_generator',Path(__file__).with_name('generate_catalogue.py'))
g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
CORE=['Summon: Choco/Mog','Summon: Esper Ramuh','Summon: Fat Chocobo','Summon: G.F. Ifrit']
NAMES=CORE+['Burn, Burn, Tree and Fern','Summon: Anima','Leaves from the Vine','Summon: Primal Garuda','Welcome to Sweettooth',"The Huntsman's Redemption",'Roar of Endless Song','Summon: Shiva','Korvold and the Noble Thief']
class SagaCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def implemented(self,c):
  cards=[c];snapshot=dict(cards=cards,card_count=1,cards_sha256=hashlib.sha256(json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest())
  return next(v['implemented']for v in g.compile_catalogue(snapshot,dict(banned_card_names=[]))[1]['cards']if v['name']==c['name'])
 def test_core_full_bodies_grouped_numbers_and_no_missing_residuals(self):
  for name in NAMES:
   c=self.cards[name];self.assertTrue(self.implemented(c),name)
   for text in [c['oracle_text']+'\nDiscover 3.',c['oracle_text'].replace('I, II','II, I'),c['oracle_text'].replace('Sacrifice after IV.','Sacrifice after V.'),c['oracle_text']+'\nI — Draw a card.',c['oracle_text'].replace('I — ','I — Discover — ',1)]:
    if text!=c['oracle_text']:self.assertFalse(self.implemented(dict(c,oracle_text=text)),(name,text))
  for name,c in self.cards.items():
   if 'Saga'in c.get('type_line','')and name not in NAMES:self.assertFalse(self.implemented(c),name)
 def test_parenthesized_unknown_chapter_and_intrinsic_residuals_stay_closed(self):
  for name in NAMES:
   c=self.cards[name]
   text=c['oracle_text'];lines=text.splitlines();chapter=next(i for i,line in enumerate(lines)if ' — 'in line)
   for residual in [' (You win the game.)',' (Discover 2.)',' (If a permanent with a stun counter would become untapped, draw a card instead.)']:
    changed=list(lines);changed[chapter]+=residual
    self.assertFalse(self.implemented(dict(c,oracle_text='\n'.join(changed))),(name,residual))
   self.assertFalse(self.implemented(dict(c,oracle_text=text+'\n(You win the game.)')),name)
   if ' (If a permanent with a stun counter' in text:
    self.assertFalse(self.implemented(dict(c,oracle_text=text.replace('remove one from it instead.','draw a card instead.'))),name)
   if ' (It\'s an artifact with' in text and 'You gain 3 life.' in text:
    self.assertFalse(self.implemented(dict(c,oracle_text=text.replace('You gain 3 life.','You win the game.'))),name)
 def test_real_composite_selectors_and_read_ahead_remain_closed(self):
  controls=[('Appeal to Eirdu',g.extract_convoke,'Convoke'),('Candy Grapple',g.extract_bargain,'Bargain'),("Djinn of Fool's Fall",g.extract_plot,'Plot'),('Burst Lightning',g.extract_kicker,'Kicker')]
  for source,extract,prefix in controls:
   row=self.cards[source];line=next(l for l in row['oracle_text'].splitlines()if l.startswith(prefix));self.assertTrue(extract(dict(row,oracle_text=line))[1],prefix)
   for name in NAMES:
    c=self.cards[name];self.assertFalse(self.implemented(dict(c,oracle_text=c['oracle_text']+'\n'+line)),(name,line))
  c=self.cards[CORE[0]]
  self.assertFalse(self.implemented(dict(c,oracle_text=c['oracle_text']+'\nRead ahead')))
  for cost in ['{X}','{4294967296}','{Q}','','{W/P}']:
   self.assertFalse(self.implemented(dict(c,mana_cost=cost)),cost)
if __name__=='__main__':unittest.main()
