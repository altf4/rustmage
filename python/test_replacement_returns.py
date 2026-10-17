"""Paid Banishing Light return must precede Get Lost token replacement choices.

Independent reviewer189 supplied the actual-paid reproducer; author248 adds
owned replay, privacy/actor and exact-incarnation assertions.
"""
import unittest
from rustmage import Game
from test_replacements import ReplacementTests
B=['Plains','Island','Swamp','Mountain','Forest']
class ImmediateReturnTests(unittest.TestCase):
 replay=ReplacementTests.replay
 deny=ReplacementTests.deny
 def test_actual_paid_until_return_finishes_before_map_choice_both_seats(self):
  for p in ['P0','P1']:
   q='P1' if p=='P0' else 'P0';own=sum(([b]*12 for b in B),[])+['Doubling Season']*12+['Ajani, Caller of the Pride']*12+['Get Lost']*12
   enemy=sum(([b]*12 for b in B),[])+['Doubling Season']*12+['Quina, Qu Gourmet']*12+['Banishing Light']*12
   with Game([own,enemy] if p=='P0' else [enemy,own],seed=248,starting_player=p,deck_rules='Freeform',library=None) as g:
    lost=False;ajani_id=None;light_id=None;old_exiled=None
    for step in range(24000):
     r=g.observe('P0');assert r['outcome'] is None,(p,step);o=g.observe(r['actor']);a=o['actor'];objs={v['id']:v for v in o['objects']}
     def field(owner,name):return next((v for v in objs.values() if v['owner']==owner and v['zone']=='Battlefield' and v['card']==name),None)
     ajani=field(p,'Ajani, Caller of the Pride');light=field(q,'Banishing Light');held=next((v for v in objs.values() if v['owner']==p and v['zone']=='Exile' and v['card']=='Ajani, Caller of the Pride'),None)
     if lost and o['replacement_choice'] and o['replacement_choice']['kind']=='Tokens':
      seen=objs[ajani_id]
      self.assertEqual(seen['zone'],'Battlefield')
      self.assertEqual(seen['counters']['loyalty'],8)
      self.assertNotEqual(seen['reference'],old_exiled)
      self.assertEqual(a,q)
      j=next(j for j,x in enumerate(o['actions']) if x['kind']=='ChooseReplacement')
      self.deny(g,o,o['actions'][j],actor=p)
      self.replay(g,o,j)
      break
     def score(x):
      k=x['kind']
      if k=='AnnounceCast':
       n=objs[x['card']]['card']
       if a==p:
        if n=='Doubling Season':return 2000 if not field(p,n) else -1000
        if n=='Ajani, Caller of the Pride':return 1900 if field(p,'Doubling Season') and not ajani and not held and ajani_id is None else -1000
        if n=='Get Lost':return 2200 if held and light and not lost else -1000
       else:
        if n in ['Doubling Season','Quina, Qu Gourmet']:return 2000 if not field(q,n) else -1000
        if n=='Banishing Light':return 1900 if ajani and not light and field(q,'Doubling Season') and field(q,'Quina, Qu Gourmet') else -1000
       return -1000
      if k in ['Cast','BeginCast','AnnounceActivate','Activate','Attack','Mulligan','Concede']:return -1000
      if k=='ChooseTarget':
       target=x['target'].get('Card');ann=o['announcing'];name=ann['source_card'] if ann else ''
       return 2600 if name=='Banishing Light' and ajani and target==ajani['id'] or name=='Get Lost' and light and target==light['id'] else -1000
      if k=='PlayLand':return 1700-30*sum(v['owner']==a and v['zone']=='Battlefield' and v['card']==objs[x['card']]['card'] for v in objs.values())
      if k in ['TapMana','ActivateMana']:return 2300 if o['decision']=='pay_costs' else -1000
      if k=='Discard':return 1000
      return {'Keep':3000,'PayCosts':2500,'PutTrigger':2400,'FinishTargets':1900,'ChooseReplacement':2000,'Pass':0,'FinishAttackers':1000,'FinishBlockers':1000}.get(k,-1000)
     j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));x=o['actions'][j];assert score(x)>=0,(p,o['decision'],o['actions'])
     if x['kind']=='PayCosts' and o['announcing']['kind']=='Spell':
      if o['announcing']['source_card']=='Get Lost':lost=True;old_exiled=held['reference']
      if o['announcing']['source_card']=='Ajani, Caller of the Pride':ajani_id=o['announcing']['card']
     self.deny(g,o,x,actor=q if a==p else p)
     self.replay(g,o,j)
    else:raise AssertionError(('unreached',p,lost,ajani_id))
