"""Actual paid Convoke spells and complete residuals, using the in-process API."""
import unittest
from rustmage import Game,EngineError
NAMES=['Appeal to Eirdu','Disruptor of Currents','Lofty Dreams','Protective Response','Sun-Dappled Celebrant','Unexpected Assistance','Vote Out','Winter, Team Player','Merrow Skyswimmer','Bespoke Bō']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class ConvokeTests(unittest.TestCase):
 def replay(self,g,o,j):
  saved=g.snapshot();actor=o['actor']
  with g.clone() as f:expected=f.step(f.observe(actor),j)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(after,expected);g.drop_snapshot(saved);return after
 def test_all_nine_and_adjacent_equipment_paid_full_bodies_both_controllers(self):
  for i in range(10):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p)
 def test_zero_contributors_all_three_hybrid_options_and_optional_zero_targets(self):
  for p in ['P0','P1']:
   for i,hybrid,zero,target_zero in [(4,0,True,False),(8,0,False,False),(8,1,False,False),(8,2,False,False),(1,0,False,True),(9,0,False,True)]:
    with self.subTest(card=NAMES[i],seat=p,hybrid=hybrid,zero=zero,target_zero=target_zero):self.scenario(i,p,hybrid,zero,target_zero)
 def scenario(self,i,p,hybrid=0,zero=False,target_zero=False):
  name=NAMES[i];other='P1' if p=='P0' else 'P0'
  deck=sum(([b]*12 for b in BASICS),[])+[name]*12+['Savannah Lions']*16+['Aegis Turtle']*16+['Chrome Companion']*16+['Candy Trail']*12
  enemy=sum(([b]*8 for b in BASICS),[])+['Bear Cub']*16+['Candy Trail']*12
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=233+i,starting_player=p,deck_rules='Freeform') as g:
   paid=False;finished=False;equipped=False;extra=False;source=None;chosen=[];spell_targets=[];baseline=None;cast_library=None;drawn=False;discarded=False;residual_paid=False;convoked=0
   for _ in range(24000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p));o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']}
    friends=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['card'] in ['Savannah Lions','Aegis Turtle','Chrome Companion']]
    enemybody=[v for v in objects.values() if v['owner']==other and v['zone']=='Battlefield' and v['card']=='Bear Cub']
    quiet=not o['stack'] and not o['announcing'] and not o['resolving'] and o['decision']=='priority'
    if paid and quiet and not finished:
     if i in [1,9] and not target_zero:self.assertTrue(spell_targets);self.assertEqual(objects[spell_targets[0]]['zone'],'Hand')
     if i==0:
      self.assertEqual(len(spell_targets),2)
      for t in spell_targets:self.assertEqual((objects[t]['power'],objects[t]['toughness']),(baseline[t][0]+2,baseline[t][1]+1))
     if i==2:
      t=spell_targets[0];self.assertEqual((objects[t]['power'],objects[t]['toughness']),(baseline[t][0]+2,baseline[t][1]+2));self.assertTrue(objects[t]['keywords']&1);self.assertEqual(o['players'][0 if p=='P0' else 1]['library_count'],cast_library-1)
     if i in [3,6]:self.assertEqual(objects[spell_targets[0]]['zone'],'Graveyard')
     if i==4:self.assertEqual((objects[source]['power'],objects[source]['toughness']),(5,6));self.assertTrue(objects[source]['keywords']&8)
     if i==5:self.assertTrue(discarded);self.assertEqual(o['players'][0 if p=='P0' else 1]['library_count'],cast_library-3)
     if i==8:
      self.assertTrue(objects[source]['keywords']&1);self.assertTrue(objects[source]['keywords']&8)
      tokens=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['token'] and v['token']['subtypes']=='Merfolk'];self.assertEqual(len(tokens),1);self.assertEqual((tokens[0]['power'],tokens[0]['toughness'],tokens[0]['keywords']),(1,1,0));self.assertEqual(tokens[0]['token']['colors'],3)
     if i!=9:self.assertEqual(convoked,0 if zero else convoked);self.assertTrue(zero or convoked>0)
     finished=True
     if i not in [7,9]:return
    if finished and quiet:
     if i==7 and extra:
      for t,stats in baseline.items():
       if t in objects and objects[t]['zone']=='Battlefield':self.assertEqual(objects[t]['power'],stats[0]+1)
      self.assertEqual(objects[source]['power'],4);return
     if i==9 and residual_paid:
      self.assertIsNotNone(equipped);self.assertEqual((objects[equipped]['power'],objects[equipped]['toughness']),(baseline[equipped][0]+2,baseline[equipped][1]+1));self.assertTrue(objects[equipped]['keywords']&8);return
    frame=o['convoke_choice']
    if frame:
     self.assertIsNone(g.observe(other)['convoke_choice']);self.assertIsNone(g.observe(other)['payment_request'])
     before=[g.observe(v) for v in ['P0','P1']]
     for bad in [{'kind':'BeginConvoke'},{'kind':'Pass'},{'kind':'ChooseConvoker','object':frame['spell'],'contribution':'Generic'}]:
      with self.assertRaises(EngineError):g.act(o,bad)
     with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor=other),{'kind':'PayCosts','payment':[0]*6})
     for s in frame['selected']:
      self.assertFalse(objects[s['object']['card']]['tapped'])
      with self.assertRaises(EngineError):g.act(o,{'kind':'ChooseConvoker',**s})
     self.assertEqual(before,[g.observe(v) for v in ['P0','P1']])
     self.assertFalse(any(a['kind'] in ['TapMana','ActivateMana','AnnounceMana'] for a in o['actions']))
    def score(a):
     k=a['kind']
     if k in ['AnnounceCast','Cast','BeginCast']:
      n=objects[a['card']]['card']
      if actor==other:
       return 1000 if n=='Bear Cub' and len(enemybody)<2 else 900 if n=='Candy Trail' and not any(v['owner']==other and v['zone']=='Battlefield' and v['card']==n for v in objects.values()) else -1000
      if finished and i==7 and not extra:return 1800 if n=='Candy Trail' else -1000
      if n==name:
       ready=i!=8 or all(sum(v['owner']==p and v['zone']=='Battlefield' and v['card']==b and not v['tapped'] for v in objects.values())>=2 for b in ['Plains','Island']) and sum(v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS for v in objects.values())>=6
       return 1500 if ready and not paid and len(friends)>=3 and k=='AnnounceCast' else -1000
      return 1100 if n in ['Savannah Lions','Aegis Turtle','Chrome Companion'] and len(friends)<4 else -1000
     if k in ['AnnounceActivate','Activate','BeginActivate']:return 1900 if actor==p and finished and i==9 and not residual_paid and a['card']==source and k=='AnnounceActivate' else -1000
     if k=='ChooseTarget':
      t=a['target'].get('Card');v=objects.get(t)
      if v is None:return -1000
      if finished and i==9:return 2000 if v in friends else -1000
      ann=o['announcing'];src=ann['source_card'] if ann else None
      if src!=name:return 1000
      if i in [1,9]:return -1000 if target_zero else 2100 if v['owner']==other and v['card']=='Candy Trail' else 2000 if v['owner']==other else 1000
      if i in [0,2]:return 2000 if v in friends else 1000
      return 2100 if v['owner']==other else 1000
     if k=='FinishTargets':return 2200 if target_zero and i in [1,9] else 1000
     if k=='ChooseSpellCost':return 2100 if a['cost']==hybrid else -1000
     if k=='BeginConvoke':return -1000 if zero else 2100
     if k=='ChooseConvoker':return 2100 if not frame['selected'] else 1900
     if k=='PayCosts':return 2000
     if k=='TapMana':return 1900 if o['decision']=='pay_costs' else -1000
     if k=='Attack':return 1500 if actor==other and i==3 and not paid and len(friends)>=3 else -1000
     if k=='PlayLand':return 1300-20*sum(v['owner']==actor and v['zone']=='Battlefield' and v['card']==objects[a['card']]['card'] for v in objects.values())
     return {'Keep':2500,'Mulligan':-10000,'Concede':-10000,'Pass':0,'FinishAttackers':1000,'FinishBlockers':1000,'PutTrigger':2100,'Discard':2000,'ArrangeLibrary':2000}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,o['decision'],o['actions']))
    if a['kind']=='ChooseTarget' and o['announcing'] and o['announcing']['source_card']==name and not (finished and i==9):spell_targets.append(a['target']['Card'])
    if a['kind']=='Discard' and i==5:discarded=True
    ann=o['announcing']
    if a['kind']=='PayCosts' and ann and ann['source_card']==name and ann['kind']=='Spell':
     paid=True;source=ann['card'];chosen=frame['selected'] if frame else [];convoked=len(chosen);baseline={v['id']:(v['power'],v['toughness']) for v in objects.values()};cast_library=o['players'][0 if p=='P0' else 1]['library_count']
    if a['kind']=='PayCosts' and finished and ann and i==7 and ann['kind']=='Spell' and ann['controller']==p and ann['source_card']=='Candy Trail':extra=True;baseline={v['id']:(v['power'],v['toughness']) for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['card'] in [name,'Savannah Lions','Aegis Turtle','Chrome Companion']}
    if a['kind']=='PayCosts' and finished and ann and i==9 and ann['source_card']==name and ann['kind']!='Spell':residual_paid=True;equipped=ann['targets'][0]['target']['Card']
    after=self.replay(g,o,j);self.assertEqual(after['schema'],90)
    if a['kind']=='PayCosts' and chosen:
     afterobjs={v['id']:v for v in after['objects']}
     for s in chosen:self.assertTrue(afterobjs[s['object']['card']]['tapped'])
     chosen=[]
   self.fail('no complete Convoke continuation '+str((name,p,paid,finished,source)))
if __name__=='__main__':unittest.main()
