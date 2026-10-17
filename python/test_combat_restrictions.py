"""Complete combat-restriction bodies in naturally drawn legal Standard games."""
import json,os,unittest
from pathlib import Path
from rustmage import Game
from test_role_consumers import RoleConsumerTests
FP=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
NAMES=['Pacifism','Mouser Mark III','Tiger-Dillo','Technodrome','The Lion-Turtle']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class CombatRestrictionTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_all_five_complete_paid_bodies_both_seats(self):
  for i,name in enumerate(NAMES):
   for p in ['P0','P1']:
    with self.subTest(name=name,seat=p):self.scenario(name,p,30800+i)
 def scenario(self,name,p,seed,library=None):
  q='P1' if p=='P0' else 'P0';pi=int(p[-1])
  deck=['Plains']*6+['Island']*6+['Swamp']*4+['Mountain']*4+['Forest']*4+sum(([n]*4 for n in [name,'Bear Cub','Chrome Companion','Bygone Colossus','Candy Trail','Disenchant','Abandon Attachments','Fancy Footwork','Llanowar Elves']),[])
  enemy=['Forest']*24+['Plains']*4+['Mountain']*4+sum(([n]*4 for n in ['Bear Cub','Basilisk Collar','Chrome Companion','Candy Trail','Llanowar Elves','Bygone Colossus','Disenchant']),[])
  self.assertEqual((len(deck),len(enemy)),(60,60))
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY')) as g:
   stage='setup';source=target=candy=spell=None;frames=paid=0;attack_ban=block_ban=False;life=drawn=mana_before=None;mana_paid=False;source_paid=False
   for iteration in range(35000):
    head=g.observe('P0')
    if head['outcome'] is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertTrue(source_paid);self.assertGreater(paid,0);return dict(name=name,player=p,frames=frames,paid=paid)
    o=g.observe(head['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(92,FP));objects={v['id']:v for v in o['objects']};own=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p];foes=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==q];hand=[v for v in g.observe(p)['objects'] if v['zone']=='Hand' and v['owner']==p];body=objects.get(source);ally=next((v for v in own if v['card']=='Bear Cub'),None);foe=next((v for v in foes if v['card']=='Bear Cub'),None);artifact=next((v for v in own if v['card']=='Candy Trail'),None);quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving'];main=actor==p and o['active']==p and o['step']=='Main1'
    lessons=[v for v in objects.values() if v['zone']=='Graveyard' and v['owner']==p and v['card'] in ['Abandon Attachments','Fancy Footwork']]
    if stage=='waitbody' and quiet and body is not None and body['zone']=='Battlefield':
     self.assertTrue(source_paid);self.assertEqual((body['owner'],body['controller']),(p,p))
     if name=='Pacifism':self.assertEqual(body['attached_to'],objects[target]['reference']);self.assertFalse(objects[target]['keywords']&16)
     if name=='Technodrome':self.assertEqual(body['keywords']&(2|256),2|256)
     if name=='The Lion-Turtle':self.assertEqual(o['players'][pi]['life'],life+3);self.assertEqual(body['keywords']&(2|8),2|8)
     stage='enable'
    if stage=='waitactivation' and quiet:
     self.assertEqual(objects[candy]['zone'],'Graveyard');self.assertEqual(o['players'][pi]['cards_drawn_this_turn'],drawn+1);self.assertEqual(body['counters']['plus_one'],1);self.assertTrue(body['tapped']);stage='footwork'
    if stage=='waitspell' and quiet:
     self.assertEqual(objects[spell]['zone'],'Graveyard');stage='enable'
    if stage=='waitmana' and quiet:
     self.assertTrue(body['tapped']);self.assertEqual(sum(o['players'][pi]['mana']),mana_before+1);mana_paid=True;stage='enable'
    if body is not None and body['zone']=='Battlefield' and stage not in ['setup','waitbody','finish']:
     if name=='Pacifism':
      recipient=objects[target]
      if actor==q and o['decision']=='attackers' and not recipient['tapped'] and not recipient['summoning_sick']:
       self.assertFalse(any(a['kind'] in ['Attack','AttackPlaneswalker'] and a['card']==target for a in o['actions']));attack_ban=True
      if actor==q and o['decision']=='blockers' and o['attackers']:
       self.assertFalse(any(a['kind']=='Block' and a['blocker']==target for a in o['actions']));block_ban=True
      if attack_ban and block_ban:stage='finish'
     else:
      enabled=(artifact is not None if name=='Mouser Mark III' else any(v['id']!=source and v.get('power',0)>=4 and v['kinds']&1 for v in own) if name=='Tiger-Dillo' else body['power']>=6 if name=='Technodrome' else len(lessons)>=3 and mana_paid)
      if enabled:stage='positive'
      elif stage=='positive':stage='enable'
      if actor==p and o['decision']=='attackers' and not body['tapped'] and not body['summoning_sick'] and not enabled:
       self.assertFalse(any(a['kind'] in ['Attack','AttackPlaneswalker'] and a['card']==source for a in o['actions']))
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure']:return -2000
     if k=='AnnounceCast':
      if actor==q:return 2800 if stage!='finish' and n=='Bear Cub' and foe is None else -1000
      if stage=='setup' and n==name and foe is not None and (name!='Pacifism' or ally is not None):return 3600
      if stage=='setup' and name=='Pacifism' and n=='Bear Cub' and ally is None:return 2800
      if stage=='enable' and name=='Mouser Mark III' and n=='Candy Trail':return 3400
      if stage=='enable' and name=='Tiger-Dillo' and n=='Bear Cub' and ally is None:return 3400
      if stage in ['enable','footwork'] and name in ['Tiger-Dillo','Technodrome'] and n=='Fancy Footwork' and main and ((name=='Tiger-Dillo' and ally is not None and not body['summoning_sick']) or (name=='Technodrome' and body['counters']['plus_one']>=1)):return 3500
      if stage=='enable' and name=='Technodrome' and n=='Candy Trail' and artifact is None:return 3400
      if stage=='enable' and name=='The Lion-Turtle' and len(lessons)<3 and n=='Abandon Attachments':return 3400
      if stage=='finish' and n in ['Bear Cub','Chrome Companion','Bygone Colossus','Llanowar Elves']:return 2800
      return -1000
     if k=='AnnounceActivate':return 3600 if actor==p and stage=='enable' and name=='Technodrome' and a['card']==source and a['ability']==0 and artifact is not None else -1000
     if k=='AnnounceMana' and actor==p and name=='The Lion-Turtle' and stage=='enable' and a['card']==source and not mana_paid:return 3700
     if k=='Sacrifice':return 3900 if a['object']['card']==candy else -1000
     if k=='ChooseTarget':
      wanted=target if name=='Pacifism' else ally['id'] if name=='Tiger-Dillo' and ally is not None else source
      return 3700 if a['target'].get('Card')==wanted else -1000
     if k=='ChooseOptional':return 3800 if not a['accept'] else -1000
     if k=='PlayLand':return 2200-20*sum(v['card']==n and v['controller']==actor and v['zone']=='Battlefield' for v in objects.values())
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3000 if o['decision'] in ['pay_costs','resolution_mana'] else -1000
     if k in ['Discard','Bottom']:
      if actor==p and stage!='finish' and n=='Bear Cub' and name in ['Tiger-Dillo','Pacifism'] and ally is None and sum(v['card']==n for v in hand)<=1:return -1000
      if actor==p and stage!='finish' and n=='Candy Trail' and name in ['Mouser Mark III','Technodrome'] and artifact is None and sum(v['card']==n for v in hand)<=1:return -1000
      if actor==p and stage!='finish' and ((n==name and sum(v['card']==name for v in hand)<=1) or (name=='The Lion-Turtle' and n in ['Abandon Attachments','Fancy Footwork'] and len(lessons)<3) or (name in ['Tiger-Dillo','Technodrome'] and n=='Fancy Footwork' and sum(v['card']==n for v in hand)<=1)):return -1000
      return 2500 if n in ['Disenchant','Chrome Companion','Candy Trail'] else 1200
     if k=='Attack':return 3600 if actor==p and (stage=='finish' or (stage=='positive' and a['card']==source) or (name=='Pacifism' and stage=='enable' and ally is not None and a['card']==ally['id'])) else -1000
     if k=='Block':return -1000
     return {'Keep':3900,'PayCosts':3800,'ChooseSpellCost':3800,'FinishTargets':3500,'PutTrigger':3400,'FinishSearch':3400,'FinishAttackers':3000,'FinishBlockers':3000,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda i:score(o['actions'][i]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision']))
    if actor==p and a['kind']=='AnnounceCast':
     if objects[a['card']]['card']==name:source=a['card'];target=foe['id'] if name=='Pacifism' else None;life=o['players'][pi]['life'];stage='waitbody'
     elif objects[a['card']]['card'] in ['Fancy Footwork','Abandon Attachments']:spell=a['card'];stage='waitspell'
    if actor==p and a['kind']=='AnnounceActivate' and a['card']==source:candy=artifact['id'];drawn=o['players'][pi]['cards_drawn_this_turn'];stage='waitactivation'
    if actor==p and a['kind']=='AnnounceMana' and a['card']==source:mana_before=sum(o['players'][pi]['mana']);stage='waitmana'
    if actor==p and a['kind']=='PayCosts':
     paid+=1
     if o['announcing'] is not None and o['announcing']['card']==source:source_paid=True
    if actor==p and a['kind']=='Attack' and stage=='positive' and a['card']==source:stage='finish'
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
