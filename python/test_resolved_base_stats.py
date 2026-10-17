"""Naturally drawn, fully paid resolved-stat consumers with owned replay."""
import json,os,unittest
from pathlib import Path
from rustmage import OBSERVATION_SCHEMA, Game
import test_role_consumers as replay
FP=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
CASES=[('Genemorph Imago',True),('Moon Girl and Devil Dinosaur',True),('Belligerent Yearling',True),('Belligerent Yearling',False),('PuPu UFO',True),('Atomic Microsizer',True)]
BASICS=['Forest','Island','Mountain','Swamp']
class ResolvedBasePublicTests(unittest.TestCase):
 frame=replay.RoleConsumerTests.frame
 def test_five_complete_bodies_and_optional_choice_paid_both_seats(self):
  for i,(name,accept)in enumerate(CASES):
   for p in ['P0','P1']:
    with self.subTest(name=name,accept=accept,seat=p):self.scenario(i,p,32600+i)
 def scenario(self,i,p,seed,library=None):
  if i>=4:return self.extra_scenario(CASES[i][0],p,seed,library)
  name,accept=CASES[i];q='P1'if p=='P0'else'P0';pi=int(p[-1]);cards=[name,'Bear Cub','Barkform Harvester','Three Tree Mascot','Quick Study','Giant Growth','Murder','Depower','Bygone Colossus']
  own=sum(([n]*6 for n in BASICS),[])+sum(([n]*4 for n in cards),[])
  enemy=['Forest']*12+['Island']*8+['Mountain']*4+sum(([n]*4 for n in ['Bear Cub','Chrome Companion','Bygone Colossus','Three Tree Mascot','Barkform Harvester','Quick Study','Murder','Giant Growth','Depower']),[])
  self.assertEqual((len(own),len(enemy)),(60,60))
  with Game([own,enemy]if p=='P0'else[enemy,own],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   source=subject=None;stage='setup';frames=0;paid=False;low=False;effect=False;turn=None;artifact=None;second=None;depower=None
   for iteration in range(30000):
    head=g.observe('P0')
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertTrue(paid);self.assertTrue(effect);return dict(name=name,accept=accept,seat=p,seed=seed,frames=frames)
    o=g.observe(head['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(OBSERVATION_SCHEMA,FP));objects={v['id']:v for v in o['objects']};field=[v for v in objects.values()if v['zone']=='Battlefield'];mine=[v for v in field if v['controller']==p];body=objects.get(source);subject_view=objects.get(subject)
    def hand(player):return [v for v in g.observe(player)['objects']if v['zone']=='Hand'and v['owner']==player]
    def available(player):
     budget=list(o['players'][int(player[-1])]['mana'])
     for v in field:
      if v['controller']==player and v['card']in BASICS and not v['tapped']:budget[['Plains','Island','Swamp','Mountain','Forest'].index(v['card'])]+=1
     return budget
    ownhand=hand(p);lands=sum(v['card']in BASICS for v in mine);budget=available(p);quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving'];bear=next((v for v in mine if v['card']=='Bear Cub'),None)
    if stage=='waitsource'and quiet and body is not None and body['zone']=='Battlefield':
     self.assertTrue(paid);stage={'Genemorph Imago':'land','Moon Girl and Devil Dinosaur':'artifact','Belligerent Yearling':'dino'}[name]
    if stage=='waitland'and quiet:
     value=6 if lands>=6 else 3;self.assertEqual((subject_view['power'],subject_view['toughness']),(value,value));self.assertTrue(any(e['recipient']==subject_view['reference']and not e['static_source']for e in o['base_stat_effects']))
     if value==3:low=True;stage='land'
     else:self.assertTrue(low);effect=True;turn=o['turn'];stage='cleanup'
    if stage=='waitartifact'and quiet:
     self.assertEqual(o['players'][pi]['cards_drawn_this_turn'],2);self.assertEqual((body['power'],body['toughness']),(6,6));self.assertTrue(body['keywords']&256);stage='secondartifact'
    if stage=='waitsecond'and quiet:
     self.assertEqual(o['players'][pi]['cards_drawn_this_turn'],2);self.assertEqual(len([e for e in o['base_stat_effects']if e['recipient']==body['reference']]),1);effect=True;turn=o['turn'];stage='cleanup'
    if stage=='waitdepower'and depower is not None and objects.get(depower,{}).get('zone')=='Graveyard':
     self.assertEqual(subject_view['power'],-2);stage='yearlingresult'
    if stage=='yearlingresult'and quiet:
     self.assertEqual((body['power'],body['toughness']),(-2 if accept else 3,2));effect=True;turn=o['turn'];stage='cleanup'
    if stage=='cleanup'and quiet and o['turn']>turn:
     self.assertFalse([e for e in o['base_stat_effects']if not e['static_source']]);self.assertFalse(o['resolved_stat_subjects'])
     if name=='Genemorph Imago':self.assertEqual((subject_view['power'],subject_view['toughness']),(2,2))
     if name=='Moon Girl and Devil Dinosaur':self.assertEqual((body['power'],body['toughness']),(2,2));self.assertFalse(body['keywords']&256)
     if name=='Belligerent Yearling':self.assertEqual((body['power'],body['toughness']),(3,2))
     stage='finish'
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceActivate','AnnounceAdventure','AnnounceOmen','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceExile']:return -10000
     if k=='AnnounceCast':
      if stage=='finish':return 2500 if n in ['Bygone Colossus','Bear Cub','Chrome Companion','Barkform Harvester','Three Tree Mascot']else -1000
      if actor!=p:return -1000
      if stage=='setup'and n==name and(name!='Genemorph Imago'or bear is not None):return 4000
      if name=='Genemorph Imago'and stage=='setup'and n=='Bear Cub'and bear is None:return 3500
      if stage=='dino'and n=='Barkform Harvester'and sum(budget)>=6 and budget[1]>=1 and any(v['card']=='Depower'for v in ownhand):return 4000
      if stage=='waitdino'and n=='Depower'and subject_view is not None and subject_view['zone']=='Battlefield':return 5000
      if stage=='artifact'and n in ['Barkform Harvester','Three Tree Mascot']and o['active']==p and o['players'][pi]['cards_drawn_this_turn']==1 and sum(budget)>=6 and sum(v['card']in ['Barkform Harvester','Three Tree Mascot']for v in ownhand)>=2:return 4000
      if stage=='secondartifact'and n in ['Barkform Harvester','Three Tree Mascot']:return 4000
      if stage in ['setup','dino']and n=='Quick Study'and quiet and sum(budget)>=4:return 2800
      return -1000
     if k=='ChooseTarget':return 4000 if a['target'].get('Card')==subject else -1000
     if k=='ChooseOptional':return 4000 if a['accept']==accept else -1000
     if k=='PlayLand':
      if actor==p and name=='Genemorph Imago'and not low and stage not in ['land','finish']:
       if n in ['Forest','Island']and any(v['card']==n for v in mine):return -1000
       if n in ['Mountain','Swamp']and sum(v['card']in ['Mountain','Swamp']for v in mine)>=2:return -1000
      if actor==p and name=='Genemorph Imago'and stage=='land':return 4200
      field_count=sum(v['card']==n and v['controller']==actor for v in field)
      return 2200-30*field_count
     if k in ['AnnounceMana','TapMana','ActivateMana']:
      if stage=='waitdino'and n=='Island':return -1000
      return 3000 if o['decision']in ['pay_costs','resolution_mana']else -1000
     if k in ['PayCosts','FinishTargets','PutTrigger','Keep','ChooseSpellCost']:return 3200
     if k in ['Discard','Bottom']:
      reserve={name,'Bear Cub','Barkform Harvester','Depower'}if name!='Moon Girl and Devil Dinosaur'else{name,'Barkform Harvester','Three Tree Mascot'}
      return 1000-(800 if stage!='finish'and n in reserve and sum(v['card']==n for v in hand(actor))<=2 else 0)
     if k=='Attack':return 2000 if stage=='finish'else -1000
     if k in ['Block','BlockAttacker']:return -1000
     if k=='ChooseReplacement':return 2500
     return 100
    chosen=max(enumerate(o['actions']),key=lambda x:score(x[1]))[0];a=o['actions'][chosen];k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
    if actor==p and k=='AnnounceCast':
     if stage=='setup'and n==name:source=a['card'];stage='waitsource'
     elif stage=='dino'and n=='Barkform Harvester':subject=a['card'];stage='waitdino'
     elif stage=='waitdino'and n=='Depower':depower=a['card'];stage='waitdepower'
     elif stage=='artifact'and n in ['Barkform Harvester','Three Tree Mascot']:artifact=a['card'];stage='waitartifact'
     elif stage=='secondartifact'and n in ['Barkform Harvester','Three Tree Mascot']:second=a['card'];stage='waitsecond'
    if actor==p and k=='PayCosts'and o['announcing']is not None and o['announcing']['card']==source:
     self.assertEqual(sum(a['payment']),3 if name=='Moon Girl and Devil Dinosaur'else 2);paid=True
    if actor==p and k=='PlayLand'and name=='Genemorph Imago'and stage=='land':subject=bear['id'];stage='waitland'
    self.frame(g,o,chosen);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal result'))

 def test_pupu_and_atomic_complete_paid_bodies_both_seats(self):
  for name in ['PuPu UFO','Atomic Microsizer']:
   for p in ['P0','P1']:
    with self.subTest(name=name,seat=p):self.extra_scenario(name,p,32604+int(name=='Atomic Microsizer'))
 def extra_scenario(self,name,p,seed,library=None):
  q='P1'if p=='P0'else'P0';towns=["Adventurer's Inn",'Capital City'];lands=towns+['Island','Forest'];slots=[name,'Bear Cub','Barkform Harvester','Three Tree Mascot','Quick Study','Giant Growth','Bygone Colossus','Chrome Companion','Llanowar Elves'];own=towns[:0]+[towns[0]]*4+[towns[1]]*4+['Island']*8+['Forest']*8+sum(([c]*4 for c in slots),[]);enemy=['Forest']*12+['Island']*12+sum(([c]*4 for c in ['Bear Cub','Barkform Harvester','Three Tree Mascot','Quick Study','Giant Growth','Bygone Colossus','Chrome Companion','Llanowar Elves','Depower']),[]);self.assertEqual((len(own),len(enemy)),(60,60))
  with Game([own,enemy]if p=='P0'else[enemy,own],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   source=bear=selected=None;stage='setup';turn=None;frames=0;cast_paid=False;power_paid=False;equip_paid=False;hand_seen=False;effect=False;captured=None
   for iteration in range(35000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(OBSERVATION_SCHEMA,FP))
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertTrue(cast_paid);self.assertTrue(effect);self.assertTrue(power_paid if name=='PuPu UFO'else equip_paid);return dict(name=name,seat=p,seed=seed,frames=frames)
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};field=[v for v in objects.values()if v['zone']=='Battlefield'];mine=[v for v in field if v['controller']==p];hand=[v for v in g.observe(p)['objects']if v['zone']=='Hand'and v['owner']==p];body=objects.get(source);bearbody=objects.get(bear);quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving'];towncount=sum(v['card']in towns for v in mine);mana=sum(o['players'][int(p[-1])]['mana'])+sum(v['card']in lands and not v['tapped']for v in mine);handtown=[v for v in hand if v['card']in towns]
    if stage=='waitsource'and quiet:
     self.assertEqual(body['zone'],'Battlefield');self.assertTrue(cast_paid);stage='power'if name=='PuPu UFO'else'equip'
    if stage=='waitpower'and quiet:
     self.assertEqual((body['power'],body['toughness']),(captured,4));self.assertTrue(body['keywords']&1);stage='hand'
    if stage=='waithand'and quiet:
     self.assertTrue(hand_seen);self.assertIsNotNone(selected);self.assertEqual(objects[selected]['zone'],'Battlefield');self.assertEqual(body['power'],captured);self.assertEqual(sum(v['card']in towns for v in mine),captured+1);effect=True;turn=o['turn'];stage='cleanup'
    if stage=='waitequip'and quiet:
     self.assertEqual(body['attached_to'],bearbody['reference']);self.assertEqual((bearbody['power'],bearbody['toughness']),(3,2));stage='attack'
    if stage=='waitattack'and bearbody is not None and bearbody['keywords']&16384:
     self.assertEqual((bearbody['power'],bearbody['toughness']),(2,1));effect=True;turn=o['turn'];stage='cleanup'
    if stage=='cleanup'and quiet and o['turn']>turn:
     if name=='PuPu UFO':self.assertEqual((body['power'],body['toughness']),(0,4))
     else:self.assertEqual((bearbody['power'],bearbody['toughness']),(3,2));self.assertFalse(bearbody['keywords']&16384)
     stage='finish'
    choice=o.get('effect_choice')
    if choice and isinstance(choice['kind'],dict)and'HandLand'in choice['kind']:
     self.assertEqual(actor,p);self.assertIsNone(g.observe(q)['effect_choice']);self.assertTrue(choice['optional']);hand_seen=True
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceAdventure','AnnounceOmen','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceExile']:return -10000
     if k=='AnnounceCast':
      if stage=='finish':return 3000 if n in ['Bear Cub','Bygone Colossus','Barkform Harvester','Chrome Companion','Three Tree Mascot','Llanowar Elves']else -1000
      if actor!=p:return -1000
      if stage=='setup'and n==name and (name=='PuPu UFO'and towncount>=2 and handtown and mana>=3 or name=='Atomic Microsizer'and bearbody is not None):return 5000
      if stage=='setup'and name=='Atomic Microsizer'and n=='Bear Cub'and bear is None:return 4500
      if stage=='setup'and n=='Quick Study'and quiet and mana>=5:return 2800
      return -1000
     if k=='AnnounceActivate':return 5000 if actor==p and a['card']==source and ((stage=='power'and a['ability']==1 and mana>=3 and o['active']==p and not body['summoning_sick']and not body['tapped'])or(stage=='hand'and a['ability']==0 and handtown)or(stage=='equip'and a['ability']==0))else -1000
     if k=='ChooseTarget':return 5000 if a['target'].get('Card')==bear else -1000
     if k=='PutHandLand':return 5000 if objects[a['object']['card']]['card']in towns else -1000
     if k=='DeclineHandEntry':return -1000
     if k=='PlayLand':
      if actor==p and stage!='finish'and name=='PuPu UFO':
       if stage in ['power','waitpower','hand','waithand','cleanup']:return -1000
       if n in towns and towncount>=2:return -1000
      return 2200-20*sum(v['card']==n and v['controller']==actor for v in field)
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3500 if o['decision']in ['pay_costs','resolution_mana']else -1000
     if k in ['PayCosts','FinishTargets','PutTrigger','Keep','ChooseSpellCost']:return 4200
     if k in ['Discard','Bottom']:
      if actor==p and stage!='finish'and (n in [name,'Bear Cub']or name=='PuPu UFO'and n in towns)and sum(v['card']==n for v in hand)<=2:return -1000
      return 1500
     if k=='Attack':return 3000 if stage=='finish'or actor==p and stage=='attack'and a['card']==bear else -1000
     if k in ['Block','BlockAttacker']:return -1000
     if k=='FinishAttackers':return 2500
     return 100
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
    if actor==p and k=='AnnounceCast':
     if stage=='setup'and n==name:source=a['card'];stage='waitsource'
     elif stage=='setup'and n=='Bear Cub':bear=a['card']
    if actor==p and k=='AnnounceActivate'and a['card']==source:
     if stage=='power':captured=towncount;stage='waitpower'
     elif stage=='hand':stage='waithand'
     elif stage=='equip':stage='waitequip'
    if actor==p and k=='Attack'and stage=='attack':stage='waitattack'
    if k=='PutHandLand':selected=a['object']['card']
    if k=='PayCosts'and o.get('announcing')and o['announcing']['card']==source:
     if stage=='waitpower':self.assertEqual(sum(a['payment']),3);power_paid=True
     elif stage=='waitequip':self.assertEqual(sum(a['payment']),2);equip_paid=True
     elif stage=='waithand':self.assertEqual(sum(a['payment']),0)
     else:self.assertEqual(sum(a['payment']),2 if name=='PuPu UFO'else 1);cast_paid=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,frames))
