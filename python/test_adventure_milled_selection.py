"""Paid complete mill-recovery pairs and owned choices in ordinary Standard games."""
import json
from pathlib import Path
import unittest
from rustmage import Game, EngineError
import test_adventure
NAMES=['Bramble Familiar // Fetch Quest','Most Decrepit Old Bird // Speak Secrets','Picklock Prankster // Free the Fae','Thranduil, Sindarin Liege // Silvan Rally','Beluna Grandsquall // Seek Thrills','Glamdring, Foe-hammer // Gleam of Death']
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
BASICS=['Plains','Island','Forest','Mountain']
class AdventureMilledSelectionTests(unittest.TestCase):
 replay=test_adventure.AdventureTests.replay
 def test_six_paid_complete_pairs_both_seats(self):
  for i in range(6):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,2830+i)
 def test_six_inset_aliases_cannot_be_deck_cards(self):
  for n in NAMES:
   with self.assertRaises(EngineError):
    with Game([[FACTS[n]['card_faces'][1]['name']]*60,['Forest']*60],deck_rules='Freeform'):pass
 def scenario(self,i,p,seed,library=None):
  name=NAMES[i];q='P1' if p=='P0' else 'P0';pi=int(p[-1])
  support=NAMES[2] if i!=2 else NAMES[0]
  deck=sum(([n]*8 for n in BASICS),[])+sum(([n]*4 for n in [name,support,'Bear Cub','Bygone Colossus','Chrome Companion','Quick Study','Shock']),[])
  self.assertEqual(len(deck),60)
  with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library) as g:
   stage='setup';source=None;frames=0;attacked=False;attached=False;before_library=None;descriptors=set();choices=0;recipients=[]
   for _ in range(30000):
    o=g.observe('P0')
    if o['outcome'] is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage));self.assertTrue(attacked,(name,p,seed));self.assertTrue({'Adventure','Normal'}<=descriptors)
     if i==5:self.assertTrue(attached)
     return frames
    o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};hand=[v for v in objects.values() if v['owner']==p and v['zone']=='Hand'];own=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield'];lands=[v for v in own if v['card']in BASICS]
    quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    for d in o['spell_characteristics']:
     if d['physical_card']==name:descriptors.add(d['face']);face=FACTS[name]['card_faces'][1 if d['face']=='Adventure' else 0];self.assertEqual((d['name'],d['mana_cost'],d['type_line']),(face['name'],face['mana_cost'],face['type_line']))
    if o.get('milled_choice') is not None:
     v=o['milled_choice'];self.assertEqual(actor,p);self.assertIsNone(g.observe(q).get('milled_choice'));self.assertEqual(g.observe(q)['actions'],[])
     self.assertEqual(v['maximum'],2 if i==3 else 1);self.assertEqual(v['minimum'],0 if i==3 else 1)
     for a in o['actions']:
      if a['kind']=='SelectMilled':self.assertIn(a['object'],v['candidates'])
     if i!=3:self.assertFalse(any(a['kind']=='FinishMilledSelection' for a in o['actions']))
    if quiet and stage=='wait_inset':
     self.assertEqual(objects[source]['zone'],'Exile');self.assertTrue(any(v['object']==objects[source]['reference'] for v in o['adventures']));self.assertEqual(o['players'][pi]['library_count'],before_library-[7,4,4,4,7,6][i]);stage='normal'
    if quiet and stage=='wait_normal':
     self.assertEqual(objects[source]['zone'],'Battlefield');self.assertFalse(any(v['object']['card']==source for v in o['adventures']))
     for keyword,bit in [('Flying',1),('Vigilance',8),('Trample',256)]:
      if keyword in FACTS[name]['card_faces'][0]['oracle_text']:self.assertTrue(objects[source]['keywords']&bit)
     stage='finish'
    if source is not None and source in objects and objects[source].get('attached_to') is not None:attached=True
    allies=[v for v in own if v.get('power') is not None]
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='AnnounceAdventure':return 2900 if actor==p and stage=='setup' and n==name and len(lands)>=8 else -1000
     if k=='AnnounceExile':return 2900 if actor==p and stage=='normal' and a['card']==source else -1000
     if k=='AnnounceCast':return 2300 if stage=='finish' and n in ['Bear Cub','Bygone Colossus','Chrome Companion'] else -1000
     if k in ['AnnounceActivate','Activate','BeginActivate']:return 2600 if i==5 and stage=='finish' and actor==p and a.get('card')==source and not attached and allies else -1000
     if k=='ChooseTarget':return 2800 if actor==p and a['target'].get('Card') in [v['id']for v in allies] else -1000
     if k=='SelectMilled':return 2800
     if k=='FinishMilledSelection':return 1500
     if k=='ChooseAuraAttachment':return 2800
     if k=='PlayLand':return 1800-20*sum(v['owner']==actor and v['zone']=='Battlefield' and v['card']==n for v in objects.values())
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2100 if o['decision']in ['pay_costs','resolution_mana'] and n in BASICS else -1000
     if k in ['PayCosts','PutTrigger','FinishTargets','FinishSearch','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost']:return 2400
     if k in ['Discard','Bottom']:
      if actor==p and stage!='finish' and n==name:return -1000
      if actor==p and stage=='setup' and n in BASICS and len(lands)<8:return 200
      return 2200
     if k=='Attack':return 2800 if stage=='finish' else -1000
     if k=='Pass':return 0
     return 1000
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,p,stage,o['decision']))
    if a['kind']=='AnnounceAdventure':source=a['card'];before_library=o['players'][pi]['library_count'];stage='wait_inset'
    if a['kind']=='AnnounceExile':stage='wait_normal'
    if a['kind']=='SelectMilled':choices+=1;recipients.append(a['object'])
    if a['kind']=='Attack' and (a['card']==source or i==5 and attached):attacked=True
    before=[g.observe(v)for v in ['P0','P1']]
    with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor='P1'if actor=='P0'else'P0'),a)
    self.assertEqual(before,[g.observe(v)for v in ['P0','P1']]);self.replay(g,o,j);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal outcome'))
