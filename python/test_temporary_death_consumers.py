"""Naturally drawn, paid whole temporary-death and Suspect Standard consumers."""
import json,unittest
from pathlib import Path
from rustmage import Game
from test_role_consumers import RoleConsumerTests
NAMES=["Galuf's Final Act",'Rabid Attack','Perigee Beckoner','Presumed Dead','Absolving Lammasu','Person of Interest',"It Doesn't Add Up",'Reasonable Doubt','Convenient Target']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
ROOT=Path(__file__).resolve().parents[1]
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
class TemporaryDeathConsumerTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_nine_whole_bodies_paid_both_seats(self):
  for name in NAMES:
   for p in ['P0','P1']:
    with self.subTest(card=name,seat=p):self.scenario(name,p,31100+NAMES.index(name))
 def test_perigee_warp_paid_both_seats(self):
  for p in ['P0','P1']:
   with self.subTest(seat=p):self.scenario('Perigee Beckoner',p,31120,warp=True)
 def scenario(self,name,p,seed,library=None,warp=False):
  q='P1' if p=='P0' else 'P0'
  spells=[name,'Bear Cub','Murder','Quick Study','Bygone Colossus','Disenchant']
  if name=='Absolving Lammasu':spells.append('Person of Interest')
  lands=60-4*len(spells);deck=sum(([n]*(lands//5+(i<lands%5)) for i,n in enumerate(BASICS)),[])+sum(([n]*4 for n in spells),[])
  self.assertEqual(len(deck),60)
  with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library) as g:
   stage='setup';recipient=source=old=counter_source=None;paid=complete=attacked=False;frames=0;draw_before=life_before=0;first_person=None;observed_warp=False;warp_exiled_turn=None;warp_recast_paid=False
   for iteration in range(40000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(98,FP))
    if head['outcome'] is not None:
     self.assertTrue(complete and paid,(name,p,seed,stage,iteration,paid));self.assertEqual(stage,'finish');self.assertTrue(attacked,(name,p,'no natural attack'))
     if warp:self.assertTrue(observed_warp and warp_recast_paid)
     return dict(frames=frames,card=name,seat=p,warp=warp,outcome=head['outcome'])
    o=g.observe(head['actor']);actor=o['actor'];objects={x['id']:x for x in o['objects']};quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    def own(player):return [x for x in objects.values() if x['zone']=='Battlefield' and x['controller']==player]
    def hand(player):return [x for x in g.observe(player)['objects'] if x['zone']=='Hand' and x['owner']==player]
    def ready_mana(player):
     mana=list(o['players'][int(player[-1])]['mana'])
     for x in own(player):
      if not x['tapped'] and x['card'] in BASICS:mana[BASICS.index(x['card'])]+=1
     return mana
    def holding(n):return any(x['card']==n for x in hand(p))
    bears=[x for x in own(p) if x['card']=='Bear Cub'];foes=[x for x in own(q) if x['card']=='Bear Cub'];suspected={x['card'] for x in o['suspected']}
    if stage=='wait_grant' and quiet:
     self.assertTrue(any(x['recipient']==old and x['granting_card']==name for x in o['temporary_abilities']),(name,o['temporary_abilities']))
     stage='kill';draw_before=o['players'][int(p[-1])]['cards_drawn_this_turn']
    if stage=='wait_return' and quiet:
     v=objects[recipient];self.assertEqual(v['zone'],'Battlefield');self.assertNotEqual(v['reference'],old);self.assertEqual(v['controller'],p)
     self.assertEqual(v['tapped'],name=='Perigee Beckoner');self.assertEqual(v['power'],2)
     if name in ['Presumed Dead',"It Doesn't Add Up"]:self.assertIn(recipient,suspected)
     complete=not warp;stage='wait_warp_exile' if warp else 'finish'
    if stage=='wait_galuf' and quiet:
     self.assertEqual(objects[recipient]['zone'],'Graveyard');self.assertTrue(any(x['counters']['plus_one']==3 for x in own(p) if x['card']=='Bear Cub'));complete=True;stage='finish'
    if stage=='wait_draw' and quiet:
     self.assertEqual(objects[recipient]['zone'],'Graveyard');self.assertEqual(o['players'][int(p[-1])]['cards_drawn_this_turn'],draw_before+1);complete=True;stage='finish'
    if stage=='wait_person' and quiet:
     self.assertIn(source,suspected);self.assertTrue(any(x['token'] and x['token']['name']=='Detective Token' for x in own(p)));complete=True;stage='finish'
    if stage=='wait_setup_person' and quiet:
     self.assertIn(first_person,suspected);stage='setup'
    if stage=='wait_lammasu' and quiet:
     self.assertNotIn(first_person,suspected);stage='kill_lammasu';life_before=o['players'][int(p[-1])]['life']
    if stage=='wait_lammasu_death' and quiet:
     self.assertEqual(o['players'][int(p[-1])]['life'],life_before+3);self.assertTrue(any(x['id'] in suspected for x in own(q) if x['card']=='Bear Cub'));complete=True;stage='finish'
    if stage=='wait_grave' and quiet:
     self.assertEqual(objects[recipient]['zone'],'Graveyard');stage='return'
    if stage=='wait_counter' and quiet:
     self.assertEqual(objects[counter_source]['zone'],'Graveyard');self.assertIn(recipient,suspected);complete=True;stage='finish'
    if stage=='wait_aura' and quiet:
     self.assertEqual(objects[recipient]['power'],3);self.assertIn(recipient,suspected);stage='remove_aura'
    if stage=='wait_remove_aura' and quiet:
     self.assertEqual(objects[source]['zone'],'Graveyard');self.assertEqual(objects[recipient]['power'],2);self.assertIn(recipient,suspected);stage='activate_aura'
    if stage=='wait_aura_hand' and quiet:
     self.assertTrue(any(x['id']==source for x in hand(p)));self.assertIn(recipient,suspected);complete=True;stage='finish'
    if stage=='wait_warp_exile' and objects[source]['zone']=='Exile':
     permission=next(x for x in o['warped_cards'] if x['object']==objects[source]['reference']);self.assertEqual(permission['owner'],p);warp_exiled_turn=permission['exiled_turn'];stage='wait_warp_later'
    if stage=='wait_recast_grant' and quiet:
     self.assertTrue(any(x['recipient']==objects[recipient]['reference'] and x['granting_card']==name for x in o['temporary_abilities']));self.assertEqual(objects[recipient]['power'],4);self.assertFalse(any(x['object']['card']==source for x in o['warped_cards']));self.assertFalse(any(x['source']['card']==source for x in o['delayed_warp_exiles']));complete=True;stage='finish'
    available=ready_mana(p);required=[holding(name),bool(bears),available[2]>=2,sum(available)>=7,holding('Murder')]
    if name in ["Galuf's Final Act",'Rabid Attack']:required.append(len(bears)>=2)
    if name=='Absolving Lammasu':required.extend([first_person is not None,bool(foes)])
    if name=='Convenient Target':required.extend([holding('Disenchant'),available[0]>=1,available[3]>=1])
    if name=='Reasonable Doubt':required.extend([available[1]>=1,any(x['card']=='Bear Cub' for x in hand(q))])
    if name in ['Rabid Attack','Presumed Dead','Perigee Beckoner']:required.append(available[2]>=3)
    if name=="It Doesn't Add Up":required.extend([available[2]>=4,sum(available)>=8])
    if name=='Absolving Lammasu' or name=='Perigee Beckoner' and not warp:required.append(sum(available)>=8)
    ready=quiet and all(required)
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k=='AnnounceExile':return 4600 if actor==p and stage=='wait_warp_later' and a['card']==source and o['turn']>warp_exiled_turn and sum(ready_mana(p))>=5 and ready_mana(p)[2]>=1 else -2000
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceActivate','AnnounceAdventure','AnnounceOmen','AnnounceFlashback','AnnouncePlot','AnnounceExile','AnnounceWarp']:return 4500 if k=='AnnounceWarp' and actor==p and name=='Perigee Beckoner' and warp and stage=='setup' and ready and n==name else 4500 if k=='AnnounceActivate' and actor==p and stage=='activate_aura' and a['card']==source else -2000
     if k=='AnnounceCast':
      if actor==p:
       if stage=='setup' and ready and n==name and name not in ["It Doesn't Add Up",'Reasonable Doubt'] and not warp:return 4500
       if stage=='return' and n==name:return 4500
       if stage in ['kill','kill_lammasu','bury'] and n=='Murder':return 4500
       if stage=='remove_aura' and n=='Disenchant':return 4500
       if name=='Absolving Lammasu' and stage=='setup' and first_person is None and n=='Person of Interest' and available[3]>=1 and sum(available)>=4:return 4200
       if name=='Reasonable Doubt' and stage=='counter' and n==name:return 4500
      if actor==q and name=='Reasonable Doubt' and n=='Bear Cub' and stage=='await_spell':return 4500
      if n=='Bear Cub':
       needed=2 if actor==p and name in ["Galuf's Final Act",'Rabid Attack'] else 1
       return 2400 if stage=='finish' or stage=='setup' and sum(x['card']=='Bear Cub' for x in own(actor))<needed else -1000
      if n=='Bygone Colossus':return 2300 if stage=='finish' else -1000
      if n=='Quick Study':return 2200 if stage=='setup' and quiet and sum(ready_mana(actor))>=5 else -1000
      return -1000
     if k=='ChooseTarget':
      t=a['target']
      if name=='Reasonable Doubt' and stage in ['counter','wait_counter']:return 4000 if t.get('Spell')==counter_source or t.get('Card')==recipient else -1000
      if name=="Galuf's Final Act" and stage=='wait_galuf':return 4000 if any(x['id']==t.get('Card') for x in bears) else -1000
      if name=='Absolving Lammasu' and stage=='wait_lammasu_death' and (o['announcing']or{}).get('source_card')!='Murder':return 4000 if any(x['id']==t.get('Card') for x in foes) else -1000
      return 4000 if t.get('Card')==(source if stage=='wait_remove_aura' else recipient) else -1000
     if k=='PlayLand':return 1800-20*sum(x['card']==n for x in own(actor))+(35 if n=='Swamp' else 0)
     if k in ['TapMana','ActivateMana','AnnounceMana']:
      if o['decision'] not in ['pay_costs','resolution_mana']:return -1000
      costs=(o['announcing']or{}).get('source_card','');color={'Murder':2,name:1}.get(costs,0)
      idx=2 if costs=='Murder' or costs in ['Presumed Dead','Rabid Attack','Perigee Beckoner',"It Doesn't Add Up"] else 4 if costs=="Galuf's Final Act" else 3 if costs in ['Person of Interest','Convenient Target'] else 1 if costs in ['Quick Study','Reasonable Doubt'] else 0
      return 2500 if n==BASICS[idx] and o['players'][int(actor[-1])]['mana'][idx]<color else 2200 if n!='Swamp' else 1800
     if k=='DeclineResolutionMana':return 3500
     if k in ['PayCosts','PutTrigger','Keep','ChooseSpellCost']:return 3000
     if k=='FinishTargets':return 2900
     if k in ['FinishAttackers','FinishBlockers']:return 2400
     if k=='Attack':return 2800 if stage=='finish' else -1000
     if k in ['Discard','Bottom']:return -500 if stage!='finish' and n in [name,'Murder','Disenchant','Person of Interest','Bear Cub'] and sum(objects.get(choice.get('card'),{}).get('card')==n for choice in o['actions'] if choice['kind']==k)<=1 else 1500
     if k=='Pass':return 0
     return 1000
    if name=="It Doesn't Add Up" and stage=='setup' and ready:recipient=bears[0]['id'];old=bears[0]['reference'];stage='bury'
    if name=='Reasonable Doubt' and stage=='setup' and ready:recipient=bears[0]['id'];stage='await_spell'
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision']))
    if a['kind']=='AnnounceCast':
     n=objects[a['card']]['card']
     if actor==q and name=='Reasonable Doubt' and n=='Bear Cub' and stage=='await_spell':counter_source=a['card'];stage='counter'
     elif actor==p:
      if n=='Person of Interest' and name=='Absolving Lammasu' and first_person is None:first_person=a['card'];stage='wait_setup_person'
      elif n==name:
       source=a['card']
       if name=='Reasonable Doubt':stage='wait_counter'
       elif name=='Person of Interest':stage='wait_person'
       elif name=='Absolving Lammasu':recipient=source;stage='wait_lammasu'
       elif name=="It Doesn't Add Up":stage='wait_return'
       else:recipient=bears[0]['id'];old=bears[0]['reference'];stage='wait_aura' if name=='Convenient Target' else 'wait_grant'
      elif n=='Murder':stage='wait_galuf' if name=="Galuf's Final Act" else 'wait_draw' if name=='Rabid Attack' else 'wait_lammasu_death' if name=='Absolving Lammasu' else 'wait_grave' if name=="It Doesn't Add Up" else 'wait_return'
      elif n=='Disenchant' and stage=='remove_aura':stage='wait_remove_aura'
    if a['kind']=='AnnounceExile':stage='wait_recast_grant'
    if a['kind']=='AnnounceWarp':source=a['card'];recipient=bears[0]['id'];old=bears[0]['reference'];stage='wait_grant';observed_warp=True
    if a['kind']=='AnnounceActivate' and stage=='activate_aura':stage='wait_aura_hand'
    if a['kind']=='PayCosts' and (o['announcing']or{}).get('card')==source:
     paid=True
     if warp and stage=='wait_recast_grant':self.assertEqual((o['announcing']['cast']['origin'],o['announcing']['cast']['method']),('Exile','Normal'));warp_recast_paid=True
    if a['kind']=='Attack' and actor==p:attacked=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal game'))
if __name__=='__main__':unittest.main()
