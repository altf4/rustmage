"""Paid split-search consumers in legal Standard games through the native C ABI.

Every action checks owned fork replay, parent destruction, restore/stale indices,
wrong actor atomicity and both observations. This measures acceptance, not speed.
"""
import argparse
import json
import sys
import unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from rustmage import Game,EngineError
from test_adventure import AdventureTests
BLOOM='Bloomvine Regent // Claim Territory'
TROOP='Troop of Ponies'
SAGU='Sagu Wildling // Roost Seek'
EXPECTED_CATALOGUE=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
class SplitStandard(unittest.TestCase):
 replay=AdventureTests.replay
 denied=AdventureTests.denied
 def scenario(self,route,p,seed,*,library,terminal=True,found=2):
  q='P1'if p=='P0'else'P0';pi=int(p[-1]);name=TROOP if route=='troop'else BLOOM
  deck=['Forest']*32+sum(([n]*4 for n in [name,'Bear Cub','Chrome Companion',SAGU,'Bygone Colossus','Snakeskin Veil','Thrill of Possibility']),[])
  self.assertEqual(len(deck),60)
  with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library)as g:
   self.assertEqual(g.observe('P0')['catalogue'],EXPECTED_CATALOGUE);self.assertEqual(g.observe('P0')['schema'],85)
   stage='setup';source=None;descriptor=False;base=None;chosen=[];destination=None;attacked=False;private=0;reveals=0;dragon=None;opponent_dragon=None
   for iteration in range(30000):
    o=g.observe('P0')
    if o['outcome']is not None:
     self.assertEqual(stage,'finish',(route,p,seed,stage,iteration))
     if route!='omen':self.assertTrue(attacked)
     return dict(decisions=iteration,private=private,reveals=reveals,outcome=o['outcome'])
    o=g.observe(o['actor']);self.assertEqual(o['schema'],85);actor=o['actor'];objects={v['id']:v for v in o['objects']};body=objects.get(source)
    quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    if stage=='normal'and quiet:
     self.assertTrue(descriptor);self.assertEqual(body['zone'],'Battlefield')
     self.assertEqual((body['power'],body['toughness']),(2,1)if route=='troop'else(4,5))
     if route=='troop':
      self.assertFalse(any(a['kind']=='AnnounceActivate'and a.get('card')==source for a in o['actions']))
      stage='attack'
     else:
      self.assertTrue(body['keywords']&1);self.assertEqual(o['players'][pi]['life'],base['life']+3);stage='dragon'
    if stage=='dragon_effect'and quiet:
     self.assertEqual(objects[dragon]['zone'],'Battlefield');self.assertEqual(o['players'][pi]['life'],base['life']+9);stage='opponent_dragon'
    if stage=='opponent_dragon_effect'and quiet:
     self.assertEqual(objects[opponent_dragon]['zone'],'Battlefield');self.assertEqual(o['players'][pi]['life'],base['life']+9);stage='attack'
    if stage=='search'and quiet:
     self.assertTrue(descriptor);self.assertEqual(len(chosen),found);self.assertFalse(o['adventures']);self.assertFalse(o['exile_permissions'])
     if found==2:
      self.assertIn(destination,chosen);other=next(c for c in chosen if c!=destination)
      self.assertEqual(objects[destination]['zone'],'Battlefield');self.assertTrue(objects[destination]['tapped']);self.assertEqual(objects[other]['zone'],'Hand')
      other_objects={v['id']:v for v in g.observe(q)['objects']};self.assertEqual(other_objects[other]['card'],'Forest')
     elif found==1:self.assertEqual(objects[chosen[0]]['zone'],'Battlefield');self.assertTrue(objects[chosen[0]]['tapped'])
     self.assertEqual(o['players'][pi]['library_count'],base['library']-found+int(route=='omen'))
     if route=='omen':
      for viewer in [p,q]:self.assertFalse(any(v['id']==source for v in g.observe(viewer)['objects']))
     else:self.assertEqual(objects[source]['zone'],'Graveyard');self.assertTrue(attacked)
     stage='finish'
     if not terminal:return dict(decisions=iteration,private=private,reveals=reveals)
    for d in o['spell_characteristics']:
     if stage in ['normal','search']and source is not None and d['object']['card']==source:
      self.assertEqual(d['physical_card'],name);self.assertEqual(d['face'],'Omen'if route=='omen'else'Normal');descriptor=True
      face=FACTS[name]['card_faces'][int(route=='omen')]if name==BLOOM else FACTS[name]
      self.assertEqual(d['name'],face['name']);self.assertEqual(d['mana_cost'],face['mana_cost'])
    search=o['search_choice'];distribution=o['search_distribution']
    if search:
     private+=1;self.assertIsNone(g.observe(q)['search_choice']);self.assertIsNone(distribution)
    if distribution:
     reveals+=1;self.assertEqual(distribution,g.observe(q)['search_distribution']);self.assertIsNone(search)
     self.assertEqual({v['id']for v in distribution['candidates']},set(chosen));self.assertEqual(distribution['battlefield_count'],1)
     self.assertFalse(any(a['kind']=='FinishSearch'for a in o['actions']))
     for view in [p,q]:self.assertFalse(any(v['zone']=='Library'for v in g.observe(view)['objects']))
     self.denied(g,o,{'kind':'ChooseSearchBattlefield','card':source})
    lands=sum(v['owner']==p and v['zone']=='Battlefield'and v['card']=='Forest'and not v['tapped']for v in objects.values())
    ready=lands>=6
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','AnnounceAdventure','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='Attack':
      return 2800 if stage=='finish'or(stage=='attack'and actor==p and a['card']==source and not attacked)else -1000
     if k=='AnnounceOmen':return 2600 if route=='omen'and stage=='setup'and actor==p and n==name and ready else -1000
     if k=='AnnounceCast':
      if stage=='finish':return 2300 if n in [name,'Bear Cub','Chrome Companion',SAGU,'Bygone Colossus']else -1000
      if actor==p and stage=='setup'and route!='omen'and n==name and ready:return 2600
      if actor==p and stage=='dragon'and n==SAGU:return 2600
      if actor==q and stage=='opponent_dragon'and n==SAGU:return 2600
      return -1000
     if k in ['AnnounceActivate','Activate']:return 2600 if route=='troop'and stage=='activate'and actor==p and a['card']==source else -1000
     if k=='PlayLand':return 1800
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2100 if o['decision']in ['pay_costs','resolution_mana']and n=='Forest'else -1000
     if k in ['PayCosts','PutTrigger','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost','FinishTargets']:return 2400
     if k in ['Discard','Bottom']:
      reserve=(stage=='setup'and actor==p and n==name)or(stage in ['setup','normal','dragon']and actor==p and n==SAGU)or(stage not in ['finish','attack','activate','search']and actor==q and n==SAGU)
      copies=sum(v['owner']==actor and v['zone']=='Hand'and v['card']==n for v in objects.values())
      return 100 if reserve and copies==1 else 2200 if n=='Forest'else 1000
     if k=='ChooseSearch':return 2500 if len(chosen)<found else -1000
     if k=='FinishSearch':return 2000
     if k=='ChooseSearchBattlefield':return 2500
     if k=='ChooseTarget':return 2000
     if k=='DeclineEffectPayment':return -1000
     if k=='Pass':return 0
     return 1000
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(route,p,seed,stage,o['decision']));ann=o['announcing']
    if stage=='setup'and a['kind']in ['AnnounceCast','AnnounceOmen']and actor==p and objects[a['card']]['card']==name:source=a['card'];stage='search'if route=='omen'else'normal'
    if stage=='dragon'and a['kind']=='AnnounceCast'and actor==p:dragon=a['card'];stage='dragon_effect'
    if stage=='opponent_dragon'and a['kind']=='AnnounceCast'and actor==q:opponent_dragon=a['card'];stage='opponent_dragon_effect'
    if stage=='normal'and a['kind']=='PayCosts'and ann and ann['kind']=='Spell'and ann['card']==source:base=dict(life=o['players'][pi]['life'])
    if route=='omen'and stage=='search'and a['kind']=='PayCosts'and ann and ann['card']==source:base=dict(library=o['players'][pi]['library_count'])
    if a['kind']=='Attack'and a['card']==source:
     attacked=True
     stage='activate'if route=='troop'else'finish'
     if not terminal and route!='troop':return dict(decisions=iteration,private=private,reveals=reveals)
    if stage=='activate'and a['kind']in ['AnnounceActivate','Activate']and a['card']==source:stage='search';base=dict(library=o['players'][pi]['library_count'])
    if a['kind']=='ChooseSearch':chosen.append(a['card'])
    if a['kind']=='ChooseSearchBattlefield':destination=a['card']
    before=[g.observe(v)for v in ['P0','P1']]
    with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor=q if actor==p else p),a)
    self.assertEqual(before,[g.observe(v)for v in ['P0','P1']]);self.replay(g,o,j)
   self.fail((route,p,seed,stage,'unreached'))
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--library',required=True);parser.add_argument('--games',type=int,default=100);parser.add_argument('--start',type=int,default=0);args=parser.parse_args()
 helper=SplitStandard();totals=dict(decisions=0,private=0,reveals=0);matrix=[[0,0]for _ in range(3)]
 for seed in range(args.start,args.start+args.games):
  mode=seed%3;p='P0'if(seed//3)%2==0 else'P1';route=['omen','normal','troop'][mode]
  result=helper.scenario(route,p,seed,library=args.library);matrix[mode][int(p[-1])]+=1
  for k in totals:totals[k]+=result[k]
  print('game',seed,route,p,result,flush=True)
 if args.games>=6:assert all(all(n>0 for n in row)for row in matrix)
 print('Completed Standard split search',args.games,'games',totals,'owned/parentclosed replays',totals['decisions'],'denials',totals['decisions']*2,'full routes/seat vectors',matrix,flush=True)
