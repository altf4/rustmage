"""Actual paid Warp, delayed exile and later normal paid recasting; complete bodies."""
import unittest
from rustmage import Game, EngineError
NAMES=['Bygone Colossus','Eusocial Engineering','Germinating Wurm','Knight Luminary','Red Tiger Mechan','Sinister Cryologist','Starbreach Whale','Weftblade Enhancer','Drix Fatemaker','Mechanozoa','Memorial Team Leader','Nova Hellkite']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
PREFERRED=['Forest','Forest','Forest','Plains','Mountain','Island','Island','Plains','Forest','Island','Mountain','Mountain']

class WarpTests(unittest.TestCase):
 def replay(self,g,o,j):
  actor=o['actor'];other='P1' if actor=='P0' else 'P0';saved=g.snapshot();parent=g.clone();child=parent.clone();parent.close()
  with child:expected=child.step(child.observe(actor),j);expected_other=child.observe(other)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(after,expected);v=g.observe(other);expected_other['revision']=v['revision'];self.assertEqual(v,expected_other);g.drop_snapshot(saved);return after
 def denied(self,g,o,a):
  old=[g.observe(v) for v in ['P0','P1']]
  with self.assertRaises(EngineError):g.act(o,a)
  self.assertEqual(old,[g.observe(v) for v in ['P0','P1']])
 def test_twelve_complete_normal_hardcasts_both_seats(self):
  for i in range(12):
   for p in ['P0','P1']:
    with self.subTest(name=NAMES[i],seat=p):self.scenario(i,p,False)
 def test_twelve_actual_warp_exile_and_later_paid_recasts_both_seats(self):
  for i in range(12):
   for p in ['P0','P1']:
    with self.subTest(name=NAMES[i],seat=p):self.scenario(i,p,True)
 def test_complete_collateral_flash_damage_and_artifact_guard_both_seats(self):
  for i in [12,13]:
   for p in ['P0','P1']:
    with self.subTest(card=i,seat=p):self.scenario(i,p,False)
 def test_actual_runaway_cycling_cost_discard_and_draw_both_seats(self):
  for p in ['P0','P1']:
   with self.subTest(seat=p):self.scenario(12,p,False,cycle=True)
 def scenario(self,i,p,warp,cycle=False):
  name=NAMES[i] if i<12 else ['Runaway Boulder','Waylaying Pirates'][i-12];pi=int(p[1]);other='P1' if p=='P0' else 'P0';preferred=PREFERRED[i] if i<12 else 'Forest' if i==12 else 'Island';foe='Gigantosaurus' if i==12 else 'Bear Cub'
  own=sum(([n]*4 for n in BASICS),[])+[preferred]*36+['Forest']*28+[name]*16+['Bear Cub']*16+['Candy Trail']*12
  enemy=['Forest']*48+[foe]*16+['Candy Trail']*12
  with Game([own,enemy] if p=='P0' else [enemy,own],seed=2450+i,starting_player=p,deck_rules='Freeform') as g:
   source=None;phase='setup';round=0;baseline=None;old_ref=None;exiled_turn=None;land_effect=False
   for step in range(30000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,warp,phase,step));o=g.observe(o['actor']);actor=o['actor'];objs={v['id']:v for v in o['objects']};mine=[v for v in objs.values() if v['owner']==p and v['zone']=='Battlefield'];theirs=[v for v in objs.values() if v['owner']==other and v['zone']=='Battlefield'];hand=[v for v in objs.values() if v['owner']==p and v['zone']=='Hand'];lands=[v for v in mine if v['card']in BASICS and not v['tapped']];ally=next((v for v in mine if v['card']=='Bear Cub'),None);enemy_target=next((v for v in theirs if v['card']==('Candy Trail' if i in [9,13] else foe)),None);artifact=any(v['card']=='Candy Trail' for v in mine);quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    if cycle and phase=='resolving' and quiet:
     self.assertEqual(objs[source]['zone'],'Graveyard');self.assertEqual(o['players'][pi]['library_count'],baseline['library']-1);self.assertEqual(o['players'][pi]['spells_cast_this_turn'],baseline['spells']);return
    if phase=='effect' and quiet:
     body=objs[source];self.assertEqual(body['zone'],'Battlefield')
     if i==1 and not land_effect:pass
     else:
      if i==0:self.assertEqual((body['power'],body['toughness']),(9,9))
      if i==1:
       robots=[v for v in mine if v.get('token') and v['token']['subtypes']=='Robot'];self.assertEqual(len(robots),baseline['robots']+1);self.assertEqual((robots[-1]['power'],robots[-1]['toughness'],robots[-1]['token']['colors']),(2,2,0))
      if i==2:self.assertEqual(o['players'][pi]['life'],baseline['life']+2)
      if i==3:
       soldiers=[v for v in mine if v.get('token') and v['token']['subtypes']=='Human Soldier'];self.assertEqual(len(soldiers),baseline['soldiers']+1);self.assertEqual((soldiers[-1]['power'],soldiers[-1]['toughness'],soldiers[-1]['token']['colors']),(1,1,1))
      if i==4:self.assertEqual((body['power'],body['toughness']),(3,3));self.assertTrue(body['keywords']&4)
      if i==5:self.assertEqual(enemy_target['power'],baseline['enemy_power']-3)
      if i==6:self.assertTrue(body['keywords']&1);self.assertEqual(o['players'][pi]['library_count'],baseline['library']-2)
      if i==7:self.assertEqual(ally['counters']['plus_one'],baseline['ally_counter']+1);self.assertEqual(enemy_target['counters']['plus_one'],baseline['enemy_counter']+1)
      if i==8:self.assertEqual(ally['counters']['plus_one'],baseline['ally_counter']+1);self.assertTrue(ally['keywords']&256);self.assertFalse(body['keywords']&256);self.assertFalse(enemy_target['keywords']&256)
      if i in [9,13]:self.assertTrue(enemy_target['tapped']);self.assertEqual(enemy_target['counters']['stun'],baseline['stun']+1)
      if i==10:self.assertEqual(ally['power'],3);self.assertEqual(body['power'],4);self.assertEqual(enemy_target['power'],2)
      if i==11:self.assertEqual(enemy_target['damage'],baseline['damage']+1);self.assertTrue(body['keywords']&1 and body['keywords']&4)
      if i==12:self.assertEqual(enemy_target['damage'],baseline['damage']+6);self.assertNotEqual(baseline['active'],p)
      if not warp or round==1:
       self.assertFalse(any(v['object']['card']==source for v in o['warped_cards']));self.assertFalse(any(v['source']['card']==source for v in o['delayed_warp_exiles']));return
      old_ref=body['reference'];d=next(v for v in o['delayed_warp_exiles'] if v['source']==old_ref);self.assertEqual(d['controller'],p);self.assertEqual(o['delayed_warp_exiles'],g.observe(other)['delayed_warp_exiles']);phase='wait_exile'
    if phase=='wait_exile' and objs[source]['zone']=='Exile':
     permission=next(v for v in o['warped_cards'] if v['object']['card']==source);self.assertEqual(permission['owner'],p);self.assertEqual(permission['object']['generation'],old_ref['generation']+1);self.assertEqual(permission['object'],objs[source]['reference']);exiled_turn=permission['exiled_turn'];self.assertEqual(exiled_turn,o['turn']);self.assertEqual(o['warped_cards'],g.observe(other)['warped_cards']);phase='wait_later'
    if phase=='wait_later':
     for illegal in [{'kind':'AnnounceWarp','card':source},{'kind':'AnnouncePlotted','card':source},{'kind':'AnnounceCast','card':source},{'kind':'Cast','card':source,'target':None,'payment':[0]*6}]:self.denied(g,o,illegal)
     if o['turn']==exiled_turn:self.denied(g,o,{'kind':'AnnounceExile','card':source})
    can_land=any(a['kind']=='PlayLand' for a in o['actions']);ready=len(lands)>=(2 if cycle else 9) and (i not in [7,8,10] or ally) and (i not in [5,7,8,9,10,11,12,13] or enemy_target) and (i!=13 or artifact) and (i!=1 or can_land) and (i!=12 or cycle or o['active']!=p)
    def score(a):
     k=a['kind']
     if k in ['Cast','BeginCast','AnnouncePlotted','AnnounceFlashback','Concede','Attack']:return -1000
     if k=='AnnounceWarp':return 2200 if actor==p and warp and phase=='setup' and objs[a['card']]['card']==name and ready else -1000
     if k=='AnnounceExile':return 2200 if actor==p and phase=='wait_later' and a['card']==source and o['turn']>exiled_turn else -1000
     if k=='AnnounceActivate':return 2200 if cycle and actor==p and phase=='setup' and objs[a['card']]['card']==name and ready else -1000
     if k=='AnnounceCast':
      n=objs[a['card']]['card']
      if actor!=p:return 1800 if n==foe and not any(v['card']==foe for v in theirs) or n=='Candy Trail' and i in [9,13] and not any(v['card']=='Candy Trail' for v in theirs) else -1000
      if n==name:return 2200 if not warp and not cycle and phase=='setup' and ready else -1000
      if n=='Bear Cub':return 1900 if i in [7,8,10] and not ally else -1000
      if n=='Candy Trail':return 1900 if i==13 and not artifact else -1000
      return -1000
     if k=='ChooseTarget':
      target=a['target'].get('Card');v=objs.get(target,{})
      if i in [5,9,11,12,13]:return 2400 if enemy_target and target==enemy_target['id'] else 1000
      if i in [7,8]:return 2400 if ally and target==ally['id'] else 2350 if i==7 and enemy_target and target==enemy_target['id'] else 1000
      return 1000
     if k=='PlayLand':
      if actor==p and phase=='effect' and i==1:return 2500
      n=objs[a['card']]['card'];count=sum(v['owner']==actor and v['zone']=='Battlefield' and v['card']==n for v in objs.values());return 1700 if n==preferred and count<2 else 1650 if n=='Forest' and count<2 else 1600-20*count
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2000 if o['decision'] in ['pay_costs','resolution_mana'] else -1000
     if k in ['PayCosts','PutTrigger','ChooseSpellCost','FinishTargets','Keep','FinishAttackers','FinishBlockers']:return 2300
     if k=='ArrangeLibrary':return 2300 if a['destination']=='Graveyard' else 2200 if a['destination']=='Top' else 2100
     if k in ['Discard','Bottom']:
      n=objs[a['card']]['card'];protected=n==name and source is None or n=='Bear Cub' and i in [7,8,10] and not ally or n=='Candy Trail' and i==13 and not artifact;return -500 if protected else 2000
     if k=='Pass':return 0
     return 500
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];ann=o['announcing']
    if a['kind']in ['AnnounceWarp','AnnounceCast','AnnounceExile','AnnounceActivate'] and actor==p and objs[a['card']]['card']==name:
     source=a['card'];self.denied(g,dict(o,actor=other),a)
    if a['kind']=='PlayLand' and actor==p and phase=='effect' and i==1:land_effect=True
    if a['kind']=='PayCosts' and ann and ann['controller']==p and ann['source_card']==name:
     if ann['kind']=='Spell':
      phase='effect';round=int(ann['cast']['origin']=='Exile');self.assertEqual(ann['cast']['method'],'Warp' if warp and round==0 else 'Normal');land_effect=False
     else:phase='resolving'
     baseline=dict(library=o['players'][pi]['library_count'],life=o['players'][pi]['life'],robots=sum(bool(v.get('token') and v['token']['subtypes']=='Robot') for v in mine),soldiers=sum(bool(v.get('token') and v['token']['subtypes']=='Human Soldier') for v in mine),ally_counter=ally['counters']['plus_one'] if ally else 0,enemy_counter=enemy_target['counters']['plus_one'] if enemy_target else 0,enemy_power=enemy_target['power'] if enemy_target else 0,stun=enemy_target['counters']['stun'] if enemy_target else 0,damage=enemy_target['damage'] if enemy_target else 0,spells=o['players'][pi]['spells_cast_this_turn'],active=o['active'])
    self.replay(g,o,j)
   self.fail(('no complete Warp scenario',name,p,warp,cycle,phase))
