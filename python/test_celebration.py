"""Paid complete celebration bodies, historical public entries and owned replay."""
import unittest
from rustmage import Game,EngineError
NAMES=['Armory Mice','Ash, Party Crasher','Belligerent of the Ball','Bespoke Battlegarb','Gallant Pie-Wielder','Grand Ball Guest','Lady of Laughter','Pests of Honor','Raging Battle Mouse','Tuinvale Guide']
BASE=[(3,1),(2,2),(3,3),(0,0),(2,3),(2,2),(4,5),(2,2),(2,1),(2,3)]
COST=[2,2,3,2,3,2,5,3,2,4]
class CelebrationTests(unittest.TestCase):
 def replay(self,g,o,a):
  saved=g.snapshot();other='P1' if o['actor']=='P0' else 'P0'
  if o['decision']=='targets' and o['announcing'] and o['announcing']['source_card']=='Bespoke Battlegarb':
   before=[g.observe(v) for v in ('P0','P1')];foe=next((v for v in o['objects'] if v['owner']==other and v['zone']=='Battlefield'),None)
   if foe:
    with self.assertRaises(EngineError):g.act(o,{'kind':'ChooseTarget','target':{'Card':foe['id']}})
    self.assertEqual([g.observe(v) for v in ('P0','P1')],before)
   with self.assertRaises(EngineError):g.act(dict(o,actor=other),a)
   self.assertEqual([g.observe(v) for v in ('P0','P1')],before)
  with g.clone() as f:
   expected=f.act(f.observe(o['actor']),a);expected_other=f.observe(other)
  g.act(o,a);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.act(o,a)
  after=g.act(g.observe(o['actor']),a);expected['revision']=after['revision'];self.assertEqual(after,expected)
  seen=g.observe(other);expected_other['revision']=seen['revision'];self.assertEqual(seen,expected_other);g.drop_snapshot(saved);return after
 def test_all_ten_paid_complete_bodies_both_seats_threshold_true_and_false(self):
  for seat in ('P0','P1'):
   for positive in (False,True):
    for i,n in enumerate(NAMES):
     with self.subTest(card=n,seat=seat,celebration=positive):self.scenario(i,seat,positive)
 def scenario(self,i,seat,positive,decline=False):
  p=int(seat[1]);name=NAMES[i];deck=['Plains']*20+['Mountain']*20+[name]*16+['Novice Inspector']*16
  foe=['Forest']*70;decks=[deck,foe] if p==0 else [foe,deck]
  paid=False;qualified=False;source_ref=None;draw_before=0;boost=False;cleanup=False;discount=False;saw_zero=False;equip_paid=False
  with Game(decks,seed=41,starting_player=seat,deck_rules='Freeform') as g:
   for _ in range(14000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],name);o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};body=next((v for v in objects.values() if v['owner']==seat and v['zone']=='Battlefield' and v['card']==name),None)
    hist=o['players'][p]['nonland_permanents_entered_this_turn'];self.assertEqual(hist,g.observe('P1' if actor=='P0' else 'P0')['players'][p]['nonland_permanents_entered_this_turn'])
    if body and source_ref is None:source_ref=body['reference'];self.assertTrue(paid)
    if body and hist>=2:qualified=True
    if body and o['active']==seat:
     if not positive and o['step']==('End' if i==6 else 'Main2') and not o['stack'] and not o['announcing']:
      self.assertLess(hist,2);self.assertEqual((body['power'],body['toughness']),BASE[i]);self.assertEqual(body['counters']['plus_one'],0);self.assertIsNone(body['attached_to']);self.assertFalse(body['keywords'] & {4:1024,5:256,9:64}.get(i,0));return
     if positive and hist>=2:
      if i in (0,4,5,9):
       expected={0:(3,3),4:(2,3),5:(3,3),9:(3,3)}[i];self.assertEqual((body['power'],body['toughness']),expected)
       if i==4:self.assertTrue(body['keywords']&512 and body['keywords']&1024)
       if i==5:self.assertTrue(body['keywords']&256)
       if i==9:self.assertTrue(body['keywords']&1 and body['keywords']&64)
       return
      if i==1 and body['counters']['plus_one']>0:self.assertTrue(body['keywords']&4);return
      if i==7 and body['counters']['plus_one']>0:return
      if i==3 and body['attached_to'] and (not decline or equip_paid):
       target=next(v for v in objects.values() if v['reference']==body['attached_to']);self.assertEqual(target['owner'],seat);self.assertEqual(target['power'],3);return
      if i in (2,8):
       target=next((v for v in objects.values() if v['owner']==seat and v['zone']=='Battlefield' and v['card']=='Novice Inspector' and v['power']>1),None)
       if target:
        self.assertEqual(target['power'],2)
        if i==2:self.assertTrue(target['keywords']&32768)
        else:self.assertEqual(target['toughness'],3)
        return
      if i==6 and o['players'][p]['cards_drawn_this_turn']>1 and o['step']=='End':return
    ownlands=[v for v in objects.values() if v['owner']==seat and v['zone']=='Battlefield' and v['card'] in ('Plains','Mountain')]
    ownhand=[v for v in objects.values() if v['owner']==seat and v['zone']=='Hand']
    def score(a):
     k=a['kind'];v=objects.get(a.get('card'))
     if k in ('Cast','AnnounceCast','BeginCast'):
      if actor!=seat or o['stack'] or len(ownlands)<COST[i]+1:return -1000
      if v['card']==name:return 3000 if body is None and (not positive or hist>=2) else -1000
      if v['card']=='Novice Inspector':return 2900 if positive and hist<2 and (body or any(v['card']==name for v in ownhand)) else -1000
      return -1000
     if k=='ChooseTarget':
      if 'Card' not in a['target']:return -1000
      t=objects[a['target']['Card']];return 4000 if t['owner']==seat and t['card']=='Novice Inspector' else 3000 if t['owner']==seat else -1000
     if k=='Attack':return 2000 if i==1 and body and a['card']==body['id'] and (not positive or hist>=2) else -1000
     if k in ('Activate','AnnounceActivate') and i==3 and decline and saw_zero and not equip_paid:return 3500 if v['card']==name else -1000
     if k=='FinishTargets' and i==3 and decline and not saw_zero:return 5500
     if k in ('Activate','AnnounceActivate','ActivateMana','AnnounceMana','Block'):return -1000
     if k=='PlayLand':return 2500
     if k=='TapMana':return 4000 if o['decision']=='pay_costs' else -1000
     if k=='Discard':
      return 5000 if v['card']==name and body else 4000 if v['card']=='Novice Inspector' and not positive else 3000 if v['card']==name and sum(v['card']==name for v in ownhand)>1 else 2000 if v['card']=='Novice Inspector' and sum(v['card']=='Novice Inspector' for v in ownhand)>1 else 0
     return {'Keep':6000,'Mulligan':-1000,'Concede':-10000,'PayCosts':5000,'PutTrigger':4500,'FinishTargets':3500,'FinishAttackers':1000,'FinishBlockers':1000,'Pass':0}.get(k,100)
    a=max(enumerate(o['actions']),key=lambda z:(score(z[1]),-z[0]))[1]
    if a['kind']=='FinishTargets' and i==3 and decline and o['announcing'] and not o['announcing']['targets']:saw_zero=True
    if a['kind']=='PayCosts' and o['announcing'] and o['announcing']['source_card']==name:
     if o['announcing']['kind']=='Spell':paid=True
     elif decline:equip_paid=True;self.assertEqual(o['payment_request']['generic'],2)
    self.replay(g,o,a)
   self.fail((name,seat,positive,'no complete witness'))
 def test_bespoke_declines_optional_attach_then_really_pays_equip_both_seats(self):
  for seat in ('P0','P1'):self.scenario(3,seat,True,decline=True)
if __name__=='__main__':unittest.main()
