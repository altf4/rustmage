"""Real paid complete pairs and actual cast/attack trigger witnesses in Standard."""
import json
from pathlib import Path
import unittest
from rustmage import Game, EngineError
import test_adventure
NAMES=['Aquatic Alchemist // Bubble Up','Scalding Viper // Steam Clean','Intrepid Trufflesnout // Go Hog Wild']
BASICS=['Plains','Island','Mountain','Forest']
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
class AdventureEventTests(unittest.TestCase):
 replay=test_adventure.AdventureTests.replay
 def test_three_complete_paid_pairs_both_seats(self):
  for i in range(3):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,2910+i)
 def test_first_qualifier_witness_survives_long_setup_and_same_turn_inset(self):
  for p in ['P0','P1']:self.scenario(0,p,2922)
 def test_inset_aliases_are_not_independent_deck_cards(self):
  for name in NAMES:
   with self.assertRaises(EngineError):
    with Game([[FACTS[name]['card_faces'][1]['name']]*60,['Forest']*60],deck_rules='Freeform'):pass
 def scenario(self,i,p,seed,library=None):
  q='P1' if p=='P0' else 'P0';name=NAMES[i]
  deck=sum(([n]*8 for n in BASICS),[])+sum(([n]*4 for n in [name,'Bear Cub','Bygone Colossus','Chrome Companion','Quick Study','Shock','Thrill of Possibility']),[])
  self.assertEqual(len(deck),60)
  with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library) as g:
   stage='setup';source=None;target=None;frames=0;attacked=False;event=False;descriptors=set();inset_effect=False;entered_turn=None
   for iteration in range(30000):
    o=g.observe('P0')
    if o['outcome'] is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage));self.assertTrue(attacked,(name,p,seed));self.assertTrue(event,(name,p,seed,'missing actual normal event'));self.assertTrue(inset_effect);self.assertTrue({'Adventure','Normal'}<=descriptors);return frames
    o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p)
    own=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p];lands=[v for v in own if v['card'] in BASICS and not v['tapped']];ally=next((v for v in own if v['card']=='Bear Cub'),None);dead=next((v for v in objects.values() if v['card']=='Shock' and v['zone']=='Graveyard' and v['owner']==p),None)
    for d in o['spell_characteristics']:
     if d['object']['card']==source:
      descriptors.add(d['face']);face=FACTS[name]['card_faces'][1 if d['face']=='Adventure' else 0];self.assertEqual((d['name'],d['mana_cost'],d['type_line']),(face['name'],face['mana_cost'],face['type_line']))
    quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    if quiet and stage=='wait_inset':
     self.assertEqual(objects[source]['zone'],'Exile');self.assertTrue(any(v['object']==objects[source]['reference'] for v in o['adventures']))
     if i==0:
      move=o['last_target_library_move'];self.assertEqual(move['target_before']['card'],target);self.assertEqual(move['destination'],'Top');self.assertEqual(move['owner'],p);self.assertNotIn(target,objects)
     if i==1:self.assertEqual(objects[target]['zone'],'Hand')
     if i==2:self.assertEqual(objects[target]['power'],4)
     inset_effect=True;stage='normal'
    if quiet and stage=='wait_normal':
     self.assertEqual(objects[source]['zone'],'Battlefield');self.assertEqual(objects[source]['controller'],p);stage='finish';entered_turn=o['turn']
    for packet in o['stack']:
     if packet['card']==source and isinstance(packet['kind'],dict) and 'TriggeredAbility' in packet['kind']:
      self.assertEqual(packet['controller'],p)
      if i==1:self.assertEqual(packet['cast_event']['controller'],q)
      if i<2:event=True
    if i==2 and stage=='finish' and any(v['card'].endswith('Food Token') and v['owner']==p for v in own):event=True
    ready=quiet and len(lands)>=6 and (dead if i==0 else ally)
    def score(a):
     k=a['kind'];card=objects.get(a.get('card'),{});n=card.get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='AnnounceAdventure':return 2900 if actor==p and stage=='setup' and n==name and ready else -1000
     if k=='AnnounceExile':return 2900 if actor==p and stage=='normal' and a['card']==source else -1000
     if k=='AnnounceCast':
      if stage=='finish':
       if n=='Quick Study':
        if event:return 1600
        return 2500 if (i==0 and actor==p and o['turn']>entered_turn) or (i==1 and actor==q) else -1000
       return 2200 if n in ['Bear Cub','Bygone Colossus','Chrome Companion'] else -1000
      if n=='Bear Cub':return 2200 if not any(v['card']=='Bear Cub' and v['zone']=='Battlefield' and v['controller']==actor for v in objects.values()) else -1000
      return 2500 if actor==p and i==0 and n=='Shock' and not dead else -1000
     if k=='ChooseTarget':
      ann=o['announcing']
      if ann and ann['card']==source:return 2900 if a['target'].get('Card')==target else -1000
      if ann and ann['source_card']=='Shock':return 2900 if a['target'].get('Player')==q else -1000
      return 2300 if objects.get(a['target'].get('Card'),{}).get('controller')==actor else 1000
     if k=='PlayLand':return 1800-20*sum(v['card']==n and v['zone']=='Battlefield' and v['controller']==actor for v in objects.values())
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2100 if o['decision'] in ['pay_costs','resolution_mana'] and n in BASICS else -1000
     if k in ['PayCosts','PutTrigger','FinishTargets','FinishSearch','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost']:return 2400
     if k in ['Discard','Bottom']:
      if n=='Quick Study' and not event and ((i==0 and actor==p) or (i==1 and actor==q)) and sum(v['card']==n and v['zone']=='Hand' for v in o['objects'])<=1:return -1000
      if actor==p and stage!='finish' and n==name:return -1000
      if actor==p and stage=='setup' and n in BASICS and len([v for v in mine['objects'] if v['zone']=='Hand' and v['card'] in BASICS])<=2:return -500
      return 2200 if n in ['Thrill of Possibility','Bygone Colossus','Chrome Companion','Shock'] else 1000
     if k=='Attack':
      if stage!='finish':return -1000
      if not attacked:return 2900 if actor==p and a['card']==source else -1000
      return 2800 if event else -1000
     if k=='Pass':return 0
     return 1000
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,p,stage,o['decision'],o['actions']))
    if a['kind']=='AnnounceAdventure':source=a['card'];target=(dead if i==0 else ally)['id'];stage='wait_inset'
    if a['kind']=='AnnounceExile':stage='wait_normal'
    if a['kind']=='Attack' and actor==p and a['card']==source:attacked=True
    before=[g.observe(v) for v in ['P0','P1']]
    with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor='P1' if actor=='P0' else 'P0'),a)
    self.assertEqual(before,[g.observe(v) for v in ['P0','P1']]);self.replay(g,o,j);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal outcome'))
