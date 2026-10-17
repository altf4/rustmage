"""Every complete Tiered/adjacent mode naturally drawn, paid and replayed in Standard."""
import json,os,unittest
from pathlib import Path
from rustmage import Game
from test_role_consumers import RoleConsumerTests
ROOT=Path(__file__).resolve().parents[1]
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
NAMES=['Fire Magic','Ice Magic','Restoration Magic','Thunder Magic',"Tifa's Limit Break","Vincent's Limit Break",'Boros Charm','Epic Fight']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
EXTRA={NAMES[0]:[(0,0),(2,0),(5,0)],NAMES[1]:[(0,0),(2,0),(5,1)],NAMES[2]:[(0,0),(1,0),(3,1)],NAMES[3]:[(0,0),(3,0),(5,1)],NAMES[4]:[(0,0),(2,0),(6,1)],NAMES[5]:[(0,0),(1,0),(3,0)]}
FACTS={c['name']:c for c in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
class TieredPublicTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_all_eighteen_tiers_paid_both_seats(self):
  for ni,name in enumerate(NAMES[:6]):
   for mode in range(3):
    for p in ['P0','P1']:
     with self.subTest(name=name,mode=mode,seat=p):self.scenario(name,p,mode,32900+ni*3+mode)
 def test_adjacent_boros_and_epic_every_combination_paid_both_seats(self):
  for ni,name in enumerate(NAMES[6:]):
   for mode in range(3):
    for p in ['P0','P1']:
     with self.subTest(name=name,mode=mode,seat=p):self.scenario(name,p,mode,32918+ni*3+mode)
 def test_blizzara_owner_bottom_choice_both_seats(self):
  for p in ['P0','P1']:
   with self.subTest(seat=p):self.scenario('Ice Magic',p,1,32924,bottom=True)
 def scenario(self,name,p,mode,seed,library=None,bottom=False):
  q='P1' if p=='P0' else 'P0';pi=int(p[-1]);qi=1-pi
  own=['Plains']*8+['Island']*8+['Swamp']*8+['Mountain']*6+['Forest']*6+sum(([n]*4 for n in [name,'Bear Cub','Murder','Bygone Colossus','Quick Study','Giant Growth']),[])
  enemy=['Forest']*32+['Island']*8+sum(([n]*4 for n in ['Bear Cub','Bygone Colossus','Quick Study','Candy Trail','Murder']),[])
  self.assertEqual((len(own),len(enemy)),(60,60))
  with Game([own,enemy] if p=='P0' else [enemy,own],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY')) as g:
   stage='setup';source=target=fighter=victim=None;before_ref=None;paid=proved=attacked=False;frames=0;life=0;dest_chosen=False;actual_payment=None;cast_total=None;owner_library_before=None
   for iteration in range(40000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(101,FP))
    if head['outcome'] is not None:
     self.assertTrue(paid and proved and attacked,(name,p,mode,seed,stage,iteration,paid,proved,attacked));self.assertEqual(stage,'finish');return dict(name=name,player=p,mode=mode,bottom=bottom,frames=frames,paid=actual_payment)
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};own_field=[v for v in objects.values()if v['zone']=='Battlefield'and v['controller']==p];enemy_field=[v for v in objects.values()if v['zone']=='Battlefield'and v['controller']==q];ally=next((v for v in own_field if v['card']=='Bear Cub'),None);foe=next((v for v in enemy_field if v['card']=='Bear Cub'),None);hand=[v for v in g.observe(p)['objects']if v['zone']=='Hand'and v['owner']==p];quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    available=list(o['players'][pi]['mana'])
    for v in own_field:
     if v['card']in BASICS and not v['tapped']:available[BASICS.index(v['card'])]+=1
    required=[0]*6;generic=0
    import re
    for symbol in re.findall(r'\{([^{}]+)\}',FACTS[name]['mana_cost']):
     if symbol.isdigit():generic+=int(symbol)
     else:required['WUBRGC'.index(symbol)]+=1
    if name in EXTRA:
     extra,second=EXTRA[name][mode];generic+=extra
     if second:required[{'Ice Magic':1,'Restoration Magic':0,'Thunder Magic':3,"Tifa's Limit Break":4}[name]]+=1
    cost=sum(required)+generic
    ready=quiet and ally is not None and foe is not None and any(v['card']==name for v in hand) and all(a>=b for a,b in zip(available,required)) and sum(available)>=cost
    if name=="Vincent's Limit Break":ready=ready and available[2]>=required[2]+2 and sum(available)>=cost+3 and any(v['card']=='Murder'for v in hand)
    if o['decision']=='spell_mode' and o['announcing']is not None and o['announcing']['card']==source:
     self.assertTrue(1<=len(o['mode_choices'])<=3);self.assertTrue(any(v['mode']==mode for v in o['mode_choices']))
     if name in EXTRA:
      self.assertIsNone(o['payment_request'])
      extra,second=EXTRA[name][mode];expected=[0]*6
      if second:expected[{'Ice Magic':1,'Restoration Magic':0,'Thunder Magic':3,"Tifa's Limit Break":4}[name]]=1
      selected=next(v for v in o['mode_choices']if v['mode']==mode);self.assertEqual(selected['additional_mana'],dict(colored=expected,generic=extra))
     else:self.assertTrue(all('additional_mana'not in v for v in o['mode_choices']))
    if stage=='wait'and quiet:
     self.assertTrue(paid);self.assertEqual(objects[source]['zone'],'Graveyard');self.assertEqual(sum(actual_payment),cast_total)
     if name=='Fire Magic':
      for b in [fighter,victim]:
       self.assertEqual(objects[b]['zone'],'Battlefield'if mode==0 else 'Graveyard')
       if mode==0:self.assertEqual(objects[b]['damage'],1)
     elif name=='Ice Magic':
      if mode==0:
       owner_view=g.observe(q);returned=next(v for v in owner_view['objects']if v['id']==target);self.assertEqual((returned['zone'],returned['card']),('Hand','Bear Cub'))
      else:
       move=o['last_target_library_move'];self.assertEqual(move['source']['card'],source);self.assertEqual(move['target_before']['card'],target);self.assertEqual(move['owner'],q);self.assertEqual(move['destination'],'Bottom'if bottom else 'Top'if mode==1 else 'Shuffle');self.assertEqual(o['players'][qi]['library_count'],owner_library_before+1);self.assertFalse(any(v['id']==target and v['zone']=='Battlefield'for v in o['objects']))
      if mode==1:self.assertTrue(dest_chosen)
     elif name=='Restoration Magic':
      self.assertEqual(o['players'][pi]['life'],life+[0,3,6][mode]);self.assertEqual(objects[target]['keywords']&(4096|8192),4096|8192);self.assertFalse(objects[victim]['keywords']&8192)
     elif name=='Thunder Magic':self.assertEqual(objects[target]['zone'],'Graveyard')
     elif name=="Tifa's Limit Break":self.assertEqual((objects[target]['power'],objects[target]['toughness']),[(4,4),(4,4),(6,6)][mode])
     elif name=="Vincent's Limit Break":
      self.assertEqual((objects[target]['power'],objects[target]['toughness']),([3,5,7][mode],2));self.assertTrue(any(v['recipient']==objects[target]['reference']and v['granting_card']==name for v in o['temporary_abilities']));before_ref=objects[target]['reference'];stage='kill'
     elif name=='Boros Charm':
      if mode==0:self.assertEqual(o['players'][qi]['life'],life-4)
      elif mode==1:self.assertTrue(objects[fighter]['keywords']&8192);self.assertTrue(all(v['keywords']&8192 for v in own_field));self.assertFalse(objects[victim]['keywords']&8192)
      else:self.assertTrue(objects[target]['keywords']&1024)
     elif name=='Epic Fight':
      if mode==0:self.assertEqual((objects[target]['power'],objects[target]['toughness']),(4,4))
      else:
       self.assertEqual(objects[victim]['zone'],'Graveyard')
       if mode==1:self.assertEqual(objects[fighter]['zone'],'Graveyard')
       else:self.assertEqual((objects[fighter]['power'],objects[fighter]['toughness']),(4,4));self.assertEqual(objects[fighter]['damage'],2)
     if stage=='wait':proved=True;stage='finish'
    if stage=='wait_return'and quiet:
     body=objects[target];self.assertEqual(body['zone'],'Battlefield');self.assertNotEqual(body['reference'],before_ref);self.assertTrue(body['tapped']);self.assertEqual(body['controller'],p);self.assertEqual((body['power'],body['toughness']),(2,2));proved=True;stage='finish'
    if o['target_library']is not None:self.assertIsNone(g.observe('P1'if actor=='P0'else'P0')['target_library'])
    remaining={a.get('card')for a in o['actions']if a['kind']in ['Discard','Bottom']}
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceActivate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and n==name and ready:return 4500
      if actor==p and stage=='kill'and n=='Murder':return 4500
      if stage=='finish':return 3000 if n in ['Bear Cub','Bygone Colossus']else -1000
      if n=='Bear Cub':return 2800 if (ally if actor==p else foe)is None else -1000
      return -1000
     if k=='ChooseMode':return 4500 if a['mode']==mode else -1000
     if k=='ChooseTarget':
      wanted=target
      if name=='Boros Charm'and mode==0 and stage=='wait':return 4500 if a['target'].get('Player')==q else -1000
      if name=='Epic Fight'and mode!=0 and stage=='wait':
       selected=len(o['announcing']['targets']);wanted=([fighter,victim]if mode==1 else[target,fighter,victim])[selected]
      return 4500 if a['target'].get('Card')==wanted else -1000
     if k=='ChooseLibraryDestination':return 4500 if a['destination']==('Bottom'if bottom else'Top')else -1000
     if k=='PlayLand':return 2500-20*sum(v['card']==n for v in objects.values()if v['zone']=='Battlefield'and v['controller']==actor)
     if k in ['AnnounceMana','TapMana','ActivateMana']:
      if o['decision']!='pay_costs'or n not in BASICS:return -1000
      request=o['payment_request'];pool=o['players'][int(actor[-1])]['mana'];index=BASICS.index(n)
      if pool[index]<request['required'][index]:return 3400
      return 2800 if not(name=="Vincent's Limit Break"and stage=='wait'and index==2)else 1500
     if k in ['Discard','Bottom']:
      count=sum(objects[x]['card']==n for x in remaining)
      if actor==p and stage=='setup'and n==name and count<=1:return -1000
      if actor==p and stage in ['setup','kill']and name=="Vincent's Limit Break"and n=='Murder'and count<=1:return -1000
      field=own_field if actor==p else enemy_field
      if actor==p and not attacked and n in ['Bear Cub','Bygone Colossus']and count<=1:return -1000
      if n=='Bear Cub'and not any(v['card']in ['Bear Cub','Bygone Colossus']for v in field)and count<=1:return -1000
      return 2600 if n not in BASICS else 1000
     if k=='Attack':return 4500 if stage=='finish'and actor==p else -1000
     if k=='Block':return -1000
     return {'Keep':4500,'PayCosts':4200,'ChooseSpellCost':4200,'FinishTargets':3900,'PutTrigger':3800,'FinishSearch':3700,'FinishAttackers':3500,'FinishBlockers':3500,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda i:score(o['actions'][i]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,mode,seed,stage,o['decision'],a))
    if a['kind']=='AnnounceCast'and actor==p:
     if stage=='setup'and objects[a['card']]['card']==name:
      source=a['card'];fighter=ally['id'];victim=foe['id'];target=foe['id']if name in ['Ice Magic','Thunder Magic']else ally['id'];life=o['players'][qi if name=='Boros Charm'and mode==0 else pi]['life'];cast_total=cost;owner_library_before=o['players'][qi]['library_count'];stage='wait'
     elif stage=='kill'and objects[a['card']]['card']=='Murder':stage='wait_return'
    if a['kind']=='PayCosts'and actor==p and o['announcing']['card']==source:paid=True;actual_payment=a['payment']
    if a['kind']=='ChooseLibraryDestination':dest_chosen=True
    if a['kind']=='Attack'and actor==p:attacked=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,mode,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
