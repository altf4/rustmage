"""Actual paid bargain declarations and complete effects through the public library."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError, Game
NAMES=["Archon's Glory","Candy Grapple","Farsight Ritual","Hamlet Glutton","High Fae Negotiator","Ice Out","Johann's Stopgap","Kellan's Lightblades","Tenacious Tomeseeker","Troublemaker Ouphe"]
BASICS=['Plains','Island','Swamp','Mountain','Forest']
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
def cost(name,pay):
 r=[0]*6;n=0
 for s in FACTS[name]['mana_cost'].replace('}','').split('{')[1:]:
  if s.isdigit():n+=int(s)
  else:r['WUBRGC'.index(s)]+=1
 return r,max(0,n-({'Hamlet Glutton':2,'Ice Out':1,"Johann's Stopgap":2}.get(name,0) if pay else 0))
class BargainTests(unittest.TestCase):
 def replay(self,g,o,j):
  saved=g.snapshot()
  with g.clone() as fork: expected=fork.step(fork.observe(o['actor']),j)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(o['actor']),j);expected['revision']=after['revision'];self.assertEqual(after,expected);g.drop_snapshot(saved);return after
 def reject(self,g,o,a,actor=None):
  before=g.observe(o['actor'])
  with self.assertRaises(EngineError):g.act(dict(o,actor=actor or o['actor']),a)
  self.assertEqual(g.observe(o['actor']),before)
 def test_ten_complete_bodies_both_branches_both_players(self):
  for i,name in enumerate(NAMES):
   for pay in (False,True):
    for p in ('P0','P1'):
     with self.subTest(card=name,pay=pay,seat=p):self.scenario(i,pay,p)
 def scenario(self,i,pay,p,resource='Candy Trail',exile=False):
  name=NAMES[i];foe_name='Hamlet Glutton' if i in (1,7) else 'Bear Cub';other='P1' if p=='P0' else 'P0';owner=int(p[1]);enemy=1-owner
  deck=sum(([n]*12 for n in BASICS),[])+[name]*16+[('Hop to It' if resource=='Rabbit' else resource)]*12+['Bear Cub']*12+['Thrill of Possibility']*8+['Shock']*8
  if exile:deck += ["Alania's Pathmaker"]*16
  foe=['Mountain']*32+['Forest']*24+[foe_name]*16+['Candy Trail']*12+['Shock']*12
  with Game([deck,foe] if p=='P0' else [foe,deck],seed=31 if i==7 else 43,starting_player=p,deck_rules='Freeform') as g:
   declared=paid=False;root=base=None;selected=[];source=None;resource_ref=None;private=None
   for _ in range(40000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,pay,p));o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};ann=o['announcing']
    friends=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['card']=='Bear Cub']
    foes=[v for v in objects.values() if v['owner']==other and v['zone']=='Battlefield' and v['card']==foe_name]
    grave=[v for v in objects.values() if v['owner']==p and v['zone']=='Graveyard' and v['card']=='Shock']
    handshock=[v for v in objects.values() if v['owner']==p and v['zone']=='Hand' and v['card']=='Shock']
    resources=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and (v['card']==resource or resource=='Rabbit' and v['token'] is not None and v['token']['name']=='Rabbit Token')]
    # Only untapped basic lands count for readiness; the engine still validates joint real funding.
    required,generic=cost(name,pay);potential=o['players'][owner]['mana'][:]
    for v in objects.values():
     if v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS and not v['tapped']:potential[BASICS.index(v['card'])]+=1
    enough=all(a>=b for a,b in zip(potential,required)) and sum(potential)>=sum(required)+generic
    ready=enough and (not pay or resources) and (i!=0 or friends) and (i!=8 or grave) and (i not in (1,6,7) or foes) and (i!=9 or any(v['owner']==other and v['zone']=='Battlefield' and v['card']=='Candy Trail' for v in objects.values()))
    if o['decision']=='bargain' and ann['source_card']==name:
     self.assertEqual(actor,p);self.assertIsNone(o['payment_request']);self.assertEqual(ann['targets'],[]);self.assertIsNone(ann['cast']['bargain']);self.assertEqual(g.observe(other)['actions'],[])
     self.reject(g,o,{'kind':'ChooseBargain','sacrifice':None},other)
     self.reject(g,o,{'kind':'ChooseBargain','sacrifice':{'card':4294967295,'generation':0}})
     self.reject(g,o,{'kind':'ChooseKicker','pay':True})
     self.reject(g,o,{'kind':'ChooseTarget','target':{'Player':other}})
     for v in objects.values():
      if v['zone']=='Battlefield' and v['owner']==p and v['card'] in BASICS:
       self.reject(g,o,{'kind':'ChooseBargain','sacrifice':v['reference']});break
     if resources:
      stale=dict(resources[0]['reference']);stale['generation']+=1;self.reject(g,o,{'kind':'ChooseBargain','sacrifice':stale})
    if ann and ann['source_card']==name and o['decision']=='pay_costs':
     self.assertEqual((o['payment_request']['required'],o['payment_request']['generic']),(required,generic));self.assertEqual(ann['cast'],{'origin':'Exile' if exile else 'Hand','method':'Normal','offspring':None,'kicker':None,'bargain':pay})
     if resource_ref:self.assertEqual(objects[resource_ref['card']]['reference'],resource_ref);self.assertEqual(objects[resource_ref['card']]['zone'],'Battlefield')
    if o['library_selection'] and o['resolving']['source_card']==name:
     f=o['library_selection'];self.assertEqual(i,2);self.assertEqual(f['remainder'],'bottom_random');self.assertFalse(f['reveal']);self.assertEqual((f['minimum'],f['maximum']),(2,2));self.assertIsNone(g.observe(other)['library_selection']);self.assertEqual(g.observe(other)['actions'],[])
     if private is None:private=o['players'][owner]['library_count'];self.assertEqual(len(f['remaining']),8 if pay else 4)
     self.assertEqual(o['players'][owner]['library_count'],private)
     for v in f['remaining']+f['selected']:self.assertEqual(v['zone'],'Library')
    def score(a):
     k=a['kind']
     if k in ('Cast','AnnounceCast','AnnounceExile'):
      n=objects[a['card']]['card']
      if actor==other:
       if n=='Shock':return 1100 if i==5 and enough and not paid else -1000
       return 700 if n==foe_name and len(foes)<1 or n=='Candy Trail' and i==9 else -1000
      if n==name:return 1400 if ready and not paid and k==('AnnounceExile' if exile else 'AnnounceCast') else -1000
      if n==resource:return 950 if not resources else -1000
      if resource=='Rabbit' and n=='Hop to It':return 950 if not resources else -1000
      if n=='Bear Cub':return 800 if i==0 and not friends else -1000
      if n=='Thrill of Possibility':return 1100 if i==8 and not grave and handshock else -1000
      if n=="Alania's Pathmaker":return 900 if exile and enough and resources else -1000
      return -1000
     if k=='ChooseBargain' and actor==other:return 1600 if a['sacrifice'] is None else -1000
     if k=='ChooseBargain':return 1600 if bool(a['sacrifice'])==pay and (not pay or a['sacrifice']['card'] in [v['id'] for v in resources]) else -1000
     if k=='ChooseAdditionalCost':return 1500 if objects[a['object']['card']]['card']=='Shock' else -1000
     if k=='ChooseTarget':
      t=a['target']
      if 'Spell' in t:return 1500 if i==5 and objects[t['Spell']]['owner']==other else -1000
      if 'Player' in t:return 1300 if t['Player']==p else -1000
      v=objects[t['Card']]
      if i==8:return 1500 if v in grave else -1000
      if i==9:return 1500 if v['owner']==other and v['card']=='Candy Trail' else -1000
      return 1500 if v in (friends if i==0 else foes) else -1000
     if k=='PlayLand':
      n=objects[a['card']]['card'];count=sum(v['zone']=='Battlefield' and v['owner']==actor and v['card']==n for v in objects.values());return 1200-count*10+int(actor==p and required[BASICS.index(n)]>count)*100
     if k=='TapMana':return 800 if o['decision']=='pay_costs' else -1000
     if k=='Attack':return 700 if i==7 and actor==other and ready and not paid else -1000
     if k=='SelectLibrary':return 1500
     if k=='Discard' and objects[a['card']]['card']==name:return 100
     if k=='Discard':return 1400 if i==8 and not grave and objects[a['card']]['card']=='Shock' else 1100 if objects[a['card']]['card'] in BASICS else 900
     if k=='ArrangeLibrary':return 1000 if a['destination']=='Top' else 900
     return {'Keep':1700,'PayCosts':1600,'PutTrigger':1500,'FinishTargets':1300,'FinishLibrarySelection':1400,'FinishAttackers':600,'FinishBlockers':600,'Pass':0}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j]
    if a['kind']=='ChooseBargain' and actor==p:declared=True;resource_ref=a['sacrifice']
    if a['kind']=='PayCosts' and ann and ann['source_card']==name:paid=True;source=ann['object'];self.assertEqual(sum(a['payment']),sum(required)+generic)
    if a['kind']=='SelectLibrary' and o['resolving']['source_card']==name:selected.append(a['card'])
    if a['kind']=='Pass' and o['stack'] and o['stack'][-1]['source_card']==name:root=o['stack'][-1];base=o
    after=self.replay(g,o,j);self.assertEqual(after['schema'],64)
    if a['kind']=='PayCosts' and ann and ann['source_card']==name and resource_ref:
     v=next((v for v in after['objects'] if v['id']==resource_ref['card']),None);self.assertTrue(v is None or v['zone'] in ('Graveyard','Outside'))
    if root and not any(v['id']==root['id'] for v in after['stack']+after['waiting_triggers']) and not(after['announcing'] and after['announcing']['id']==root['id']) and not(after['resolving'] and after['resolving']['id']==root['id']):
     packet=root;root=None;views={v['id']:v for v in after['objects']}
     if packet['kind']=='Spell' and i in (3,4,8,9):
      entered=views[source['card']];self.assertEqual(entered['zone'],'Battlefield');self.assertEqual(entered['reference']['generation'],source['generation']+1)
      if i==3:self.assertTrue(entered['keywords']&256);continue
      if i==4:self.assertTrue(entered['keywords']&1)
      if pay:continue
      self.assertFalse(any(v['source_card']==name for v in after['stack']+after['waiting_triggers']));self.assertEqual(after['players'][owner]['life'],base['players'][owner]['life']);return
     if i==0:
      old=next(v for v in base['objects'] if v['id']==packet['targets'][0]['object']['card']);v=views[old['id']];self.assertEqual((v['power'],v['toughness']),(old['power']+2,old['toughness']+2));self.assertEqual(bool(v['keywords']&1),pay);self.assertEqual(bool(v['keywords']&64),pay)
     if i==1:
      old=next(v for v in base['objects'] if v['id']==packet['targets'][0]['object']['card']);v=views[old['id']];self.assertEqual(v['zone'],'Battlefield');self.assertEqual((v['power'],v['toughness']),(old['power']-(5 if pay else 3),old['toughness']-(5 if pay else 3)))
     if i==2:
      self.assertEqual(len(selected),2);self.assertEqual(after['players'][owner]['library_count'],private-2);self.assertEqual(after['players'][owner]['cards_drawn_this_turn'],base['players'][owner]['cards_drawn_this_turn']);self.assertTrue(all(views[n]['zone']=='Hand' for n in selected))
     if i==3:self.assertEqual(after['players'][owner]['life'],base['players'][owner]['life']+3)
     if i==4:self.assertEqual(after['players'][owner]['life'],base['players'][owner]['life']+3);self.assertEqual(after['players'][enemy]['life'],base['players'][enemy]['life']-3)
     if i==5:self.assertEqual(views[packet['targets'][0]['object']['card']]['zone'],'Graveyard');self.assertEqual(after['players'][owner]['life'],base['players'][owner]['life'])
     if i==6:self.assertEqual(views[packet['targets'][0]['object']['card']]['zone'],'Hand');self.assertEqual(after['players'][owner]['cards_drawn_this_turn'],base['players'][owner]['cards_drawn_this_turn']+1)
     if i==7:
      v=views[packet['targets'][0]['object']['card']];self.assertEqual(v['zone'],'Graveyard' if pay else 'Battlefield')
      if not pay:self.assertEqual(v['damage'],3)
     if i==8:self.assertEqual(views[packet['targets'][0]['object']['card']]['zone'],'Hand')
     if i==9:self.assertEqual(views[packet['targets'][0]['object']['card']]['zone'],'Exile')
     self.assertTrue(declared and paid);return
   self.fail('no complete bargain witness '+str((name,pay,p,exile)))
 def treasure_window(self,p):
  other='P1' if p=='P0' else 'P0';owner=int(p[1])
  deck=['Swamp']*24+['Mountain']*24+['Candy Grapple']*16+['Candy Trail']*12+['Plundering Pirate']*16+['Marauding Blight-Priest']*16
  foe=['Forest']*48+['Bear Cub']*24
  with Game([deck,foe] if p=='P0' else [foe,deck],seed=31,starting_player=p,deck_rules='Freeform') as g:
   for _ in range(20000):
    o=g.observe('P0');self.assertIsNone(o['outcome']);o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']}
    lands=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS]
    trail=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['card']=='Candy Trail']
    treasure=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['token'] is not None and v['token']['name']=='Treasure Token']
    foecreature=[v for v in objects.values() if v['owner']==other and v['zone']=='Battlefield' and v['card']=='Bear Cub']
    grapples=[v for v in objects.values() if v['owner']==p and v['zone']=='Hand' and v['card']=='Candy Grapple']
    if o['decision']=='bargain':
     self.assertEqual(o['announcing']['source_card'],'Candy Grapple');self.assertEqual(len(treasure),1);self.assertTrue(trail)
     available=sum(o['players'][owner]['mana'])+sum(not v['tapped'] for v in lands);self.assertEqual(available,1)
     self.assertIn({'kind':'ChooseBargain','sacrifice':None},o['actions']);bad={'kind':'ChooseBargain','sacrifice':treasure[0]['reference']};self.assertNotIn(bad,o['actions']);self.reject(g,o,bad)
     good={'kind':'ChooseBargain','sacrifice':trail[0]['reference']};self.assertIn(good,o['actions']);saved=g.observe(p)
     with g.clone() as fork:
      f=fork.observe(p);self.replay(fork,f,f['actions'].index(good));paid=False
      for _ in range(30):
       f=fork.observe(p)
       if paid:
        break
       def payment_score(a):
        k=a['kind']
        if k=='ChooseTarget':return 2000 if a['target']=={'Card':foecreature[0]['id']} else -1000
        if k=='PayCosts':return 1900
        if k=='TapMana':return 1200
        if k=='ActivateMana':return 1500 if a['card']==treasure[0]['id'] and a['production'][2]==1 else 1000
        return -1000
       j=max(range(len(f['actions'])),key=lambda j:payment_score(f['actions'][j]));paid=f['actions'][j]['kind']=='PayCosts';after=self.replay(fork,f,j)
      self.assertTrue(paid);v={v['id']:v for v in after['objects']};self.assertEqual(v[trail[0]['id']]['zone'],'Graveyard');self.assertTrue(treasure[0]['id'] not in v or v[treasure[0]['id']]['zone'] in ('Outside','Graveyard'));self.assertTrue(after['stack'][-1]['cast']['bargain'])
     self.assertEqual(g.observe(p),saved);self.replay(g,o,o['actions'].index({'kind':'ChooseBargain','sacrifice':None}));return
    def score(a):
     k=a['kind']
     if k in ('AnnounceCast','Cast'):
      n=objects[a['card']]['card']
      if actor==other:return 900 if n=='Bear Cub' and not foecreature else -1000
      if n=='Candy Trail':return 1000 if not trail else -1000
      if n=='Plundering Pirate':return 1100 if len(lands)==4 and trail and grapples and not treasure and foecreature else -1000
      if n=='Marauding Blight-Priest':return 1100 if treasure and sum(o['players'][owner]['mana'])+sum(not v['tapped'] for v in lands)==4 else -1000
      if n=='Candy Grapple':return 1400 if treasure and trail and foecreature and sum(o['players'][owner]['mana'])+sum(not v['tapped'] for v in lands)==1 else -1000
      return -1000
     if k=='PlayLand':
      n=objects[a['card']]['card'];mountains=sum(v['card']=='Mountain' for v in lands);return 1200 if actor!=p or len(lands)<4 and (n=='Mountain' or mountains or len(lands)<3) else -1000
     if k=='ChooseTarget':return 1400 if a['target']=={'Card':foecreature[0]['id']} else -1000
     if k=='TapMana':return 800 if o['decision']=='pay_costs' else -1000
     if k=='ArrangeLibrary':return 1000 if a['destination']=='Top' else 900
     if k=='Discard':
      n=objects[a['card']]['card'];copies=sum(v['owner']==actor and v['zone']=='Hand' and v['card']==n for v in objects.values());return 100 if n in ('Candy Grapple','Plundering Pirate','Marauding Blight-Priest') and copies==1 else 1100
     return {'Keep':1700,'PayCosts':1600,'PutTrigger':1500,'FinishTargets':1300,'FinishAttackers':600,'FinishBlockers':600,'Pass':0}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));self.replay(g,o,j)
   self.fail('no actual Treasure funding window '+p)
 def test_real_nonartifact_tokens_enchantments_and_paid_exile(self):
  for p in ('P0','P1'):
   for resource in ('Warleader\'s Call','Rabbit'):
    with self.subTest(seat=p,resource=resource):self.scenario(0,True,p,resource)
   with self.subTest(seat=p,origin='Exile'):self.scenario(0,True,p,exile=True)
   with self.subTest(seat=p,treasure=True):self.treasure_window(p)
if __name__=='__main__':unittest.main()
