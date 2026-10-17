"""Complete actual-cast-payment bodies, closed costs and authenticated residuals."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('paid_generator',Path(__file__).with_name('generate_catalogue.py'))
g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Aberrant Manawurm','Blazing Bomb','Cuboid Colony','Deluge Virtuoso','Exhibition Tidecaller','Expressive Firedancer','Freestrider Commando','Hungry Graffalon','Muse Seeker','Prompto Argentum','Rancorous Archaic','Sahagin','Shantotto, Tactician Magician','Spectacular Skywhale','Tackle Artist','Textbook Tabulator','Thunderdrum Soloist','Ultros, Obnoxious Octopus','Unravel','Arcane Omens']
class CastPaymentCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.rows={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  cards=[c];snapshot=dict(cards=cards,card_count=1,cards_sha256=hashlib.sha256(json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest())
  return g.compile_catalogue(snapshot,dict(banned_card_names=[]))
 def admitted(self,c):return next(v['implemented'] for v in self.compile(c)[1]['cards'] if v['name']==c['name'])
 def test_twenty_complete_bodies_and_every_unknown_residual_stays_closed(self):
  for n in NAMES:
   c=self.rows[n]
   with self.subTest(name=n):
    self.assertTrue(self.admitted(c))
    for tail in ['\nDiscover 3.','\nWhenever this creature leaves, draw a card.','\n{X}: Draw a card.']:
     self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text']+tail)),n+tail)
    for layout in ['split','transform','modal_dfc']:
     self.assertFalse(self.admitted(dict(c,layout=layout)))
    for cost in ['{X}','{4294967296}','{4294967295}{1}','{Q}']:
     self.assertFalse(self.admitted(dict(c,mana_cost=cost)),n+cost)
 def test_exact_increment_thresholds_and_source_power_residuals(self):
  for n in ['Cuboid Colony','Hungry Graffalon','Textbook Tabulator']:
   c=self.rows[n]
   for old,new in [('power or toughness','power and toughness'),('greater than','less than'),('a spell','an ability')]:
    with self.subTest(name=n,mutant=new):self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace(old,new))))
  c=self.rows['Blazing Bomb']
  for old,new in [('its power','its toughness'),('Sacrifice this creature','Sacrifice another creature'),('target creature','any target')]:
   self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace(old,new))))
  self.assertTrue(self.admitted(dict(c,oracle_text=c['oracle_text'].replace('as a sorcery','during your turn'))))  # Existing real OwnTurn guard remains valid.
  for n in ['Deluge Virtuoso','Exhibition Tidecaller','Expressive Firedancer','Muse Seeker','Spectacular Skywhale','Tackle Artist','Thunderdrum Soloist']:
   c=self.rows[n]
   for old,new in [('five or more','three or more'),('cast that spell','activate that ability')]:
    self.assertIn(old,c['oracle_text']);self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace(old,new))))
 def test_recognized_other_cost_selectors_do_not_revive_payment_listeners(self):
  declarations=[
   (next(l for l in self.rows['Appeal to Eirdu']['oracle_text'].splitlines() if l.startswith('Convoke ')),g.extract_convoke),
   (next(l for l in self.rows['Candy Grapple']['oracle_text'].splitlines() if l.startswith('Bargain ')),g.extract_bargain),
   (next(l for l in self.rows["Djinn of Fool's Fall"]['oracle_text'].splitlines() if l.startswith('Plot ')),g.extract_plot),
   (next(l for l in self.rows['Germinating Wurm']['oracle_text'].splitlines() if l.startswith('Warp ')),g.extract_warp)]
  for n in NAMES:
   c=self.rows[n]
   if g.actual_mana_body(c) is None:continue
   for declaration,extractor in declarations:
    mutant=dict(c,oracle_text=c['oracle_text']+'\n'+declaration)
    with self.subTest(name=n,declaration=declaration):
     self.assertTrue(extractor(mutant)[1]);self.assertFalse(self.admitted(mutant))
 def test_bounded_converge_entry_and_counter_victim_programs_are_complete(self):
  for n in ['Arcane Omens','Rancorous Archaic']:
   c=self.rows[n]
   self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace('colors of mana','types of mana').replace('color of mana','type of mana'))))
  c=self.rows['Unravel'];self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace('less than','greater than'))))
  future=['Ambitious Augmenter','Berta, Wise Extrapolator','Fractal Tender','Pensive Professor','Topiary Lecturer','Colorstorm Stallion','Elemental Mascot','Molten-Core Maestro','Tellah, Great Sage','The Prima Vista','Tester of the Tangential','Wildgrowth Archaic','Magmablood Archaic','Snarl Song','Together as One','Transcendent Archaic','Sundering Archaic','Archaic\'s Agony','Astelli Reclaimer','Mockingbird','Molten Note','Mana Sculpt','Satoru, the Infiltrator','Tokka & Rahzar, Terrible Twos','Uncover the Moon-Letters','Dyadrine, Synthesis Amalgam']
  for n in future:
   with self.subTest(gated=n):self.assertFalse(self.admitted(self.rows[n]))
