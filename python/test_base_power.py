"""Naturally drawn, paid complete effective-base-power and casting-rule bodies."""
import json,os,unittest
from pathlib import Path
from rustmage import OBSERVATION_SCHEMA, Game,EngineError
from test_role_consumers import RoleConsumerTests
FP=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
BASICS=['Forest','Plains','Island','Mountain','Swamp']
CASES=[('Sovereign Okinec Ahau','growth'),('Kutzil, Malamet Exemplar','growth'),('Kutzil, Malamet Exemplar','base_only'),('Voice of Victory','mobilize')]
class BasePowerPublicTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_three_complete_bodies_and_both_base_power_qualification_outcomes(self):
  for i in range(len(CASES)):
   for p in ['P0','P1']:
    with self.subTest(case=CASES[i],seat=p):self.scenario(i,p,34000+i)
 def scenario(self,i,p,seed,library=None):
  name,variant=CASES[i];q='P1'if p=='P0'else'P0';pi=int(p[-1])
  lands=['Forest']*9+['Plains']*9+['Island']*10+['Mountain']*4
  own=lands+sum(([n]*4 for n in [name,'Bear Cub','April, Reporter of the Weird','Quick Study','Opt','Giant Growth','Water Wings']),[])
  enemy=lands+sum(([n]*4 for n in ['Bear Cub','April, Reporter of the Weird','Quick Study','Opt','Candy Trail','Bygone Colossus','Pacifism']),[])
  self.assertEqual((len(own),len(enemy)),(60,60))
  with Game([own,enemy]if p=='P0'else[enemy,own],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   stage='setup';source=target=None;paid=False;frames=0;effect_seen=False;blocked=False;draw_before=None;packet_seen=False;declared=False
   for iteration in range(30000):
    head=g.observe('P0')
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,variant,p,seed,stage,iteration));self.assertTrue(paid);self.assertTrue(effect_seen)
     if name!='Sovereign Okinec Ahau':self.assertTrue(blocked)
     return dict(name=name,variant=variant,player=p,frames=frames,paid=True)
    o=g.observe(head['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(OBSERVATION_SCHEMA,FP));objects={v['id']:v for v in o['objects']};field=[v for v in objects.values()if v['zone']=='Battlefield'];mine=[v for v in field if v['controller']==p];foes=[v for v in field if v['controller']==q]
    ally=next((v for v in mine if v['card']=='Bear Cub'),None);body=objects.get(source);quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving'];main=actor==p and o['active']==p and o['step']=='Main1'
    if body is not None and body['zone']=='Battlefield'and name!='Sovereign Okinec Ahau'and actor==q and o['active']==p and o['decision']=='priority':
     self.assertTrue(o['casting_prohibited']);self.assertFalse(g.observe(p)['casting_prohibited'])
     opt=next((v for v in objects.values()if v['owner']==q and v['zone']=='Hand'and v['card']=='Opt'),None)
     blue=any(v['card']=='Island'and not v['tapped']for v in foes)
     if opt is not None and blue:
      forbidden={'kind':'AnnounceCast','card':opt['id']};self.assertNotIn(forbidden,o['actions']);before=[g.observe(v)for v in ['P0','P1']]
      with self.assertRaises(EngineError):g.act(o,forbidden)
      self.assertEqual(before,[g.observe(v)for v in ['P0','P1']]);blocked=True
    if stage=='wait_body'and quiet and body is not None and body['zone']=='Battlefield':stage='await_block'if name!='Sovereign Okinec Ahau'else'buff'
    if stage=='await_block'and blocked:stage='attack'if variant=='mobilize'else'buff'
    if stage=='wait_buff'and quiet:
     self.assertEqual(objects[target]['power_above_base'],variant!='base_only');self.assertEqual(objects[target]['power'],4 if variant=='base_only'else 5);stage='attack'
    if stage=='witness':
     if name=='Kutzil, Malamet Exemplar':
      packet_seen |= any(s['card']==source and s['damage_event']is not None for s in o['stack'])
      if quiet and o['step']=='CombatDamage':
       self.assertEqual(o['players'][pi]['cards_drawn_this_turn'],draw_before+(variant!='base_only'));self.assertEqual(packet_seen,variant!='base_only');effect_seen=True;stage='finish'
     elif name=='Sovereign Okinec Ahau'and quiet:
      self.assertEqual(objects[target]['counters']['plus_one'],3);self.assertEqual(objects[target]['power'],8);effect_seen=True;stage='finish'
     elif name=='Voice of Victory'and quiet:
      tokens=[v for v in mine if v['token']and v['token']['subtypes']=='Warrior'and v['attacking']]
      self.assertEqual(len(tokens),2)
      for v in tokens:self.assertTrue(v['tapped']);self.assertEqual((v['power'],v['toughness']),(1,1))
      effect_seen=True;stage='finish'
    ready=ally is not None and len([v for v in mine if v['card']in BASICS])>=7
    discarded={a.get('card')for a in o['actions']if a['kind']in ['Discard','Bottom']}
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure','AnnounceActivate']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and n==name and ready:return 4200
      if actor==p and stage=='buff'and main and body is not None and not body['summoning_sick']and n==('Water Wings'if variant=='base_only'else'Giant Growth'):return 4200
      if stage=='finish':return 2700 if n in ['Bear Cub','April, Reporter of the Weird','Bygone Colossus']else -1000
      current=mine if actor==p else foes
      if n in ['Bear Cub','April, Reporter of the Weird']:return 2900 if not any(v['card']==n for v in current)else -1000
      if n=='Quick Study'and stage in ['setup','await_block']:return 2800
      if n=='Opt'and stage=='setup'and(actor==p or blocked):return 2800
      return -1000
     if k=='ChooseTarget':return 4300 if a['target']=={'Card':target}else -1000
     if k=='PlayLand':return 2600-20*sum(v['card']==n and v['controller']==actor for v in field)
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3000 if o['decision']in ['pay_costs','resolution_mana']else -1000
     if k in ['Discard','Bottom']:
      keep=set()
      if actor==p and stage!='finish':
       if source is None:keep.add(name)
       if stage in ['setup','wait_body','await_block','buff']:keep.add('Water Wings'if variant=='base_only'else'Giant Growth')
       if ally is None:keep.add('Bear Cub')
      if actor==q and not blocked:keep.add('Opt')
      if n in keep and sum(objects[x]['card']==n for x in discarded)<=1:return -1000
      return 2800 if n not in BASICS else 1600
     if k=='Attack':
      wanted=source if name in ['Sovereign Okinec Ahau','Voice of Victory']else target
      return 4300 if stage=='finish'or(actor==p and stage=='attack'and a['card']==wanted)else -1000
     if k=='Block':return -1000
     return {'Keep':4000,'PayCosts':4000,'ChooseSpellCost':4000,'FinishTargets':3900,'PutTrigger':3800,'FinishAttackers':3500,'FinishBlockers':3500,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,variant,p,seed,stage,o['decision'],a))
    if a['kind']=='AnnounceCast'and actor==p:
     n=objects[a['card']]['card']
     if stage=='setup'and n==name:source=a['card'];target=ally['id'];stage='wait_body'
     elif stage=='buff'and n in ['Giant Growth','Water Wings']:stage='wait_buff'
    if a['kind']=='Attack'and actor==p and stage=='attack':declared=True
    if a['kind']=='FinishAttackers'and actor==p and stage=='attack'and declared:draw_before=o['players'][pi]['cards_drawn_this_turn'];stage='witness'
    if a['kind']=='PayCosts'and actor==p and o['announcing']is not None and o['announcing']['card']==source:paid=True
    self.frame(g,o,index);frames+=1
   self.fail((name,variant,p,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
