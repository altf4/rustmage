"""Legal Standard terminal acceptance for ten Omen pairs and two adjacent graveyard bodies through the in-process native C ABI.

This is an acceptance harness, not an isolated engine-throughput benchmark.
Run with --library ABSOLUTE_IMMUTABLE_LIBRARY; all fork/replay calls stay native.
"""
import json,unittest
from pathlib import Path
import sys,argparse
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from rustmage import Game,EngineError
import test_adventure
NAMES=['Dirgur Island Dragon // Skimming Strike','Disruptive Stormbrood // Petty Revenge','Marang River Regent // Coil and Catch','Purging Stormbrood // Absorb Essence','Riling Dawnbreaker // Signaling Roar','Runescale Stormbrood // Chilling Screech','Sagu Wildling // Roost Seek','Stormshriek Feral // Flush Out','Twinmaw Stormbrood // Charring Bite','Feral Deathgorger // Dusk Sight','Arashin Sunshield','Griffnaut Tracker']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
PREFERRED=['Island','Forest','Island','Swamp','Plains','Mountain','Forest','Mountain','Plains']
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
class OmenStandard(unittest.TestCase):
 replay=test_adventure.AdventureTests.replay
 denied=test_adventure.AdventureTests.denied
 def scenario(self,i,p,inset,seed):
  q='P1'if p=='P0'else'P0';pi=int(p[-1]);name=NAMES[i];grave_owner=p if (seed//48)%2==0 else q
  own=sum(([b]*n for b,n in zip(BASICS,[6,6,6,6,8])),[])+sum(([n]*4 for n in [name,'Bear Cub','Chrome Companion','Shock','Snakeskin Veil','Bygone Colossus','Thrill of Possibility']),[])
  foe=list(own);self.assertEqual(len(own),60)
  with Game([own,foe]if p=='P0'else[foe,own],seed=seed,starting_player=p,deck_rules='Standard')as g:
   stage='setup';source=None;base=None;descriptor=False;boosted=False;activated=False;target_ids=set();search_chosen=False;enemy_spell=None;attacked=False;private=0;activation_started=False
   for iteration in range(30000):
    o=g.observe('P0')
    if o['outcome'] is not None:
     self.assertEqual(stage,'finish',(name,p,inset,seed,'prematureterminal',stage))
     if not inset:self.assertTrue(attacked,(name,p,seed,'no actual source attack'))
     return dict(decisions=iteration,private=private,outcome=o['outcome'])
    o=g.observe(o['actor']);self.assertEqual(o['schema'],89);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p)
    def field(owner,n):return next((v for v in objects.values()if v['owner']==owner and v['zone']=='Battlefield'and v['card']==n),None)
    ally=field(p,'Bear Cub');enemy=field(q,'Bear Cub');artifact=field(q,'Chrome Companion');body=objects.get(source)
    graves={v:[x for x in objects.values()if x['owner']==v and x['zone']=='Graveyard']for v in [p,q]}
    quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    if stage=='effect'and quiet:
     self.assertTrue(descriptor);self.assertFalse(o['adventures']);self.assertFalse(o['exile_permissions'])
     if inset:
      self.assertFalse(any(v['id']==source for v in g.observe(p)['objects']));self.assertFalse(any(v['id']==source for v in g.observe(q)['objects']))
      self.assertEqual(o['players'][pi]['library_count'],base['library']+1-({0:1,2:3,6:int(search_chosen),7:2,9:1}.get(i,0)))
      if i==0:self.assertTrue(objects[base['enemy']]['tapped'])
      if i in [1,8]:self.assertEqual(objects[base['enemy']]['zone'],'Graveyard')
      if i==3:self.assertEqual((ally['power'],ally['toughness']),(base['ally_power']+2,base['ally_toughness']+2));self.assertEqual(ally['keywords']&(64|4096),64|4096)
      if i==4:
       soldiers=[v for v in objects.values()if v['owner']==p and v['zone']=='Battlefield'and v.get('token')and v['token']['subtypes']=='Soldier'];self.assertEqual(len(soldiers),1);self.assertEqual((soldiers[0]['power'],soldiers[0]['toughness']),(2,2));self.assertEqual(soldiers[0]['token']['colors'],1)
      if i==9:self.assertEqual(objects[base['enemy']]['counters']['plus_one'],base['enemy_counters']+1)
      if i==5:self.assertEqual(objects[enemy_spell]['zone'],'Graveyard');self.assertEqual(o['players'][pi]['life'],base['life'])
      stage='finish'
      continue
     self.assertEqual(body['zone'],'Battlefield');self.assertEqual(body['card'],name)
     front=FACTS[name]['card_faces'][0]if i<10 else FACTS[name];power=int(front['power'])+(2 if i==5 and boosted else 1 if i==7 and activated else 0)
     self.assertEqual((body['power'],body['toughness']),(power,int(front['toughness'])))
     if i!=10:self.assertTrue(body['keywords']&1)
     if i==9:self.assertTrue(body['keywords']&128)
     if i>=9:
      self.assertEqual(len(target_ids),2);self.assertTrue(all(objects[t]['zone']=='Exile'and objects[t]['owner']==grave_owner for t in target_ids))
     if i==1:self.assertEqual(objects[base['artifact']]['zone'],'Graveyard')
     if i==2:self.assertEqual(len(target_ids),2);self.assertTrue(all(objects[t]['zone']=='Hand'for t in target_ids))
     if i==3:self.assertEqual(ally['counters']['plus_one'],0)
     if i in [6,8]:self.assertEqual(o['players'][pi]['life'],base['life']+(3 if i==6 else 5))
     if i==4 and ally['power']==base['ally_power']:pass
     elif i==5 and not boosted:pass
     elif i in [7,10]and not activated:pass
     else:
      if i==4:self.assertEqual(ally['power'],base['ally_power']+1)
      if i==7:self.assertTrue(body['keywords']&4)
      if i==10:self.assertTrue(body['tapped']);self.assertTrue(objects[base['enemy']]['tapped'])
      stage='finish'
      continue
    lands=[v for v in objects.values()if v['owner']==p and v['zone']=='Battlefield'and v['card']in BASICS and not v['tapped']]
    ready=len(lands)>=8 and ally and enemy and artifact and (inset or i!=3 or ally['counters']['plus_one']>0)and (inset or i<9 or all(len(graves[v])>=2 for v in [p,q]))
    for d in o['spell_characteristics']:
     if stage=='effect'and source is not None and d['object']['card']==source:
      f=FACTS[name]['card_faces'][int(inset)]if i<10 else FACTS[name];self.assertEqual(d['physical_card'],name);self.assertEqual(d['name'],f['name']);self.assertEqual(d['type_line'],f['type_line']);self.assertEqual(d['mana_cost'],f['mana_cost']);self.assertEqual(d['face'],'Omen'if inset else'Normal');descriptor=True
      if inset:
       item=o['announcing']or next((x for x in o['stack']if x['card']==source),None)or o['resolving'];self.assertEqual(item['cast']['method'],'Normal');self.assertEqual(item['cast']['physical_card'],name)
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if stage=='finish':
      if k=='Attack':return 2800
      if k=='AnnounceCast':return 2300 if n in [name,'Bear Cub','Chrome Companion','Bygone Colossus']else -1000
      if k in ['Cast','BeginCast','AnnounceOmen','AnnounceAdventure','AnnounceActivate','Activate','Mulligan','Concede']:return -2000

     if k in ['Concede','Mulligan','Attack','Cast','BeginCast','AnnounceAdventure','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='AnnounceOmen':return 2600 if actor==p and inset and stage=='setup'and n==name and ready else -1000
     if k=='AnnounceCast':
      if i>=9 and not inset and n=='Shock'and stage=='setup':return 2500 if len(graves[actor])<2 and (ally if actor==p else enemy)else -1000
      if actor==p:
       if n==name:return 2600 if not inset and stage=='setup'and ready else -1000
       if n=='Bear Cub':return 2000 if not ally else -1000
       if n=='Snakeskin Veil':return 2300 if not inset and i==3 and ally and ally['counters']['plus_one']==0 and stage=='setup'else -1000
       if n=='Shock':return 2500 if not inset and i==5 and stage=='effect'and body and body['zone']=='Battlefield'and not boosted else -1000
       return -1000
      if n=='Bear Cub':return 2000 if not enemy else -1000
      if n=='Chrome Companion':return 2000 if not artifact else -1000
      if n=='Shock':return 2500 if inset and i==5 and stage=='setup'and ready and quiet and any(v['zone']=='Hand'and v['card']==name for v in mine['objects'])else -1000
      return -1000
     if k in ['AnnounceActivate','Activate']:return 2500 if not inset and i in [7,10]and actor==p and stage=='effect'and a['card']==source and body and body['zone']=='Battlefield'and not activated else -1000
     if k=='ChooseTarget':
      t=a['target'];ann=o['announcing'];src=ann['source_card']if ann else''
      if 'Spell'in t:return 2700
      if 'Player'in t:return 2600 if t['Player']==(p if actor==q else q)else 1000
      v=objects[t['Card']]
      if i>=9 and not inset and src=='Shock'and stage=='setup':return 2800 if v['id']==(ally if actor==p else enemy)['id']else -1000
      if i>=9 and not inset and src==name:
       if activation_started:return 2800 if v['id']==base['enemy']else -1000
       return 2800 if v['zone']=='Graveyard'and v['owner']==grave_owner else 500
      if src=='Snakeskin Veil':return 2700 if ally and v['id']==ally['id']else 500
      if not inset and i==2 and src==name:return 2700 if v['card']=='Bear Cub'else 500
      desired=artifact if not inset and i==1 else ally if i==3 or not inset else enemy
      return 2700 if desired and t['Card']==desired['id']else 500
     if k=='FinishTargets':return 2000
     if k=='PlayLand':return 1800-20*sum(v['owner']==actor and v['zone']=='Battlefield'and v['card']==n for v in objects.values())
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2100 if o['decision']in ['pay_costs','resolution_mana']and n in BASICS else -1000
     if k in ['PayCosts','PutTrigger','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost']:return 2400
     if k in ['Discard','Bottom']:
      # Retain one fixture spell until its real paid route has been witnessed.
      reserve=(stage=='setup'and((actor==p and n==name)or(n=='Bear Cub'and not(ally if actor==p else enemy))or(actor==q and n=='Chrome Companion'and not artifact)))or(i==5 and n=='Shock'and stage!='finish'and actor==(q if inset else p))or(not inset and i==3 and actor==p and stage=='setup'and n=='Snakeskin Veil')or(i>=9 and not inset and stage=='setup'and n=='Shock'and len(graves[actor])<2)
      copies=sum(v['owner']==actor and v['zone']=='Hand'and v['card']==n for v in objects.values())
      return 100 if reserve and copies==1 else 2200 if n in BASICS else 1000
     if k=='ChooseSearch':return 2500 if not search_chosen else -1000
     if k=='FinishSearch':return 2000
     if k=='DeclineEffectPayment':return -1000
     if k=='Pass':return 0
     return 1000
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,p,inset,stage,o['decision']));ann=o['announcing']
    if stage!='finish'and a['kind']in ['AnnounceOmen','AnnounceCast']and actor==p and objects[a['card']]['card']==name:source=a['card'];stage='effect'
    if stage!='finish'and a['kind']=='PayCosts'and ann:
     if ann['kind']=='Spell'and ann['card']==source:
      base=dict(library=o['players'][pi]['library_count'],life=o['players'][pi]['life'],ally_power=ally['power'],ally_toughness=ally['toughness'],enemy=enemy['id'],enemy_counters=enemy['counters']['plus_one'],artifact=artifact['id']);target_ids={b['target']['Card']for b in ann['targets']if 'Card'in b['target']}
     if ann['kind']=='Spell'and ann['source_card']=='Shock':
      if actor==q:enemy_spell=ann['card']
      elif i==5:boosted=True
     if ann['card']==source and ann['kind']!='Spell':activated=True
    if a['kind']in ['AnnounceActivate','Activate']and a['card']==source:activation_started=True
    if a['kind']=='ChooseTarget'and not inset and not activation_started and i in [2,9,10,11] and ann and ann['source_card']==name and ann['kind']!='Spell':target_ids.add(a['target']['Card'])
    if a['kind']=='ChooseSearch':search_chosen=True
    if a['kind']=='Attack'and actor==p and a['card']==source:attacked=True
    if o['decision']=='targets':
     private+=1
     other=g.observe(q if actor==p else p)
     self.assertFalse(other['actions']);self.assertFalse(other['announcing']['targets']);self.assertIsNone(other['targeting_group'])
    if o['search_choice'] or o['effect_choice']:
     private+=1
     other=g.observe(q if actor==p else p)
     self.assertIsNone(other['search_choice']);self.assertIsNone(other['effect_choice'])
    before=[g.observe(v)for v in ['P0','P1']]
    with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor=q if actor==p else p),a)
    self.assertEqual(before,[g.observe(v)for v in ['P0','P1']]);self.replay(g,o,j)
    if i>=9 and not inset and len(target_ids)==1:
     after=g.observe(g.observe('P0')['actor'])
     if after['decision']=='targets':
      forbidden=graves[q if grave_owner==p else p][0]['id'];self.denied(g,after,dict(kind='ChooseTarget',target={'Card':forbidden}))
   self.fail(('unreached',name,p,inset,stage))
