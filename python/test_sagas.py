"""Paid full Saga bodies and real chapter turns through the public library."""
import unittest
from rustmage import Game,EngineError
import test_crew
NAMES=['Summon: Choco/Mog','Summon: Esper Ramuh','Summon: Fat Chocobo','Summon: G.F. Ifrit','Burn, Burn, Tree and Fern','Summon: Anima','Leaves from the Vine','Summon: Primal Garuda','Welcome to Sweettooth',"The Huntsman's Redemption",'Roar of Endless Song','Summon: Shiva']
B=['Forest','Plains','Island','Mountain','Swamp']
class SagaTests(unittest.TestCase):
 replay=test_crew.CrewTests.replay
 def test_all_twelve_paid_full_chapters_in_both_seats(self):
  for i,n in enumerate(NAMES):
   for p in ['P0','P1']:
    with self.subTest(card=n,seat=p):self.scenario(i,p)
 def scenario(self,i,p):
  name=NAMES[i];q='P1'if p=='P0'else'P0';pi=int(p[-1]);own=sum(([b]*12 for b in B),[])+[name]*12+['Bear Cub']*24+['Thrill of Possibility']*12+['Firebending Lesson']*12
  enemy=sum(([b]*12 for b in B),[])+['Hill Giant']*12+['Candy Trail']*12+['Llanowar Elves']*12
  with Game([own,enemy]if p=='P0'else[enemy,own],seed=256+i,starting_player=p,deck_rules='Freeform')as g:
   source=None;paid=False;thrill=False;entered=False;seen=set();base=None;last=None;held_frames=0;discarded=0
   for step in range(24000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,step));o=g.observe(o['actor']);a=o['actor'];objs={v['id']:v for v in o['objects']};ownview=g.observe(p)
    def field(owner,n):return next((v for v in objs.values()if v['owner']==owner and v['zone']=='Battlefield'and v['card']==n),None)
    body=next((v for v in ownview['objects']if v['id']==source),None)
    if paid and body and body['zone']=='Battlefield':
     entered=True;seen.add(int(body['counters'].get('lore',0)));self.assertTrue(body['modified']);last=body
    if entered and body and body['zone']=='Graveyard':
     self.assertEqual(seen,set(range(1,5 if i in [0,2,3,4,5]else 4)));self.assertGreater(held_frames,0);self.assertEqual(ownview['schema'],103)
     if i==5:self.assertEqual(ownview['players'][pi]['life'],base['players'][pi]['life']-3);self.assertEqual(ownview['players'][1-pi]['life'],base['players'][1-pi]['life']-3)
     if i==3:self.assertGreater(discarded,0)
     print('completed Saga',name,p,'steps',step,'chapters',sorted(seen),flush=True)
     return
    ann=o['announcing'];quiet=o['decision']=='priority'and not o['stack']and not ann and not o['resolving']
    own_bears=[v for v in objs.values()if v['owner']==p and v['zone']=='Battlefield'and v['card']=='Bear Cub']
    def score(x):
     k=x['kind'];n=objs.get(x.get('card'),{}).get('card','')
     if k=='AnnounceCast':
      if a==p:
       if n=='Bear Cub':return 1900 if len(own_bears)<3 and not paid else -1000
       if n=='Thrill of Possibility':return 2200 if not thrill and any(v['owner']==p and v['zone']=='Hand'and v['card']=='Firebending Lesson'for v in objs.values())else -1000
       if n==name:return 2300 if not paid and thrill and len(own_bears)>=3 and field(q,'Hill Giant')and field(q,'Candy Trail')and(i!=7 or field(q,'Llanowar Elves'))else -1000
      else:
       if n in ['Hill Giant','Candy Trail','Llanowar Elves']:return 1800 if not field(q,n)else -1000
      return -1000
     if k in ['Cast','BeginCast','AnnounceActivate','Activate','Mulligan','Concede']:return -1000
     if k=='Attack':return -1000
     if k=='ChooseTarget':
      t=x['target'].get('Card');return 2400 if t in objs and ((ann and ann['source_card']==name and i==4 and objs[t]['owner']==q)or (objs[t]['owner']==p and objs[t]['card']=='Bear Cub'))else 1500
     if k=='ChooseAdditionalCost':return 2400 if objs[x['object']['card']]['card']=='Firebending Lesson'else 0
     if k=='Sacrifice':return 2400 if objs[x['object']['card']]['card']=='Bear Cub'else 1500
     if k=='ChooseSearch':return 2100 if n in B else 1900
     if k=='ArrangeLibrary':return 1800 if x['destination']=='Top'else 1000
     if k=='Discard':return 1800 if n in B else 1000
     if k=='PlayLand':return 1700-50*sum(v['owner']==a and v['zone']=='Battlefield'and v['card']==n for v in objs.values())
     if k in ['TapMana','ActivateMana']:return 2300 if o['decision']=='pay_costs'else 2000 if i==7 and a==q and n=='Llanowar Elves'else -1000
     return {'Keep':3000,'PayCosts':2500,'ChooseSpellCost':2100,'PutTrigger':2000,'FinishTargets':1800,'FinishSearch':1800,'ChooseReplacement':1900,'Pass':0,'FinishAttackers':1200,'FinishBlockers':1200,'DeclineEffectPayment':500}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));x=o['actions'][j];self.assertGreaterEqual(score(x),0,(name,p,o['decision'],o['actions']))
    if x['kind']=='PayCosts'and ann and ann['kind']=='Spell':
     if ann['source_card']==name:paid=True;source=ann['card'];base=ownview
     if ann['source_card']=='Thrill of Possibility':thrill=True
    if entered and any(v['source_card']==name for v in o['stack']):held_frames+=1
    if x['kind']=='Discard'and o['resolving']and o['resolving']['source_card']==name:discarded+=1
    if o['effect_choice']and o['effect_choice']['kind']=='Discard':
     self.assertIsNone(g.observe(q if a==p else p)['effect_choice'])
    before=[g.observe(t)for t in ['P0','P1']]
    with self.assertRaises(EngineError):g.act(dict(o,actor=q if a==p else p),x)
    self.assertEqual(before,[g.observe(t)for t in ['P0','P1']])
    self.replay(g,o,j)
   self.fail(('unreached',name,p,paid,entered,seen))
if __name__=='__main__':unittest.main()
