"""Paid complete Worldsoul and Bonny bodies through naturally drawn Standard games."""
import json, os, re, unittest
from pathlib import Path
from rustmage import Game, EngineError
from test_role_consumers import RoleConsumerTests
ROOT=Path(__file__).resolve().parents[1]
FACTS={c['name']:c for c in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
ROWS=[("Worldsoul's Rage",0,'decline'),("Worldsoul's Rage",2,'both'),("Worldsoul's Rage",3,'decline'),("Worldsoul's Rage",3,'subset'),('Bonny Pall, Clearcutter',None,'hand'),('Bonny Pall, Clearcutter',None,'grave'),('Bonny Pall, Clearcutter',None,'decline')]
class JointLandTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def raw_frame(self,g,o,action):
  if action in o['actions']:self.frame(g,o,o['actions'].index(action));return
  self.assertEqual(action['kind'],'ChooseX');actor=o['actor'];other='P1'if actor=='P0'else'P0';before=[g.observe(p)for p in['P0','P1']]
  with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor=other),action)
  self.assertEqual(before,[g.observe(p)for p in['P0','P1']]);saved=g.snapshot();parent=g.clone();child=parent.clone();parent.close()
  with child:child.act(child.observe(actor),action);expected=[child.observe(p)for p in['P0','P1']]
  g.act(o,action);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.act(o,action)
  g.act(g.observe(actor),action)
  for actual,wanted in zip([g.observe(p)for p in['P0','P1']],expected):wanted['revision']=actual['revision'];self.assertEqual(actual,wanted)
  g.drop_snapshot(saved)
 def test_complete_two_bodies_all_selection_cases_paid_both_seats(self):
  for i,row in enumerate(ROWS):
   for p in['P0','P1']:
    with self.subTest(row=row,seat=p):self.scenario(i,p,33600+i)
 def scenario(self,i,p,seed,library=None):
  name,x,selection=ROWS[i];q='P1'if p=='P0'else'P0';pi=int(p[-1]);qi=1-pi;bonny=x is None
  deck=['Forest']*10+['Island']*12+['Mountain']*10+sum(([n]*4 for n in[name,'Bear Cub','Attercop','Bygone Colossus','Quick Study','Thrill of Possibility','Candy Trail']),[])
  enemy=['Forest']*20+['Island']*12+sum(([n]*4 for n in['Bear Cub','Bygone Colossus','Chrome Companion','Llanowar Elves','Quick Study','Candy Trail','Attercop']),[])
  self.assertEqual((len(deck),len(enemy)),(60,60));stage='setup';source=None;paid=False;selected=[];choice_seen=False;attacked=False;frames=0;base=None
  with Game([deck,enemy]if p=='P0'else[enemy,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   for _ in range(35000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(101,FP))
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage));self.assertTrue(paid);self.assertTrue(attacked)
     return dict(name=name,x=x,selection=selection,seat=p,seed=seed,frames=frames,selected=selected)
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p);quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    def board(player):return[v for v in objects.values()if v['zone']=='Battlefield'and v['controller']==player]
    def hand(player):return[v for v in g.observe(player)['objects']if v['zone']=='Hand'and v['owner']==player]
    def grave(player):return[v for v in objects.values()if v['zone']=='Graveyard'and v['owner']==player]
    def budget(player):
     mana=list(o['players'][int(player[-1])]['mana'])
     for v in board(player):
      if v['card']in BASICS and not v['tapped']:mana[BASICS.index(v['card'])]+=1
     return mana
    def payable(n,player,extra=0):
     tokens=re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[n].get('mana_cost',''));colors=[tokens.count(c)for c in'WUBRGC'];mana=budget(player);return all(a>=b for a,b in zip(mana,colors))and sum(mana)>=sum(colors)+sum(int(c)for c in tokens if c.isdecimal())+extra
    body=objects.get(source);hand_lands=[v for v in hand(p)if v['card']in BASICS];grave_lands=[v for v in grave(p)if v['card']in BASICS];need_grave=selection in['both','grave'];ready_bear=any(v['card']=='Bear Cub'and not v['summoning_sick']for v in board(p))
    ready=quiet and payable(name,p,x or 0)and sum(v['card']in BASICS for v in board(p))>=7 and len(hand_lands)>=int(selection in['both','subset','hand']) and(not need_grave or grave_lands)and(not bonny or ready_bear)
    if quiet and stage=='wait_body':
     self.assertEqual(body['zone'],'Battlefield');self.assertTrue(body['keywords']&2);self.assertTrue(paid);stage='attack'
    if quiet and stage=='wait_effect':
     self.assertTrue(paid);self.assertEqual(body['zone'],'Battlefield'if bonny else'Graveyard')
     if bonny:self.assertEqual(o['players'][pi]['cards_drawn_this_turn']-base['drawn'],1)
     else:self.assertEqual(base['life']-o['players'][qi]['life'],x)
     self.assertEqual(choice_seen,bonny or x>0)
     expected=0 if selection=='decline'else 2 if selection=='both'else 1;self.assertEqual(len(selected),expected)
     for ref,origin in selected:
      land=objects[ref['card']];self.assertEqual(land['zone'],'Battlefield');self.assertEqual((land['owner'],land['controller']),(p,p));self.assertNotEqual(land['reference'],ref);self.assertEqual(land['tapped'],not bonny)
     if bonny:
      tokens=[v for v in board(p)if v.get('token')and v['token']['name']=='Beau'];self.assertEqual(len(tokens),1);count=sum(v['kinds']&2!=0 for v in board(p));self.assertEqual((tokens[0]['power'],tokens[0]['toughness']),(count,count))
     stage='finish'
    effect=o.get('effect_choice');land_choice=effect and isinstance(effect['kind'],dict)and'LandCards'in effect['kind']
    if land_choice and stage!='finish':
     choice_seen=True;self.assertEqual(actor,p);self.assertTrue(effect['optional']);self.assertEqual(effect['kind']['LandCards']['maximum'],1 if bonny else x)
     peer=g.observe(q)['effect_choice'];self.assertIsNotNone(peer)
     for ref in peer['candidates']+sum(peer['selected'],[]):self.assertEqual(objects[ref['card']]['zone'],'Graveyard')
     for ref in effect['candidates']:
      v=objects[ref['card']];self.assertEqual(v['owner'],p);self.assertIn(v['zone'],['Hand','Graveyard']);self.assertIn(v['card'],BASICS);self.assertEqual(v['reference'],ref)
     for ref,origin in selected:self.assertEqual(objects[ref['card']]['zone'],origin)
    if o.get('x_choice'):
     self.assertEqual(actor,p);self.assertTrue(any(v['minimum']<=x<=v['maximum']for v in o['x_choice']['intervals']));self.raw_frame(g,o,dict(kind='ChooseX',value=x));frames+=1;continue
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceActivate','AnnounceAdventure','AnnounceOmen','AnnounceExile','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and n==name and ready:return 5000
      if actor==p and stage=='setup'and need_grave and not grave_lands and n=='Thrill of Possibility'and len(hand_lands)>=2 and payable(n,p):return 3500
      if actor==p and stage=='setup'and bonny and n=='Bear Cub'and not any(v['card']=='Bear Cub'for v in board(p))and payable(n,p):return 3000
      if actor==p and stage=='setup'and n=='Quick Study'and quiet and sum(budget(p))>=5 and payable(n,p):return 2200
      if stage=='finish'and actor==p and n in['Bear Cub','Bygone Colossus','Attercop']and payable(n,p):return 2800
      return -1000
     if k=='ChooseTarget':return 4500 if a['target']=={'Player':q}else -1000
     if k=='SelectLand':
      if stage=='finish':return -1000
      zone=objects[a['object']['card']]['zone'];wanted='Hand'if selection in['hand','subset']or selection=='both'and not selected else'Graveyard'
      return 4500 if selection!='decline'and zone==wanted and(selection!='subset'or not selected)else -1000
     if k=='FinishLandSelection':return 4600 if stage=='finish'or selection=='decline'or selection=='subset'and selected else -1000
     if k=='PlayLand':
      required={'Island':2,'Forest':1}if bonny else{'Mountain':1,'Forest':1}
      missing_color=n in required and sum(v['card']==n for v in board(p))<required[n]
      return 1900-15*sum(v['card']==n for v in board(actor))if actor!=p or stage=='finish'or sum(v['card']in BASICS for v in board(p))<7 or missing_color else -1000
     if k in['AnnounceMana','TapMana','ActivateMana']:return 3500 if o['decision']in['pay_costs','resolution_mana']else -1000
     if k in['PayCosts','PutTrigger','FinishTargets','Keep','ChooseSpellCost','ChooseEntryLife','KeepLegend']:return 4200
     if k=='Attack':return 3500 if actor==p and(stage=='finish'or stage=='attack'and n=='Bear Cub')else -1000
     if k in['FinishAttackers','FinishBlockers']:return 2600
     if k in['Discard','Bottom']:
      if actor==p and stage!='finish'and n==name and sum(v['card']==n for v in hand(p))<=1:return -1000
      if actor==p and stage=='setup'and need_grave and not grave_lands and n in BASICS:return 2500
      if actor==p and stage!='finish'and n in BASICS and len(hand_lands)<=2:return -1000
      if actor==p and stage=='setup'and bonny and n=='Bear Cub'and not any(v['card']=='Bear Cub'for v in board(p))and sum(v['card']==n for v in hand(p))<=1:return -1000
      return 1500
     if k=='Pass':return 0
     return 1000
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision']))
    if a['kind']=='AnnounceCast'and actor==p and objects[a['card']]['card']==name and stage=='setup':source=a['card'];base=dict(life=o['players'][qi]['life']);stage='wait_body'if bonny else'wait_effect'
    if a['kind']=='PayCosts'and(o['announcing']or{}).get('card')==source:self.assertEqual(sum(a['payment']),6 if bonny else 2+x);paid=True
    if a['kind']=='FinishAttackers'and actor==p and stage=='attack':base=dict(drawn=o['players'][pi]['cards_drawn_this_turn']);stage='wait_effect';attacked=True
    if a['kind']=='SelectLand':selected.append((a['object'],objects[a['object']['card']]['zone']))
    if a['kind']=='Attack'and actor==p and stage=='finish':attacked=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
