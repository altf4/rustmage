"""Complete live-static bodies, naturally drawn and paid in Standard games."""
import json,os,unittest
from pathlib import Path
from rustmage import Game
from test_role_consumers import RoleConsumerTests
FP=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
CASES=[('Maha, Its Feathers Night','default'),('Archon of the Wild Rose','default'),('Aettir and Priwen','life'),('Burden of Proof','detective'),('Burden of Proof','other'),('Burden of Proof','foreign_detective')]
BASICS=['Plains','Island','Swamp','Forest','Mountain']
class LiveStaticPublicTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_all_four_complete_bodies_and_conditional_branches_both_seats(self):
  for i in range(len(CASES)):
   for p in ['P0','P1']:
    with self.subTest(case=CASES[i],seat=p):self.scenario(i,p,33100+i)
 def scenario(self,i,p,seed,library=None):
  name,variant=CASES[i];q='P1'if p=='P0'else'P0';pi=int(p[-1])
  lands=['Plains']*8+['Island']*8+['Swamp']*8+['Forest']*4+['Mountain']*4
  own=lands+sum(([n]*4 for n in [name,'Bear Cub','April, Reporter of the Weird','Pacifism','Quick Study','Opt','Shock']),[])
  enemy=lands+sum(([n]*4 for n in ['Bear Cub','April, Reporter of the Weird','Quick Study','Opt','Candy Trail','Bygone Colossus','Murder']),[])
  self.assertEqual((len(own),len(enemy)),(60,60))
  with Game([own,enemy]if p=='P0'else[enemy,own],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   stage='setup';source=target=None;paid=False;effect_seen=False;frames=0;life_before=None
   for iteration in range(30000):
    head=g.observe('P0')
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,variant,p,seed,stage,iteration));self.assertTrue(paid);self.assertTrue(effect_seen)
     return dict(name=name,variant=variant,player=p,frames=frames,paid=True)
    o=g.observe(head['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(101,FP));objects={v['id']:v for v in o['objects']};field=[v for v in objects.values()if v['zone']=='Battlefield'];mine=[v for v in field if v['controller']==p];foes=[v for v in field if v['controller']==q]
    ally=next((v for v in mine if v['card']=='Bear Cub'),None);foe=next((v for v in foes if v['card']=='Bear Cub'),None);detective=next((v for v in mine if v['card']=='April, Reporter of the Weird'),None);foreign=next((v for v in foes if v['card']=='April, Reporter of the Weird'),None);body=objects.get(source)
    quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving'];main=actor==p and o['active']==p and o['step']=='Main1'
    ready=ally is not None and foe is not None and detective is not None and foreign is not None and len([v for v in mine if v['card']in BASICS])>=9
    if stage=='wait'and quiet and body is not None and body['zone']=='Battlefield':
     self.assertTrue(paid);self.assertTrue(any(v['source']==body['reference']for v in o['live_static_sources']))
     if name=='Maha, Its Feathers Night':
      self.assertEqual((foe['power'],foe['toughness']),(2,1));self.assertEqual((ally['power'],ally['toughness']),(2,2));self.assertEqual(body['keywords']&(1|256),1|256);effect_seen=True;stage='finish'
     elif name=='Archon of the Wild Rose':stage='aura'
     elif name=='Aettir and Priwen':stage='equip'
     else:
      t=objects[target];self.assertEqual((t['power'],t['toughness']),(4,4)if variant=='detective'else(1,1));effect_seen=True;stage='finish'
    if stage=='waitaura'and quiet:
     t=objects[target];self.assertEqual((t['power'],t['toughness']),(4,4));self.assertTrue(t['keywords']&1);self.assertEqual((detective['power'],detective['toughness']),(2,2));self.assertFalse(detective['keywords']&1);effect_seen=True;stage='finish'
    if stage=='waitequip'and quiet:
     self.assertEqual(body['attached_to'],objects[target]['reference']);life=o['players'][pi]['life'];self.assertEqual((objects[target]['power'],objects[target]['toughness']),(life,life));stage='shock'
    if stage=='waitshock'and quiet:
     life=o['players'][pi]['life'];self.assertEqual(life,life_before-2);self.assertEqual((objects[target]['power'],objects[target]['toughness']),(life,life));effect_seen=True;stage='finish'
    remaining={a.get('card')for a in o['actions']if a['kind']in ['Discard','Bottom']}
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and n==name and ready:return 4200
      if actor==p and stage=='aura'and n=='Pacifism'and main:return 4200
      if actor==p and stage=='shock'and n=='Shock':return 4200
      if stage=='finish':return 2700 if n in ['Bear Cub','April, Reporter of the Weird','Bygone Colossus']else -1000
      current=mine if actor==p else foes
      if n in ['Bear Cub','April, Reporter of the Weird']:return 2900 if not any(v['card']==n for v in current)else -1000
      if n in ['Quick Study','Opt']and stage=='setup':return 2800
      return -1000
     if k=='AnnounceActivate':return 4200 if actor==p and stage=='equip'and main and a['card']==source and a['ability']==0 else -1000
     if k=='ChooseTarget':
      wanted={'Player':p}if actor==p and stage=='waitshock'else{'Card':target}
      return 4300 if a['target']==wanted else -1000
     if k=='PlayLand':return 2600-20*sum(v['card']==n for v in field if v['controller']==actor)
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3000 if o['decision']in ['pay_costs','resolution_mana']else -1000
     if k in ['Discard','Bottom']:
      keep={name,'Pacifism','Shock'}if actor==p else set()
      for card in ['Bear Cub','April, Reporter of the Weird']:
       if not any(v['card']==card for v in(mine if actor==p else foes)):keep.add(card)
      if stage!='finish'and n in keep and sum(objects[x]['card']==n for x in remaining)<=1:return -1000
      return 2800 if n not in BASICS else 1600
     if k=='Attack':return 4300 if stage=='finish'else -1000
     if k=='Block':return -1000
     return {'Keep':4000,'PayCosts':4000,'ChooseSpellCost':4000,'FinishTargets':3900,'PutTrigger':3800,'FinishAttackers':3500,'FinishBlockers':3500,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,variant,p,seed,stage,o['decision'],a))
    if a['kind']=='AnnounceCast'and actor==p:
     n=objects[a['card']]['card']
     if stage=='setup'and n==name:
      source=a['card'];target=(detective if variant=='detective'else foreign if variant=='foreign_detective'else ally)['id'];stage='wait'
     elif stage=='aura'and n=='Pacifism':target=ally['id'];stage='waitaura'
     elif stage=='shock'and n=='Shock':life_before=o['players'][pi]['life'];stage='waitshock'
    if a['kind']=='AnnounceActivate'and actor==p and a['card']==source:target=ally['id'];stage='waitequip'
    if a['kind']=='PayCosts'and actor==p and o['announcing']is not None and o['announcing']['card']==source:paid=True
    self.frame(g,o,index);frames+=1
   self.fail((name,variant,p,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
