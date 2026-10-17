"""Complete paid Standard Blessing and actual draw-step conditional witnesses."""
import json,os,unittest
from pathlib import Path
from rustmage import Game
import test_role_consumers as roles
NAME="Lord Skitter's Blessing"
EXPECTED=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
class EnchantedDrawTests(unittest.TestCase):
 frame=roles.RoleConsumerTests.frame
 def test_complete_paid_role_then_additional_draw_both_seats(self):
  for p in ['P0','P1']:
   with self.subTest(player=p):self.scenario(p,False,2960)
 def test_paid_role_destroyed_before_draw_step_prevents_trigger_both_seats(self):
  for p in ['P0','P1']:
   with self.subTest(player=p):self.scenario(p,True,2961)
 def scenario(self,p,remove,seed,library=None):
  q='P1' if p=='P0' else 'P0'
  deck=['Forest']*12+['Swamp']*12+['Plains']*8+sum(([n]*4 for n in [NAME,'Llanowar Elves','Bear Cub','Disenchant','Candy Trail','Chrome Companion','Bygone Colossus']),[])
  self.assertEqual(len(deck),60);enemy=['Forest']*60
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY')) as g:
   source=target=aura=None;stage='setup';capture=None;proved=False;frames=0;role_seen=False;paid=0;destroyed=False
   for iteration in range(25000):
    o=g.observe('P0')
    if o['outcome'] is not None:
     self.assertTrue(proved,(p,remove,stage,iteration));return dict(player=p,remove=remove,frames=frames,role_seen=role_seen,paid=paid,destroyed=destroyed)
    o=g.observe(o['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(92,EXPECTED));objects={v['id']:v for v in o['objects']};private=g.observe(p);hand=[v for v in private['objects'] if v['zone']=='Hand' and v['owner']==p];field=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p];ally=next((v for v in field if v['card']=='Llanowar Elves'),None);quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving'];ann=o['announcing']
    if stage=='wait_role' and quiet:
     token=next((v for v in objects.values() if v['zone']=='Battlefield' and v['token'] and v['token']['name']=='Wicked' and v['attached_to']==objects[target]['reference']),None)
     if token:
      role_seen=True;aura=token['id'];self.assertEqual(objects[target]['power'],2);stage='destroy' if remove else 'wait_draw'
    if stage=='wait_destroy' and quiet and objects.get(aura,{}).get('zone')!='Battlefield':
     self.assertEqual(paid,2);self.assertFalse(any(v['aura']['card']==aura for v in o['role_timestamps']));destroyed=True;stage='wait_absent_draw'
    draw_trigger=bool(source is not None and any(s['card']==source for s in o['stack']) and o['step']=='Draw')
    if stage=='wait_draw' and draw_trigger and capture is None:capture=(len(hand),private['players'][int(p[-1])]['life']);stage='resolving_draw'
    if stage=='resolving_draw' and quiet:
     self.assertEqual(len(hand),capture[0]+1);self.assertEqual(private['players'][int(p[-1])]['life'],capture[1]-1);proved=True;stage='finish'
    if stage=='wait_absent_draw' and o['step']=='Upkeep' and o['active']==p and quiet and capture is None:capture=(len(hand),private['players'][int(p[-1])]['life'])
    if stage=='wait_absent_draw' and o['step']=='Draw' and o['active']==p:
     self.assertFalse(draw_trigger)
     if quiet and capture is not None:
      self.assertEqual(len(hand),capture[0]+1);self.assertEqual(private['players'][int(p[-1])]['life'],capture[1]);proved=True;stage='finish'
    def score(a):
     k=a['kind'];obj=objects.get(a.get('card'),{});name=obj.get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate']:return -2000
     if k=='AnnounceCast':
      if actor!=p:return -1000
      if stage=='setup' and name==NAME and ally and (not remove or any(v['card']=='Disenchant' for v in hand)):return 3100
      if stage=='setup' and name=='Llanowar Elves':return 2800 if not ally else -1000
      if stage=='destroy' and name=='Disenchant':return 3100
      return 2700 if stage=='finish' and name in ['Bear Cub','Bygone Colossus','Chrome Companion','Llanowar Elves'] else -1000
     if k=='ChooseTarget':return 3100 if a['target'].get('Card')==(aura if stage=='wait_destroy' else target) else -1000
     if k=='PlayLand':return 2300
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 2800 if o['decision'] in ['pay_costs','resolution_mana'] else -1000
     if k in ['Discard','Bottom']:
      if actor==p and stage=='setup' and name in [NAME,'Disenchant','Llanowar Elves']:return -1000
      return 2400 if name in ['Candy Trail','Chrome Companion','Bygone Colossus'] else 1200
     if k=='Attack':return 3300 if stage=='finish' else -1000
     if k in ['AnnounceActivate','Block']:return -1000
     return {'Keep':3500,'PayCosts':3400,'ChooseSpellCost':3300,'FinishTargets':3200,'PutTrigger':3200,'FinishAttackers':3000,'FinishBlockers':3000,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda i:score(o['actions'][i]));action=o['actions'][index];self.assertGreaterEqual(score(action),0,(p,remove,stage,o['decision'],o['actions']))
    if actor==p and action['kind']=='AnnounceCast':
     if objects[action['card']]['card']==NAME:source=action['card'];target=ally['id'];stage='wait_role'
     if objects[action['card']]['card']=='Disenchant':stage='wait_destroy'
    if actor==p and action['kind']=='PayCosts' and ann and ann['source_card'] in [NAME,'Disenchant']:paid+=1
    self.frame(g,o,index);frames+=1
   self.fail((p,remove,stage,'no natural terminal game'))
if __name__=='__main__':unittest.main()
