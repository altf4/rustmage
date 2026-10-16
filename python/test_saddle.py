"""Legal paid Saddle costs, exact designations and complete attack bodies."""
import unittest
from rustmage import Game,EngineError
NAMES=['Alacrian Jaguar', 'Autarch Mammoth', 'Bounding Felidar', 'Bridled Bighorn', 'Brightfield Glider', 'Brightfield Mustang', 'Congregation Gryff', 'District Mascot', 'Dracosaur Auxiliary', 'Drover Grizzly', 'Gila Courser', 'Gilded Ghoda', 'Gloryheath Lynx', 'Quilled Charger', 'Seraphic Steed', 'Stubborn Burrowfiend', 'Trained Arynx', 'Unswerving Sloth', 'Venomsac Lagac']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class SaddleTests(unittest.TestCase):
 def replay(self,g,o,j):
  saved=g.snapshot();actor=o['actor']
  with g.clone() as f:expected=f.step(f.observe(actor),j)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(after,expected);g.drop_snapshot(saved);return after
 def test_all_nineteen_actual_paid_cast_cost_resolution_and_attack_in_both_seats(self):
  for i in range(19):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p)
 def test_paid_district_residual_and_actual_gila_exiled_card_play(self):
  for p in ['P0','P1']:
   for i in [7,10]:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,residual=True)
 def scenario(self,i,p,residual=False):
  name=NAMES[i];other='P1' if p=='P0' else 'P0'
  deck=sum(([b]*16 for b in BASICS),[])+[name]*16+['Bear Cub']*24+['Chrome Companion']*16+['Candy Trail']*16
  enemy=sum(([b]*16 for b in BASICS),[])+['Bear Cub']*24+['Candy Trail']*16
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=230+i,starting_player=p,deck_rules='Freeform') as g:
   paid=False;source=None;saddle_paid=0;marked=False;attacked=False;trigger=None;done=False;destroyed=None;extra_paid=False
   for _ in range(24000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p));o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']}
    friends=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['card'] in ['Bear Cub','Chrome Companion']]
    body=next((v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['card']==name),None)
    if body and body['saddled']:marked=True
    if trigger is not None and not any(s['id']==trigger for s in o['stack']) and not o['announcing'] and not o['resolving'] and o['decision']=='priority':
     self.assertTrue(marked);self.assertGreater(saddle_paid,0);self.assertTrue(attacked or i==15)
     done=True
     if not residual:return
     if i==7 and destroyed is not None and objects[destroyed]['zone']=='Graveyard' and not o['stack'] and not o['announcing'] and not o['resolving']:
      self.assertIsNone(body);return
     if i==10 and extra_paid:return
    f=o['saddle_choice']
    if f:
     self.assertEqual(actor,p);self.assertIsNone(g.observe(other)['saddle_choice']);self.assertIsNone(o['payment_request'])
     before=[g.observe(v) for v in ['P0','P1']]
     with self.assertRaises(EngineError):g.act(dict(o,actor=other),{'kind':'FinishSaddle'})
     self.assertEqual(before,[g.observe(v) for v in ['P0','P1']])
     self.assertEqual(f['selected_power'],sum(objects[r['card']]['power'] for r in f['selected']))
     for r in f['selected']:
      self.assertFalse(objects[r['card']]['tapped']);self.assertEqual(objects[r['card']]['reference'],r)
      with self.assertRaises(EngineError):g.act(o,{'kind':'ChooseSaddler','object':r})
     with self.assertRaises(EngineError):g.act(o,{'kind':'ChooseSaddler','object':f['source']})
     if f['selected']:
      stale=dict(f['selected'][0]);stale['generation']+=1
      with self.assertRaises(EngineError):g.act(o,{'kind':'ChooseSaddler','object':stale})
     if not f['can_finish']:
      with self.assertRaises(EngineError):g.act(o,{'kind':'FinishSaddle'})
    def score(a):
     k=a['kind']
     if k in ['AnnounceCast','Cast','AnnounceExile']:
      n=objects[a['card']]['card']
      if actor==other:return 800 if n=='Candy Trail' and not any(v['owner']==other and v['zone']=='Battlefield' and v['card']==n for v in objects.values()) else -1000
      if residual and i==10 and done and objects[a['card']]['zone']=='Exile':return 1600 if k=='AnnounceExile' else -1000
      if n==name:return 1200 if not paid and len(friends)>=3 and k=='AnnounceCast' else -1000
      if n in ['Bear Cub','Chrome Companion']:return 1000 if len(friends)<3 else -1000
      return -1000
     if k in ['AnnounceActivate','Activate','BeginActivate']:
      if residual and i==7 and done and actor==p and body and a['card']==body['id'] and k=='AnnounceActivate' and a['ability']==0:return 2000
      return 1500 if not done and actor==p and body and a['card']==body['id'] and k=='AnnounceActivate' and a['ability']==(1 if i==7 else 0) and not body['saddled'] else -1000
     if k=='ChooseSaddler':return 1600+objects[a['object']['card']]['power']
     if k=='FinishSaddle':return 2000
     if k=='ChooseTarget':
      t=a['target']
      if residual and i==7 and done and 'Card' in t:return 2100 if objects[t['Card']]['card']=='Candy Trail' and objects[t['Card']]['owner']==other else 1000
      return 1900 if t.get('Player')==other else 1800 if t.get('Card') in [v['id'] for v in friends] else 1000
     if k=='Attack':return 1400 if actor==p and body and a['card']==body['id'] and body['saddled'] and not attacked else -1000
     if k=='TapMana':return 1600 if o['decision']=='pay_costs' else -1000
     if k=='PlayLand':
      if residual and i==10 and done and objects[a['card']]['zone']=='Exile':return 2000
      n=objects[a['card']]['card'];return 1300-20*sum(v['owner']==actor and v['zone']=='Battlefield' and v['card']==n for v in objects.values())
     return {'Keep':2000,'Mulligan':-10000,'Concede':-10000,'PayCosts':1900,'Pass':0,'FinishAttackers':700,'FinishBlockers':700,'PutTrigger':1600,'FinishTargets':1400,'ArrangeLibrary':1500,'ChooseSearch':1500,'FinishSearch':1200,'Discard':1300}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j]
    self.assertGreaterEqual(score(a),0,(name,p,o['decision'],o['actions']))
    if a['kind']=='PayCosts' and o['announcing'] and o['announcing']['source_card']==name and o['announcing']['kind']=='Spell':paid=True;source=o['announcing']['object']
    if residual and i==7 and done and a['kind']=='ChooseTarget':destroyed=a['target'].get('Card')
    if residual and i==10 and done and ((a['kind']=='PlayLand' and objects[a['card']]['zone']=='Exile') or (a['kind']=='PayCosts' and o['announcing'] and o['announcing']['cast'] and o['announcing']['cast']['origin']=='Exile')):extra_paid=True
    if a['kind']=='FinishSaddle':saddle_paid+=1
    if a['kind']=='Attack':attacked=True
    for s in o['stack']:
     if s['source_card']==name and isinstance(s['kind'],dict) and 'TriggeredAbility' in s['kind'] and (attacked or i==15 and marked):trigger=s['id']
    self.replay(g,o,j)
   self.fail('no complete saddle continuation '+str((name,p,paid,saddle_paid,marked,attacked,trigger)))
if __name__=='__main__':unittest.main()
