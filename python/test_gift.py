"""Paid complete Gift bodies in naturally terminal Standard games and owned replay."""
import json,os,re,unittest
from pathlib import Path
from rustmage import Game,EngineError
import test_role_consumers as replay
ROOT=Path(__file__).resolve().parents[1]
FACTS={c['name']:c for c in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
NAMES=["Bilbo's Gambit",'Blooming Blast','Consumed by Greed','Crumb and Get It',"Dawn's Truce",'Dewdrop Cure','Into the Flood Maw',"Long River's Pull",'Longstalk Brawl','Mind Spiral','Nocturnal Hunger','Peerless Recycling',"Sazacap's Brew",'Scrapshooter','Valley Rally','Wear Down','Wildfire Howl']
class GiftTests(unittest.TestCase):
 frame=replay.RoleConsumerTests.frame
 def test_seventeen_complete_promised_bodies_paid_both_seats(self):
  for i,name in enumerate(NAMES):
   for p in ['P0','P1']:
    with self.subTest(card=name,seat=p,promise=True):self.scenario(name,p,31000+i,promise=True)
 def test_seventeen_complete_declined_bodies_paid_both_seats(self):
  for i,name in enumerate(NAMES):
   for p in ['P0','P1']:
    with self.subTest(card=name,seat=p,promise=False):self.scenario(name,p,31040+i,promise=False)
 def scenario(self,name,p,seed,promise=True,library=None):
  q='P1'if p=='P0'else'P0';pi=int(p[-1]);qi=1-pi
  deck=['Plains']*6+['Island']*6+['Swamp']*7+['Mountain']*6+['Forest']*7+sum(([n]*4 for n in [name,'Bear Cub','Chrome Companion','Quick Study','Murder','Doubling Season','Bygone Colossus']),[])
  self.assertEqual(len(deck),60)
  grave_need=(3 if promise else 2)if name=='Dewdrop Cure'else(2 if promise else 1)if name=='Peerless Recycling'else int(name=='Consumed by Greed'and promise)
  artifacts=(2 if promise else 1)if name=='Wear Down'else int(name in ['Into the Flood Maw','Scrapshooter'])
  counter=name in ["Bilbo's Gambit","Long River's Pull"]
  victim_name='Bear Cub' if name=="Long River's Pull" and not promise else 'Quick Study'
  ally_need=name in ['Crumb and Get It',"Dawn's Truce",'Longstalk Brawl',"Sazacap's Brew",'Valley Rally','Wildfire Howl']
  foe_need=name in ['Blooming Blast','Consumed by Greed','Into the Flood Maw','Longstalk Brawl','Mind Spiral','Nocturnal Hunger','Wildfire Howl']
  with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_GIFT_LIBRARY'))as g:
   stage='setup';source=victim=kill=discard=None;targets=[];pre=None;paid=False;declared=False;attacked=False;frames=extra=0;cast_ref=None;entry_packets=set()
   for iteration in range(40000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(101,FP))
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,p,seed,promise,stage,iteration));self.assertTrue(paid and declared and attacked)
     return dict(card=name,seat=p,promise=promise,frames=frames,denials=2*frames+extra,outcome=head['outcome'])
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']}
    def field(player):return[v for v in objects.values()if v['zone']=='Battlefield'and v['controller']==player]
    def hand(player):return[v for v in g.observe(player)['objects']if v['zone']=='Hand'and v['owner']==player]
    def graves():return[v for v in objects.values()if v['zone']=='Graveyard'and v['owner']==p and v['card']in ['Bear Cub','Chrome Companion']]
    def available(player):
     pool=list(o['players'][int(player[-1])]['mana'])
     for v in field(player):
      if not v['tapped']and v['card']in BASICS:pool[BASICS.index(v['card'])]+=1
     return pool
    def can_pay(card,player):
     symbols=re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[card]['mana_cost']);colored=[symbols.count(c)for c in 'WUBRGC'];generic=sum(int(s)for s in symbols if s.isdecimal());pool=available(player)
     return all(a>=b for a,b in zip(pool,colored))and sum(pool)>=sum(colored)+generic
    quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    own=field(p);enemy=field(q);ally=next((v for v in own if v['card']=='Bear Cub'),None);foe=next((v for v in enemy if v['card']=='Bear Cub'),None);enemy_artifacts=[v for v in enemy if v['card']=='Doubling Season'];big=next((v for v in enemy if v['card']=='Bygone Colossus'),None)
    if stage=='wait_grave'and quiet:stage='setup';kill=None
    for packet in [*o['stack'],*([o['announcing']]if o['announcing']else[])]:
     if packet['card']==source and packet.get('gift')is not None:
      context=packet['gift'];self.assertEqual((context['source'],context['caster'],context['beneficiary']),(cast_ref,p,q));self.assertEqual(context['entry'],objects[source]['reference']);self.assertEqual(packet['controller'],p);entry_packets.add(packet['id'])
    if stage=='wait_body'and quiet:
     self.assertTrue(paid and declared);s=objects[source];self.assertEqual(s['owner'],p);self.assertEqual(s['zone'],'Battlefield'if name=='Scrapshooter'else'Graveyard')
     kind=FACTS[name]['oracle_text'].splitlines()[0].split(' (')[0]
     for subtype in ['Food','Treasure','Fish']:
      count=sum(bool(v['token'])and subtype in v['token']['subtypes'].split()for v in enemy)
      expected=int(promise and subtype in kind)*(2 if name=='Into the Flood Maw'and subtype=='Fish'else 1)
      self.assertEqual(count-pre['tokens'][subtype],expected,(name,p,subtype))
      for v in enemy:
       if v['token']and subtype=='Fish'and 'Fish'in v['token']['subtypes'].split():self.assertTrue(v['tapped']);self.assertEqual((v['power'],v['toughness']),(1,1))
     self.assertEqual(len(hand(q))-pre['qhand'],int(promise and kind=='Gift a card')+int(name in ["Bilbo's Gambit",'Into the Flood Maw']))
     if name=='Blooming Blast':self.assertEqual(objects[targets[0]]['zone'],'Graveyard');self.assertEqual(o['players'][qi]['life'],pre['qlife']-(3 if promise else 0))
     if name=='Consumed by Greed':
      self.assertEqual(objects[pre['big']]['zone'],'Graveyard')
      if promise:self.assertEqual(objects[targets[-1]]['zone'],'Hand')
     if name=='Crumb and Get It':self.assertEqual(objects[targets[0]]['power'],4);self.assertEqual(bool(objects[targets[0]]['keywords']&8192),promise)
     if name=="Dawn's Truce":
      self.assertTrue(o['players'][pi]['hexproof_until_end_turn'])
      for id in pre['own']:
       self.assertTrue(objects[id]['keywords']&4096);self.assertEqual(bool(objects[id]['keywords']&8192),promise)
     if name in ['Dewdrop Cure','Peerless Recycling']:
      for id in targets:self.assertEqual(objects[id]['zone'],'Battlefield'if name=='Dewdrop Cure'else'Hand')
     if name=='Into the Flood Maw':self.assertEqual(objects[targets[0]]['zone'],'Hand')
     if name in ["Bilbo's Gambit","Long River's Pull"]:self.assertEqual(objects[victim]['zone'],'Hand'if name=="Bilbo's Gambit"else'Graveyard')
     if name=="Bilbo's Gambit":self.assertEqual(o['casting_prohibited'],promise)
     if name=='Longstalk Brawl':
      self.assertEqual(objects[targets[1]]['zone'],'Graveyard');self.assertEqual(objects[targets[0]]['zone'],'Battlefield'if promise else'Graveyard')
      if promise:self.assertEqual(objects[targets[0]]['counters']['plus_one'],1)
     if name=='Mind Spiral':
      self.assertEqual(len(hand(p)),pre['phand']+3)
      if promise:self.assertTrue(objects[targets[0]]['tapped']);self.assertEqual(objects[targets[0]]['counters']['stun'],1)
     if name=='Nocturnal Hunger':self.assertEqual(objects[targets[0]]['zone'],'Graveyard');self.assertEqual(o['players'][pi]['life'],pre['plife']-(0 if promise else 2))
     if name=="Sazacap's Brew":self.assertEqual(len(hand(p)),pre['phand']+1);self.assertIsNotNone(discard);self.assertEqual(objects[discard]['zone'],'Graveyard');self.assertEqual(objects[targets[0]]['power'],4 if promise else 2)
     if name=='Scrapshooter':self.assertEqual(objects[targets[0]]['zone'],'Graveyard'if promise else'Battlefield');self.assertEqual(len(entry_packets),2 if promise else 0)
     if name=='Valley Rally':
      self.assertEqual(objects[targets[0]]['power'],4);self.assertEqual(bool(objects[targets[0]]['keywords']&512),promise)
     if name=='Wear Down':
      for id in targets:self.assertEqual(objects[id]['zone'],'Graveyard')
     if name=='Wildfire Howl':
      for id in targets:self.assertEqual(objects[id]['zone'],'Graveyard')
      self.assertEqual(o['players'][qi]['life'],pre['qlife']-int(promise))
     stage='finish'
    ready=bool(stage=='setup'and any(v['card']==name for v in hand(p))and can_pay(name,p)and(not ally_need or ally)and(not foe_need or foe)and len(graves())>=grave_need and len(enemy_artifacts)>=artifacts and(name!='Consumed by Greed'or big))
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceActivate','AnnounceExile','AnnounceAdventure','AnnounceOmen','AnnounceWarp','AnnouncePlot','AnnouncePlotted','AnnounceFlashback','AnnounceFlashSurcharge']:return -2000
     if k=='AnnounceCast':
      if actor==p and n==name and(ready and not counter or stage=='counter_window'):return 4000
      if counter and actor==q and stage=='setup'and ready and n==victim_name:return 4100
      if actor==p and stage=='setup'and len(graves())<grave_need and own and n=='Murder'and any(v['card']in ['Bear Cub','Chrome Companion']for v in own):return 3900
      if stage=='finish':return 2400 if n in ['Bear Cub','Chrome Companion','Bygone Colossus']else -1000
      if stage!='setup':return -1000
      if n=='Bear Cub':return 2500 if not(ally if actor==p else foe)else -1000
      if n=='Chrome Companion':return 2400 if actor==p and len(graves())<grave_need and not any(v['card']in ['Bear Cub','Chrome Companion']for v in own)else -1000
      if n=='Bygone Colossus':return 2450 if actor==q and name=='Consumed by Greed'and big is None else -1000
      if n=='Doubling Season':return 2450 if actor==q and len(enemy_artifacts)<artifacts else -1000
      if n=='Quick Study':
       need=not ready if actor==p else((foe_need and foe is None)or len(enemy_artifacts)<artifacts or(name=='Consumed by Greed'and big is None))
       return 2200 if quiet and need and o['players'][int(actor[-1])]['library_count']>8 else -1000
      return -1000
     if k=='ChooseGift':return 3500 if a['beneficiary']==(q if promise else None)else -1000
     if k=='ChooseTarget':
      if(o['announcing']or{}).get('source_card')=='Murder':return 3500 if a['target'].get('Card')==kill else -1000
      if(o['announcing']or{}).get('card')==source:
       if 'Spell'in a['target']:return 3500 if a['target']['Spell']==victim else -1000
       if 'Player'in a['target']:return 3500 if a['target']['Player']==(q if name in ['Consumed by Greed','Wildfire Howl']else p)else -1000
       return 3500 if a['target'].get('Card')in targets else -1000
      return -1000
     if k=='ChooseAdditionalCost':return 3400
     if k=='PlayLand':
      if actor==p and stage=='setup'and len(graves())<grave_need and sum(available(p))>=6 and can_pay('Murder',p)and can_pay(name,p):return -1000
      colors=re.findall(r'\{([WUBRG])\}',FACTS[name]['mana_cost']);return 1800-10*sum(v['card']==n for v in field(actor))+(25 if n in [BASICS['WUBRG'.index(c)]for c in colors]else 0)
     if k in ['TapMana','ActivateMana','AnnounceMana']:
      if o['decision']!='pay_costs':return -1000
      req=o['payment_request'];pool=o['players'][int(actor[-1])]['mana'];return 2400 if n in BASICS and pool[BASICS.index(n)]<req['required'][BASICS.index(n)]else 2200
     if k=='ChooseManaProduction':
      req=o.get('payment_request');return 2000+sum(a['production'][i]*max(0,req['required'][i]-o['players'][int(actor[-1])]['mana'][i])for i in range(6))if req else 2000
     if k in ['PayCosts','PutTrigger','FinishTargets','Keep','ChooseSpellCost','ChooseReplacement','ChooseEffectCard','FinishEffectChoice']:return 3000
     if k in ['FinishAttackers','FinishBlockers']:return 2500
     if k=='Attack':return 2800 if stage=='finish'else -1000
     if k in ['Discard','Bottom']:
      if actor==p and stage=='setup'and len(graves())<grave_need and n in ['Bear Cub','Chrome Companion']:return 2600
      if actor==p and stage!='finish'and n==name and sum(v['card']==name for v in hand(p))<=1:return -500
      return 1500
     if k=='Pass':return 0
     return 1000
    actions=o['actions'];index=max(range(len(actions)),key=lambda j:score(actions[j]));a=actions[index];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision'],iteration))
    if a['kind']=='AnnounceCast':
     n=objects[a['card']]['card']
     if actor==q and counter and stage=='setup'and ready and n==victim_name:victim=a['card'];stage='counter_window'
     elif actor==p and n=='Murder'and stage=='setup':kill=next(v['id']for v in own if v['card']in ['Bear Cub','Chrome Companion']);stage='wait_grave'
     elif actor==p and n==name:
      source=a['card'];stage='wait_body'
      targets=([v['id']for v in graves()[:grave_need]]if name in ['Dewdrop Cure','Peerless Recycling']else[graves()[0]['id']]if name=='Consumed by Greed'and promise else[ally['id'],foe['id']]if name=='Longstalk Brawl'else[enemy_artifacts[0]['id']]if name=='Into the Flood Maw'and promise or name=='Scrapshooter'else[v['id']for v in enemy_artifacts[:artifacts]]if name=='Wear Down'else[foe['id']]if name in ['Blooming Blast','Nocturnal Hunger','Into the Flood Maw','Mind Spiral']and foe else[ally['id']]if name in ['Crumb and Get It',"Sazacap's Brew",'Valley Rally']else[v['id']for v in [ally,foe]if v]if name=='Wildfire Howl'else[])
    if o['decision']=='gift'and(o['announcing']or{}).get('card')==source:
     ann=o['announcing'];self.assertEqual(ann['targets'],[]);self.assertIsNone(ann['cast'].get('gift'));self.assertIsNone(o['payment_request']);self.assertEqual(g.observe(q)['actions'],[])
     before=[g.observe(v)for v in [p,q]]
     with self.assertRaisesRegex(EngineError,'IllegalAction'):g.act(o,{'kind':'ChooseGift','beneficiary':p})
     self.assertEqual(before,[g.observe(v)for v in [p,q]]);extra+=1;declared=True
    if a['kind']=='ChooseAdditionalCost'and(o['announcing']or{}).get('card')==source:
     discard=a['object']['card'];self.assertEqual((objects[discard]['owner'],objects[discard]['zone']),(p,'Hand'))
    if a['kind']=='PayCosts'and(o['announcing']or{}).get('card')==source:
     ann=o['announcing'];self.assertEqual(ann['cast']['gift'],{'Promised':{'beneficiary':q}}if promise else'Declined');self.assertEqual(sum(a['payment']),int(FACTS[name]['cmc']));paid=True;cast_ref=ann['object']
     pre=dict(phand=len(hand(p)),qhand=len(hand(q)),plife=o['players'][pi]['life'],qlife=o['players'][qi]['life'],tokens={s:sum(bool(v['token'])and s in v['token']['subtypes'].split()for v in enemy)for s in ['Food','Treasure','Fish']},own=[v['id']for v in own],big=big['id']if big else None)
    if a['kind']=='Attack'and actor==p:attacked=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,promise,stage,'no natural terminal game'))
if __name__=='__main__':unittest.main()
