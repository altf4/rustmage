"""Actual paid complete Ward attachments, linked exile and recipient-owned payment."""
import unittest
from rustmage import Game,EngineError
NAMES=['Dusk Rose Reliquary','Trapped in the Screen','Dwarven Mattock',"Falcon's Wing Harness",'Lavaspur Boots','Hardlight Containment','Sheltered by Ghosts','Super Strength','Detention Chariot','Static Snare']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class AttachmentWardTests(unittest.TestCase):
 def replay(self,g,o,j):
  a=o['actions'][j];actor=o['actor'];other='P1'if actor=='P0'else'P0';saved=g.snapshot()
  parent=g.clone();child=parent.clone();parent.close()
  with child:expected=child.step(child.observe(actor),j);opponent=child.observe(other)
  before=[g.observe(p)for p in ['P0','P1']]
  with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor=other),a)
  self.assertEqual(before,[g.observe(p)for p in ['P0','P1']]);g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(after,expected)
  actual=g.observe(other);opponent['revision']=actual['revision'];self.assertEqual(actual,opponent);g.drop_snapshot(saved);return after
 def test_ten_complete_paid_bodies_equip_exile_return_and_neighbors_both_seats(self):
  for i in range(10):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,None)
 def test_eight_ward_sources_and_grants_pay_and_decline_both_seats(self):
  for i in range(8):
   for p in ['P0','P1']:
    for pay in [False,True]:
     with self.subTest(card=NAMES[i],seat=p,pay=pay):self.scenario(i,p,pay)
 def test_paid_super_strength_on_opponent_grants_recipient_owned_ward(self):
  for p in ['P0','P1']:
   for pay in [False,True]:
    with self.subTest(seat=p,pay=pay):self.scenario(7,p,pay,foreign=True)
 def scenario(self,i,p,ward,foreign=False):
  q='P1'if p=='P0'else'P0';name=NAMES[i]
  own=sum(([b]*12 for b in BASICS),[])+[name]*12+['Omni-Cheese Pizza']*12+['Axgard Cavalry']*12+['Bear Cub']*12+['Disenchant']*12
  if foreign:own+=['Shock']*12
  enemy=sum(([b]*12 for b in BASICS),[])+['Bear Cub']*24+['Shock']*12+['Disenchant']*12
  with Game([own,enemy]if p=='P0'else[enemy,own],seed=257+i,starting_player=p,deck_rules='Freeform')as g:
   source=None;paid=False;entry=False;equipped=False;ward_cast=False;ward_packet=None;ward_done=False;removed=False;remove_paid=False;cycle_paid=False;cycled=False;crew_paid=False;crewed=False;before_cycle=None;target=None;enemy_id=None;spell=None;attacked=False
   for step in range(24000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,ward,step));o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']}
    own_view=g.observe(p);own_objects={v['id']:v for v in own_view['objects']}
    def field(owner,n):return next((v for v in objects.values()if v['owner']==owner and v['zone']=='Battlefield'and v['card']==n),None)
    bear=field(p,'Bear Cub');dwarf=field(p,'Axgard Cavalry');pizza=field(p,'Omni-Cheese Pizza');enemy_bear=field(q,'Bear Cub');body=objects.get(source)
    quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    if paid and quiet and body and body['zone']=='Battlefield'and not entry:
     self.assertEqual(body['card'],name)
     if i in [0,1,5,6,8,9]:self.assertEqual(own_objects[enemy_id]['zone'],'Exile')
     if i in [2,3]:self.assertEqual(body['attached_to'],dwarf['reference'])
     if i in [5,6,7]:self.assertEqual(body['attached_to'],target)
     if i==5:self.assertEqual(objects[target['card']]['kinds']&1,0)
     entry=True
    if equipped and quiet and body and not removed and not ward_done:
     recipient=objects[target['card']];self.assertEqual(body['attached_to'],target)
     bonus={2:(2,2,0),3:(1,1,1),4:(1,0,4)}[i]
     self.assertEqual((int(recipient['power']),int(recipient['toughness'])),(2+bonus[0],2+bonus[1]));self.assertEqual(recipient['keywords']&bonus[2],bonus[2])
    if entry and i in [6,7]and not removed and not ward_done:
     recipient=objects[target['card']];bonus={6:(1,0,64),7:(4,4,256)}[i]
     self.assertEqual((int(recipient['power']),int(recipient['toughness'])),(2+bonus[0],2+bonus[1]));self.assertEqual(recipient['keywords']&bonus[2],bonus[2])
    if cycle_paid and quiet and not cycled:
     self.assertEqual(own_view['players'][int(p[-1])]['cards_drawn_this_turn'],before_cycle+1);cycled=True
    if crew_paid and quiet and not crewed:
     self.assertEqual((int(body['power']),int(body['toughness'])),(6,6));self.assertEqual(body['kinds']&1,1);crewed=True
    ready=quiet and entry and (i not in [2,3,4]or equipped)and(i!=8 or cycled and crewed)
    if ward_cast:
     for packet in o['stack']:
      if packet.get('related_stack')and packet['related_stack']['id']==ward_packet:
       self.assertEqual(packet['controller'],q if foreign else p);self.assertEqual(packet['related_stack']['controller'],p if foreign else q)
       if i not in [0,1]:self.assertTrue(packet['attachment_trigger']['granted']);self.assertEqual(packet['attachment_trigger']['recipient'],target)
    if ward_done and quiet:
     self.assertEqual(objects[spell]['zone'],'Graveyard')
     victim=objects[source if i in [0,1]else target['card']]
     if ward is False:self.assertEqual(victim['zone'],'Battlefield')
     elif i in [0,1,5]:self.assertEqual(victim['zone'],'Graveyard')
     elif victim['zone']=='Battlefield':self.assertEqual(int(victim['damage']),2)
     return
    if remove_paid and quiet:
     self.assertEqual(body['zone'],'Graveyard')
     if i in [0,1,5,6,8,9]:self.assertEqual(objects[enemy_id]['zone'],'Battlefield')
     return
    def land_funds(n,owner):
     cost={'Bear Cub':(2,4,1),'Axgard Cavalry':(2,3,1),'Omni-Cheese Pizza':(2,0,0),'Disenchant':(2,0,1),'Shock':(1,3,1),name:([(1,0,1),(3,0,1),(2,0,0),(2,1,1),(1,0,0),(1,0,1),(2,0,1),(5,4,1),(6,0,2),(4,0,1)][i])}[n]
     pool=o['players'][int(owner[-1])]['mana'];available=[pool[z]+sum(v['owner']==owner and v['zone']=='Battlefield'and not v['tapped']and v['card']==BASICS[z]for v in objects.values())for z in range(5)]+[pool[5]]
     return sum(available)>=cost[0]and available[cost[1]]>=cost[2]
    def score(a):
     k=a['kind']
     if k in ['Cast','BeginCast','Activate','BeginActivate','Mulligan','Concede']:return -1000
     if k=='AnnounceCast':
      n=objects[a['card']]['card']
      if actor==p:
       if not land_funds(n,p):return -1000
       if n in ['Axgard Cavalry','Bear Cub','Omni-Cheese Pizza']and n!=name:return 1800 if not field(p,n)and not paid else -1000
       if n==name:return 2000 if not paid and bear and dwarf and pizza and enemy_bear and(i!=9 or bool(o['attackers']))else -1000
       if n=='Disenchant':return 2100 if ward is None and ready and not remove_paid else -1000
       if n=='Shock':return 2200 if foreign and ready and not ward_cast else -1000
      else:
       if n=='Bear Cub':return 1700 if not enemy_bear and not paid else -1000
       wanted='Disenchant'if i in [0,1,5]else'Shock'
       if n==wanted:return 2200 if not foreign and ward is not None and ready and not ward_cast else -1000
      return -1000
     if k=='AnnounceActivate':
      obj=objects[a['card']]
      if actor==p and i in [2,3,4]and obj['id']==source:return 2200 if entry and not equipped else -1000
      if actor==p and i==8 and obj['card']==name:
       if obj['zone']=='Hand':return 2100 if entry and not cycle_paid and land_funds('Disenchant',p)else -1000
       return 2100 if cycled and not crew_paid else -1000
      return -1000
     if k=='ChooseTarget':
      t=a['target'].get('Card');ann=o['announcing'];n=ann['source_card']if ann else''
      if n=='Disenchant':wanted=source if actor==p or i in [0,1]else target['card'];return 2600 if t==wanted else -1000
      if n=='Shock':return 2600 if t==target['card']else -1000
      if ann and ann['kind']=='Spell'and n==name and i in [5,6,7]:wanted=pizza['id']if i==5 else enemy_bear['id']if foreign else bear['id'];return 2500 if t==wanted else -1000
      if n==name and ann and ann['kind']=='ActivatedAbility':return 2600 if t==bear['id']else -1000
      if n==name and ann and isinstance(ann['kind'],dict)and 'ActivatedAbility'in ann['kind']:return 2600 if t==bear['id']else -1000
      if n==name and i in [2,3]:return 2500 if dwarf and t==dwarf['id']else -1000
      if n==name and i in [0,1,5,6,8,9]:return 2500 if enemy_bear and t==enemy_bear['id']else -1000
      if t in objects and objects[t]['owner']==p and objects[t]['card']=='Bear Cub':return 1800
      return -1000
     if k=='ChooseAdditionalCost':return 2400 if objects[a['object']['card']]['card']=='Omni-Cheese Pizza'else -1000
     if k=='ChooseCrewer':return 2000 if objects[a['object']['card']]['card']in['Bear Cub','Axgard Cavalry']else -1000
     if k=='FinishCrew':return 2600
     if k=='PayResolutionMana':return 2600 if ward is True else -1000
     if k=='DeclineResolutionMana':return 2600 if ward is False else -1000
     if k=='TapMana':
      if not(o['decision']=='pay_costs'or o.get('resolution_payment')and ward is True):return -1000
      domain=o.get('mana_payment');z=BASICS.index(objects[a['card']]['card'])
      if domain and domain['available'][z]<domain['required'][z]:return 2450
      return 2300
     if k=='Attack':return 2000 if i==9 and actor==q and not paid and land_funds(name,p)and bear and dwarf and pizza and any(v['card']==name and v['zone']=='Hand'for v in own_objects.values())and sum(v['owner']==p and v['zone']=='Battlefield'and v['card']in BASICS for v in objects.values())>=4 else -1000
     if k=='PlayLand':return 1500-20*sum(v['owner']==actor and v['zone']=='Battlefield'and v['card']==objects[a['card']]['card']for v in objects.values())
     if k=='Discard':
      n=objects[a['card']]['card'];copies=sum(v['zone']=='Hand'and v['owner']==actor and v['card']==n for v in objects.values())
      if n==name:return 1900 if copies>2 else -100
      if n=='Disenchant':return 2100 if copies>1 else 100
      if n in ['Axgard Cavalry','Bear Cub','Omni-Cheese Pizza']:return 2200 if field(actor,n)or copies>1 else 200
      return 1800 if n in BASICS else 1000
     return {'Keep':3000,'Pass':0,'PutTrigger':2500,'FinishTargets':1600,'PayCosts':2600,'ChooseSpellCost':1800,'FinishAttackers':1300,'FinishBlockers':1300}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,p,ward,o['decision'],o['actions']))
    ann=o['announcing']
    if a['kind']=='PayCosts'and ann:
     if ann['source_card']==name and ann['kind']=='Spell':
      paid=True;source=ann['card'];enemy_id=enemy_bear['id']if enemy_bear else None
      if i in [5,6,7]:target=pizza['reference']if i==5 else enemy_bear['reference']if foreign else bear['reference']
      if i==9:self.assertEqual(sum(a['payment']),4)
     elif ann['source_card']==name and objects[ann['card']]['zone']=='Hand':cycle_paid=True;before_cycle=own_view['players'][int(p[-1])]['cards_drawn_this_turn']
     elif ann['source_card']==name and i in [2,3,4]:equipped=True;target=bear['reference']
     elif ann['source_card']=='Disenchant'and actor==p:remove_paid=True;removed=True
     elif ann['source_card']in ['Shock','Disenchant']and actor==(p if foreign else q):ward_cast=True;ward_packet=ann['id'];spell=ann['card']
    if a['kind']=='FinishCrew':crew_paid=True
    if a['kind']in ['PayResolutionMana','DeclineResolutionMana']:ward_done=True
    if a['kind']=='Attack':attacked=True
    self.replay(g,o,j)
   self.fail(('unreached',name,p,ward,paid,entry,equipped,cycle_paid,cycled,crew_paid,ward_cast))
