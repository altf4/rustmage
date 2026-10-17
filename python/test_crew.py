"""Paid Vehicle spells, private Crew costs, live kinds and complete residuals."""
import unittest
from rustmage import Game,EngineError
NAMES=["Adventurer's Airship","Air Response Unit","Boosted Sloop","Broadcast Rambler","Burner Rocket","Careening Mine Cart","Clamorous Ironclad","Cultivator's Caravan","Debris Beetle","Dependable Quinjet","Fire Nation Warship","Haunted Hellride","Hulldrifter","Lumbering Worldwagon","Magitek Armor","Ripclaw Wrangler","S.H.I.E.L.D. Helicarrier","Skybox Ferry","Spotcycle Scouter","Strixhaven Skycoach","Thopter Fabricator","Tundra Tank","Turtle Blimp","Veloheart Bike"]
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class CrewTests(unittest.TestCase):
 def replay(self,g,o,j):
  actor=o['actor'];other='P1' if actor=='P0' else 'P0';saved=g.snapshot()
  parent=g.clone();child=parent.clone();parent.close()
  with child:expected=child.step(child.observe(actor),j);other_expected=child.observe(other)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(expected,after)
  other_after=g.observe(other);other_expected['revision']=other_after['revision'];self.assertEqual(other_expected,other_after)
  g.drop_snapshot(saved);return after
 def test_all_twenty_four_paid_spells_actual_crew_and_attack_both_seats(self):
  for i,name in enumerate(NAMES):
   for p in ['P0','P1']:
    with self.subTest(card=name,seat=p):self.scenario(i,p)
 def scenario(self,i,p):
  name=NAMES[i];other='P1' if p=='P0' else 'P0';pi=int(p[-1])
  own=sum(([b]*12 for b in BASICS),[])+[name]*12+['Bear Cub']*24+['Thrill of Possibility']*12+['Disenchant']*12
  enemy=['Forest']*80
  with Game([own,enemy] if p=='P0' else [enemy,own],seed=250+i,starting_player=p,deck_rules='Freeform') as g:
   paid=False;source=None;entry_checked=False;entry=None;crewed=False;attack=False;cycling=False;cycle_done=False;fabricated=False;death=False;attack_base=None;arranged=0;cycled_base=None
   for step in range(24000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,step));o=g.observe(o['actor']);actor=o['actor'];objs={v['id']:v for v in o['objects']}
    own_view=g.observe(p);own_objs={v['id']:v for v in own_view['objects']};body=own_objs.get(source)
    quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    token=lambda sub:sum(v['owner']==p and v['zone']=='Battlefield' and bool(v.get('token')) and v['token']['subtypes']==sub for v in own_objs.values())
    if paid and quiet and body and body['zone']=='Battlefield' and not entry_checked:
     self.assertEqual(body['kinds']&1,0);self.assertEqual((body['power'],body['toughness']),(0,0));self.assertEqual(body['card'],name)
     if i in [3,14,16,22]:self.assertEqual(token({3:'Thopter',14:'Hero',16:'Soldier',22:'Mutant'}[i]),2 if i==16 else 1)
     if i==4:self.assertTrue(any(v['owner']==p and v['zone']=='Battlefield' and v['card']=='Bear Cub' and int(v['power'])==4 and v['keywords']&256 for v in own_objs.values()))
     if i==8:self.assertEqual(own_view['players'][pi]['life'],entry['players'][pi]['life']+3);self.assertEqual(own_view['players'][1-pi]['life'],entry['players'][1-pi]['life']-3)
     if i==12:self.assertEqual(own_view['players'][pi]['cards_drawn_this_turn'],entry['players'][pi]['cards_drawn_this_turn']+2)
     if i==13:self.assertEqual(sum(v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS for v in own_objs.values()),sum(v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS for v in entry['objects'])+1)
     if i==15:self.assertEqual(own_view['players'][1-pi]['cards_discarded_this_turn'],entry['players'][1-pi]['cards_discarded_this_turn']+1)
     if i==18:self.assertEqual(arranged,2)
     if i==19:self.assertTrue(own_view['known_hand'])
     if i==21:self.assertTrue(any(v['owner']==p and v['zone']=='Battlefield' and v['card']=='Bear Cub' and v['keywords']&8192 for v in own_objs.values()))
     if i==23:self.assertEqual(own_view['players'][pi]['life'],entry['players'][pi]['life']+2)
     entry_checked=True
    if attack and quiet:
     if i in [0,2]:self.assertEqual(own_view['players'][pi]['cards_discarded_this_turn'],attack_base['players'][pi]['cards_discarded_this_turn']+1)
     if i==5:self.assertEqual(token('Treasure'),1)
     if i==11:self.assertTrue(any(v['owner']==p and v['zone']=='Battlefield' and v['keywords']&128 and not v['tapped'] for v in own_objs.values()))
     if i==13:self.assertEqual(sum(v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS for v in own_objs.values()),sum(v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS for v in attack_base['objects'])+1)
     if i==21:self.assertEqual(own_view['players'][pi]['mana'][3],1)
     if i not in [10]:return
    if death and quiet:self.assertEqual(token('Clue'),1);return
    if cycle_done and quiet:
     self.assertEqual(own_view['players'][pi]['cards_drawn_this_turn'],cycled_base+1);cycle_done=False;cycling=True
    if i==20 and entry_checked and token('Thopter')==1:fabricated=True
    def score(a):
     k=a['kind']
     if k in ['AnnounceCast','Cast','BeginCast']:
      n=objs[a['card']]['card']
      if actor!=p:return -1000
      if n=='Bear Cub':return 1800 if sum(v['owner']==p and v['zone']=='Battlefield' and v['card']=='Bear Cub' for v in objs.values())<3 else -1000
      if n==name:return 1900 if not paid and k=='AnnounceCast' and sum(v['owner']==p and v['zone']=='Battlefield' and v['card']=='Bear Cub' for v in objs.values())>=3 else -1000
      if n=='Thrill of Possibility':return 2000 if i==20 and entry_checked and not fabricated else -1000
      if n=='Disenchant':return 2200 if i==10 and attack and not death else -1000
      return -1000
     if k=='AnnounceActivate':
      if actor!=p or objs[a['card']]['card']!=name:return -1000
      if objs[a['card']]['zone']=='Hand':return 2100 if i in [6,17] and entry_checked and not cycling and not cycle_done else -1000
      return 1900 if entry_checked and not attack and (i!=20 or fabricated) and (i not in [6,17] or cycling) and not body['summoning_sick'] and body['kinds']&1==0 and o['active']==p and o['step']=='Main1' else -1000
     if k=='ChooseOptional':return 2000 if a['accept'] else -1000
     if k=='ChooseCrewer':return 1800+int(objs[a['object']['card']]['power'])
     if k=='FinishCrew':return 2300
     if k=='ChooseAdditionalCost':return 2200 if objs[a['object']['card']]['card']=='Bear Cub' else 1200
     if k=='ChooseTarget':
      t=a['target'].get('Card');return 2300 if i==10 and attack and t==source else 1800 if t in objs and objs[t]['owner']==p and objs[t]['card']=='Bear Cub' else 1000
     if k=='Attack':return 2000 if actor==p and a['card']==source and crewed and not attack else -1000
     if k=='PlayLand':return 1300-20*sum(v['owner']==actor and v['zone']=='Battlefield' and v['card']==objs[a['card']]['card'] for v in objs.values())
     if k=='TapMana':return 1900 if o['decision']=='pay_costs' else -1000
     if k=='Discard':return 1800 if objs[a['card']]['card'] in BASICS else 1000
     if k=='ArrangeLibrary':return 1800 if a['destination']=='Top' else 1000
     return {'Keep':2500,'Mulligan':-10000,'Concede':-10000,'PayCosts':2300,'Pass':0,'PutTrigger':2000,'FinishTargets':1600,'ChooseSearch':1800,'FinishSearch':1700,'ChooseSpellCost':1800,'FinishAttackers':1200,'FinishBlockers':1200}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,o['decision'],o['actions']))
    if o['crew_choice']:
     self.assertIsNone(g.observe(other)['crew_choice']);self.assertIsNone(o['payment_request']);views=[g.observe(q) for q in ['P0','P1']]
     with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor=other),a)
     with self.assertRaises(EngineError):g.act(o,{'kind':'ChooseCrewer','object':body['reference']})
     self.assertEqual(views,[g.observe(q) for q in ['P0','P1']])
    ann=o['announcing']
    if a['kind']=='PayCosts' and ann and ann['source_card']==name and ann['kind']=='Spell':paid=True;source=ann['card'];entry=own_view
    if a['kind']=='PayCosts' and ann and ann['source_card']==name and ann['kind']!='Spell' and objs[ann['card']]['zone']=='Hand':cycle_done=True;cycled_base=own_view['players'][pi]['cards_drawn_this_turn']
    if a['kind']=='ArrangeLibrary' and entry and not entry_checked:arranged+=1
    if a['kind']=='FinishCrew':crewed=True
    if a['kind']=='Attack' and a['card']==source:attack=True;attack_base=own_view
    if a['kind']=='PayCosts' and ann and ann['source_card']=='Disenchant':death=True
    after=self.replay(g,o,j)
    if a['kind']=='FinishCrew':
     self.assertTrue(after['crewed_vehicles']);packet=after['stack'][-1];self.assertTrue(packet['crew']['contributors']);self.assertEqual(packet['crew']['contributors'],o['crew_choice']['selected'])
   self.fail(('unreached',name,p,paid,entry_checked,crewed,attack))
