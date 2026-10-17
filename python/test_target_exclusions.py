"""Real paid complete filter/modal bodies in naturally drawn Standard games."""
import json,os,unittest
from pathlib import Path
from rustmage import Game
import test_role_consumers as roles
EXPECTED=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
SIMPLE=['Coeurl','Sterling Keykeeper','Rock Soldiers','Guerrilla Gorilla']
MODAL=['Overwhelming Surge','Amazing Acrobatics','Decoy Ploy']
class TargetExclusionTests(unittest.TestCase):
 frame=roles.RoleConsumerTests.frame
 def test_complete_filter_bodies_both_seats(self):
  for i,name in enumerate(SIMPLE):
   for p in ['P0','P1']:
    with self.subTest(name=name,player=p):self.scenario(name,p,None,3000+i)
 def test_complete_modal_bodies_every_combination_both_seats(self):
  for i,name in enumerate(MODAL):
   for mode in range(3):
    for p in ['P0','P1']:
     with self.subTest(name=name,mode=mode,player=p):self.scenario(name,p,mode,3010+i*3+mode)
 def scenario(self,name,p,mode,seed,library=None):
  q='P1' if p=='P0' else 'P0';deck=['Plains']*6+['Island']*6+['Swamp']*4+['Mountain']*4+['Forest']*4+sum(([n]*4 for n in [name,'Bear Cub','Chrome Companion','Bygone Colossus','Candy Trail','Basilisk Collar','Agent of Atlas','Common Crook','Llanowar Elves']),[])
  enemy=['Forest']*24+['Plains']*4+['Mountain']*4+sum(([n]*4 for n in ['Bear Cub','Basilisk Collar','Chrome Companion','Candy Trail','Llanowar Elves','Bygone Colossus','Disenchant']),[])
  self.assertEqual((len(deck),len(enemy)),(60,60))
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY')) as g:
   stage='setup';source=body=artifact=hero=villain=counter=None;frames=paid=0;proved=False;initial=None
   for iteration in range(30000):
    o=g.observe('P0')
    if o['outcome'] is not None:
     self.assertTrue(proved,(name,p,mode,seed,stage,iteration));return dict(name=name,player=p,mode=mode,frames=frames,paid=paid)
    o=g.observe(o['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(86,EXPECTED));objects={v['id']:v for v in o['objects']};private=g.observe(p);hand=[v for v in private['objects'] if v['zone']=='Hand' and v['owner']==p];own=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p];foes=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==q];quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    if stage=='setup':
     body=next((v['id'] for v in foes if v['card']=='Bear Cub'),None);artifact=next((v['id'] for v in foes if v['card']=='Basilisk Collar'),None)
     hero=next((v['id'] for v in objects.values() if v['zone']=='Graveyard' and v['owner']==p and v['card']=='Agent of Atlas'),None);villain=next((v['id'] for v in objects.values() if v['zone']=='Graveyard' and v['owner']==p and v['card']=='Common Crook'),None)
    if stage=='wait_body' and quiet and objects[source]['zone']=='Battlefield':stage='activate'
    if stage=='wait_effect' and quiet:
     if name in ['Coeurl','Sterling Keykeeper']:self.assertTrue(objects[body]['tapped']);self.assertTrue(objects[source]['tapped'])
     elif name in ['Rock Soldiers','Guerrilla Gorilla']:self.assertEqual(objects[artifact]['zone'],'Graveyard');self.assertEqual(objects[source]['zone'],'Battlefield' if name=='Rock Soldiers' else 'Graveyard')
     elif name=='Overwhelming Surge':
      self.assertEqual(objects[body]['zone'],'Graveyard' if mode!=1 else 'Battlefield');self.assertEqual(objects[artifact]['zone'],'Graveyard' if mode!=0 else 'Battlefield')
     elif name=='Amazing Acrobatics':
      if counter is not None:self.assertEqual(objects[counter]['zone'],'Graveyard' if mode!=1 else 'Battlefield')
      self.assertTrue(objects[body]['tapped'] if mode!=0 else not objects[body]['tapped'])
     else:
      self.assertEqual(objects[villain]['zone'],'Hand' if mode!=1 else 'Graveyard');self.assertEqual(objects[hero]['zone'],'Hand' if mode!=0 else 'Graveyard')
     self.assertGreaterEqual(paid,1);proved=True;stage='finish'
    enemy_spell=next((s['card'] for s in o['stack'] if objects.get(s['card'],{}).get('controller')==q and objects.get(s['card'],{}).get('card')=='Bear Cub'),None)
    ready=body is not None and artifact is not None
    if name=='Decoy Ploy':ready=ready and hero is not None and villain is not None
    if name=='Amazing Acrobatics' and mode!=1:ready=ready and enemy_spell is not None
    wanted={v for v in ([villain] if mode==0 else [hero] if mode==1 else [villain,hero]) if v is not None} if name=='Decoy Ploy' else {v for v in ([body] if name in ['Coeurl','Sterling Keykeeper'] or (name=='Overwhelming Surge' and mode==0) else [artifact] if name in ['Rock Soldiers','Guerrilla Gorilla'] or (name=='Overwhelming Surge' and mode==1) else [body,artifact]) if v is not None}
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure']:return -2000
     if k=='AnnounceCast':
      if actor==p:
       if stage=='setup' and ready and n==name:return 3600
       if stage=='finish' and n in ['Bear Cub','Chrome Companion','Bygone Colossus','Llanowar Elves','Agent of Atlas','Common Crook']:return 2600
       return -1000
      if stage=='finish':return -1000
      if n=='Bear Cub' and not any(v['card']=='Bear Cub' for v in foes):return 2800
      if n=='Basilisk Collar' and not any(v['card']=='Basilisk Collar' for v in foes):return 2800
      if name=='Amazing Acrobatics' and mode!=1 and stage=='setup' and body is not None and artifact is not None and n=='Bear Cub' and any(v['card']==name for v in hand) and sum(v['card']=='Island' and not v['tapped'] for v in own)>=2 and sum(v['card'] in BASICS and not v['tapped'] for v in own)>=3:return 2800
      return -1000
     if k=='AnnounceActivate':return 3600 if actor==p and stage=='activate' and a['card']==source and a['ability']==0 else -1000
     if k=='ChooseMode':return 3600 if a['mode']==mode else -1000
     if k=='ChooseTarget':
      if name=='Amazing Acrobatics':return 3700 if (counter is not None and a['target'].get('Spell')==counter) or (a['target'].get('Card')==body and mode!=0) else -1000
      return 3700 if a['target'].get('Card') in wanted else -1000
     if k=='PlayLand':
      if actor==p and name=='Decoy Ploy' and stage=='setup' and sum(v['card'] in BASICS for v in own)>=5 and (n!='Swamp' or any(v['card']=='Swamp' for v in own)):return -1000
      return 2200-20*sum(v['card']==n and v['controller']==actor and v['zone']=='Battlefield' for v in objects.values())
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3000 if o['decision'] in ['pay_costs','resolution_mana'] else -1000
     if k in ['Discard','Bottom']:
      if actor==p and stage=='setup' and name=='Decoy Ploy' and ((n=='Agent of Atlas' and hero is None) or (n=='Common Crook' and villain is None)):return 3500
      if actor==p and stage=='setup' and n==name and sum(v['card']==n for v in hand)<=1:return -1000
      return 2500 if n in ['Candy Trail','Basilisk Collar','Chrome Companion'] else 1200
     if k=='Attack':return 3600 if stage=='finish' and actor==p else -1000
     if k=='Block':return -1000
     return {'Keep':3900,'PayCosts':3800,'ChooseSpellCost':3800,'FinishTargets':3500,'PutTrigger':3400,'FinishAttackers':3000,'FinishBlockers':3000,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda i:score(o['actions'][i]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,mode,seed,stage,o['decision']))
    if actor==p and a['kind']=='AnnounceCast' and objects[a['card']]['card']==name:
     source=a['card'];counter=enemy_spell;stage='wait_body' if name in ['Coeurl','Sterling Keykeeper','Guerrilla Gorilla'] else 'wait_effect';initial=private['players'][int(p[-1])]['mana']
    elif actor==p and a['kind']=='AnnounceActivate' and a['card']==source:stage='wait_effect'
    if actor==p and a['kind']=='PayCosts':paid+=1
    self.frame(g,o,index);frames+=1
   self.fail((name,p,mode,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
