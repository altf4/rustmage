"""Paid simultaneous returns in naturally drawn, legal Standard games."""
import json,os,unittest
from pathlib import Path
from rustmage import Game
from test_role_consumers import RoleConsumerTests
FP=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class BattlefieldReturnTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_complete_returns_both_seats(self):
  for name in ['Aetherize',"River's Rebuke"]:
   for seat in ['P0','P1']:
    with self.subTest(name=name,seat=seat):self.scenario(name,seat,30600 if name=='Aetherize' else 30601)
 def scenario(self,name,p,seed,library=None):
  q='P1' if p=='P0' else 'P0'
  deck=['Plains']*6+['Island']*6+['Swamp']*4+['Mountain']*4+['Forest']*4+sum(([n]*4 for n in [name,'Bear Cub','Chrome Companion','Bygone Colossus','Candy Trail','Basilisk Collar','Agent of Atlas','Common Crook','Llanowar Elves']),[])
  enemy=['Forest']*24+['Plains']*4+['Mountain']*4+sum(([n]*4 for n in ['Bear Cub','Basilisk Collar','Chrome Companion','Candy Trail','Llanowar Elves','Bygone Colossus','Disenchant']),[])
  self.assertEqual((len(deck),len(enemy)),(60,60))
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY')) as g:
   stage='setup';source=None;group=[];frames=paid=0
   for iteration in range(30000):
    head=g.observe('P0')
    if head['outcome'] is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertGreater(paid,0);return dict(name=name,player=p,frames=frames,paid=paid)
    o=g.observe(head['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(101,FP));objects={v['id']:v for v in o['objects']};own=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p];foes=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==q];hand=[v for v in g.observe(p)['objects'] if v['zone']=='Hand' and v['owner']==p];quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    if stage=='wait' and quiet:
     self.assertEqual(objects[source]['zone'],'Graveyard')
     for before in group:
      after=objects.get(before['id']);self.assertIsNotNone(after);self.assertEqual(after['zone'],'Hand',(name,before,after));self.assertEqual(after['controller'],before['owner']);self.assertNotEqual(after['reference'],before['reference'])
     self.assertTrue(group);self.assertGreater(paid,0);stage='finish'
    attackers=set(o['attackers'])
    ready=(bool(attackers) and o['active']==q) if name=='Aetherize' else any(v['card']=='Bear Cub' for v in foes) and any(v['card']=='Basilisk Collar' for v in foes)
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure']:return -2000
     if k=='AnnounceCast':
      if actor==p:
       if stage=='setup' and ready and n==name:return 3600
       if stage=='finish' and n in ['Bear Cub','Chrome Companion','Bygone Colossus','Llanowar Elves','Agent of Atlas','Common Crook']:return 2600
       return -1000
      if stage=='finish':return -1000
      if stage=='setup' and n in ['Bear Cub','Basilisk Collar'] and not any(v['card']==n for v in foes):return 2800
      return -1000
     if k=='AnnounceActivate':return -1000
     if k=='ChooseTarget':return 3700 if a['target'].get('Player')==q else -1000
     if k=='PlayLand':return 2200-20*sum(v['card']==n and v['controller']==actor and v['zone']=='Battlefield' for v in objects.values())
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3000 if o['decision'] in ['pay_costs','resolution_mana'] else -1000
     if k in ['Discard','Bottom']:
      if actor==p and stage=='setup' and n==name and sum(v['card']==name for v in hand)<=1:return -1000
      return 2500 if n in ['Candy Trail','Basilisk Collar','Chrome Companion'] else 1200
     if k=='Attack':return 3600 if (stage=='finish' and actor==p) or (stage=='setup' and name=='Aetherize' and actor==q and sum(v['card']=='Island' and not v['tapped'] for v in own)>=1 and sum(v['card'] in BASICS and not v['tapped'] for v in own)>=4 and any(v['card']==name for v in hand)) else -1000
     if k=='Block':return -1000
     return {'Keep':3900,'PayCosts':3800,'ChooseSpellCost':3800,'FinishTargets':3500,'PutTrigger':3400,'FinishAttackers':3000,'FinishBlockers':3000,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda i:score(o['actions'][i]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision']))
    if actor==p and a['kind']=='AnnounceCast' and objects[a['card']]['card']==name:
     source=a['card'];stage='wait';group=[v.copy() for v in objects.values() if v['zone']=='Battlefield' and ((v['id'] in attackers) if name=='Aetherize' else v['controller']==q and v['card'] not in BASICS)]
    if actor==p and a['kind']=='PayCosts':paid+=1
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