if __name__=='__main__':
 args=argparse.ArgumentParser();args.add_argument('--library',required=True);args.add_argument('--games',type=int,default=100);args.add_argument('--start',type=int,default=0);options=args.parse_args()
 OriginalGame=Game
 def explicit_game(*a,**kw):kw['library']=options.library;return OriginalGame(*a,**kw)
 Game=explicit_game
 totals=dict(decisions=0,private=0);matrix=[[[0,0]for _ in range(2)]for _ in range(len(NAMES))]
 helper=OmenStandard()
 for seed in range(options.start,options.start+options.games):
  i=seed%len(NAMES);p='P0'if (seed//len(NAMES))%2==0 else'P1';inset=i<10 and (seed//(2*len(NAMES)))%2==0
  result=helper.scenario(i,p,inset,seed)
  matrix[i][int(inset)][int(p[-1])]+=1
  for k in totals:totals[k]+=result[k]
  print('game',seed,NAMES[i],p,'inset',inset,result,flush=True)
 if options.games>=48:assert all(all(all(n>0 for n in seats)for mode,seats in enumerate(modes)if mode==0 or i<10)for i,modes in enumerate(matrix))
 print('Completed Standard Omen games',options.games,'native owned replays/parent-closed',totals['decisions'],'denials',totals['decisions']*2,'private frames',totals['private'],'full-body mode/seat vectors',matrix,flush=True)
