"""Naturally drawn complete untapped-entry consumers; every action owns its replay."""
import json,os,re,unittest
from pathlib import Path
from rustmage import OBSERVATION_SCHEMA, Game
from test_role_consumers import RoleConsumerTests
ROOT=Path(__file__).resolve().parents[1]
FACTS={c['name']:c for c in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
TOWNS=['Baron, Airship Kingdom','Gohn, Town of Ruin','Gongaga, Reactor Town','Guadosalam, Farplane Gateway','Windurst, Federation Center']
ROWS=[('Captivating Cave',True,False),('Forest',True,False),(None,True,False),('Baron, Airship Kingdom',False,False),('Baron, Airship Kingdom',True,False),('Breeding Pool',False,False),('Breeding Pool',True,False),('Breeding Pool',True,True),('minstrel',True,False)]
class UntappedEntryTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_complete_bodies_all_entry_orders_paid_both_seats(self):
  for i,row in enumerate(ROWS):
   for p in ['P0','P1']:
    with self.subTest(row=row,seat=p):self.scenario(i,p,34200+i)
 def scenario(self,i,p,seed,library=None):
  selection,untapped_last,pay=ROWS[i];minstrel=selection=='minstrel';name='The Wandering Minstrel'if minstrel else'Spelunking';q='P1'if p=='P0'else'P0';pi=int(p[-1]);stage='setup';source=None;selected=None;paid_spell=False;paid_boost=False;token_seen=False;frames=0;attacked=False;baseline=None;replacements=[];payment_seen=False
  deck=(sum(([n]*4 for n in TOWNS),[])+['Forest']*8+['Island']*8 if minstrel else ['Forest']*16+['Island']*8+['Captivating Cave']*4+['Baron, Airship Kingdom']*4+['Breeding Pool']*4)+sum(([n]*4 for n in [name,'Bear Cub','Bygone Colossus','Quick Study','Candy Trail','Attercop']),[])
  enemy=['Forest']*20+['Island']*12+sum(([n]*4 for n in ['Bear Cub','Bygone Colossus','Chrome Companion','Llanowar Elves','Quick Study','Candy Trail','Attercop']),[])
  self.assertEqual((len(deck),len(enemy)),(60,60))
  with Game([deck,enemy]if p=='P0'else[enemy,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   for _ in range(35000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(OBSERVATION_SCHEMA,FP))
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage));self.assertTrue(paid_spell);self.assertTrue(attacked)
     if minstrel:self.assertTrue(token_seen);self.assertTrue(paid_boost)
     return dict(name=name,selection=selection,untapped_last=untapped_last,pay=pay,seat=p,seed=seed,frames=frames,replacements=replacements)
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    def board(player):return[v for v in objects.values()if v['zone']=='Battlefield'and v['controller']==player]
    def hand(player):return[v for v in g.observe(player)['objects']if v['zone']=='Hand'and v['owner']==player]
    lands=[v for v in board(p)if v['kinds']&2];towns=[v for v in lands if v['card']in TOWNS];bears=[v for v in board(p)if v['card']=='Bear Cub'];body=objects.get(source)
    elements=[v for v in board(p)if v.get('token')and v['token']['name']=='Elemental Token'and v['token']['colors']==31]
    if quiet and stage=='wait_source':
     self.assertEqual(body['zone'],'Battlefield');self.assertTrue(paid_spell);stage='wait_token'if minstrel else'wait_effect'
    if minstrel and quiet and stage=='wait_token'and elements:
     token_seen=True;self.assertEqual((elements[0]['power'],elements[0]['toughness']),(2,2));stage='boost'
    if quiet and stage=='wait_effect':
     self.assertTrue(paid_spell);self.assertEqual(body['zone'],'Battlefield')
     if minstrel:
      self.assertTrue(paid_boost);self.assertEqual(body['power'],1);self.assertEqual(objects[baseline['bear']]['power'],baseline['bear_power']+len(towns));self.assertEqual(objects[baseline['element']]['power'],baseline['element_power']+len(towns))
     else:
      self.assertEqual(o['players'][pi]['cards_drawn_this_turn'],baseline['drawn']+1)
      self.assertEqual(o['players'][pi]['life'],baseline['life']+(4 if selection=='Captivating Cave'else 0)-(2 if selection=='Breeding Pool'and pay else 0))
      self.assertEqual(selected is None,selection is None)
      if selected:
       actual=objects[selected['card']];self.assertEqual(actual['zone'],'Battlefield');self.assertEqual((actual['owner'],actual['controller']),(p,p));self.assertNotEqual(actual['reference'],selected)
       self.assertEqual(actual['tapped'],selection in TOWNS and not untapped_last or selection=='Breeding Pool'and not untapped_last and not pay)
      if selection in TOWNS or selection=='Breeding Pool':self.assertEqual(len(replacements),1)
      self.assertEqual(payment_seen,selection=='Breeding Pool')
     stage='finish'
    hand_choice=o.get('effect_choice')
    if hand_choice and stage!='finish':
     self.assertEqual(actor,p);self.assertIsNone(g.observe(q)['effect_choice']);self.assertTrue(hand_choice['optional'])
     for ref in hand_choice['candidates']:
      actual=objects[ref['card']];self.assertEqual((actual['zone'],actual['owner']),('Hand',p));self.assertTrue(actual['kinds']&2)
    def castable(n):return any(a['kind']=='AnnounceCast'and objects[a['card']]['card']==n for a in o['actions'])
    need_land=selection is not None and not minstrel
    have_selection=not need_land or any(v['card']==selection for v in hand(p))
    ready=quiet and have_selection and(len(lands)>=8 and len(towns)>=5 and bool(bears)if minstrel else len(lands)>=3)
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceAdventure','AnnounceOmen','AnnounceExile','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and n==name and ready:return 6000
      if actor==p and stage=='setup'and minstrel and n=='Bear Cub'and not bears:return 5000
      if actor==p and stage=='setup'and n=='Quick Study'and quiet and len(lands)>=5:return 2000
      if stage=='finish'and actor==p and n in ['Bear Cub','Bygone Colossus','Attercop']:return 2500
      return -1000
     if k=='AnnounceActivate':return 6000 if actor==p and stage=='boost'and a['card']==source else -1000
     if k=='PutHandLand':return 6000 if stage!='finish'and objects[a['object']['card']]['card']==selection else -1000
     if k=='DeclineHandEntry':return 6000 if selection is None or stage=='finish'else -1000
     if k=='ChooseEntryReplacement':
      v=o['entry_replacement'];effect=next(e for e in v['effects']if e['source']==a['source']and e['intrinsic']==a['intrinsic'])
      return 6000 if (effect['kind']=='Untapped')!=untapped_last else 1000
     if k=='ChooseEntryLife':return 6000 if a['pay']==(pay if stage!='setup'and stage!='finish'else False)else -1000
     if k=='PlayLand':
      if actor==p and stage!='finish':
       if need_land and n==selection and sum(v['card']==n for v in hand(p))<=1:return -1000
       if minstrel:return 1800+(200 if n in TOWNS else 0)if len(lands)<10 else -1000
       return 1800 if len(lands)<7 else -1000
      return 1800
     if k in ['AnnounceMana','TapMana','ActivateMana','ChooseMana']:
      if o['decision']not in ['pay_costs','resolution_mana','mana']:return -1000
      production=a.get('production',a.get('mana',[0]*6));cost=o.get('announcing_cost')or{};required=cost.get('colored',[0]*6);pool=o['players'][int(actor[-1])]['mana']
      return 3500+100*sum(min(v,max(0,need-have))for v,need,have in zip(production,required,pool))
     if k in ['PayCosts','PutTrigger','FinishTargets','Keep','ChooseSpellCost','ChooseActivationCost','KeepLegend']:return 5000
     if k=='Attack':return 3500 if actor==p and stage=='finish'else -1000
     if k in ['FinishAttackers','FinishBlockers']:return 2600
     if k in ['Discard','Bottom']:
      if actor==p and stage!='finish':
       if n==name and sum(v['card']==n for v in hand(p))<=1:return -1000
       if need_land and n==selection and sum(v['card']==n for v in hand(p))<=1:return -1000
       if minstrel and n=='Bear Cub'and not bears and sum(v['card']==n for v in hand(p))<=1:return -1000
       if minstrel and n in TOWNS and len(towns)<5:return -1000
      return 1500
     if k=='ChooseTarget':return 4500 if a['target']=={'Player':q}else -1000
     if k=='Pass':return 0
     return 1000
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision']))
    if a['kind']=='AnnounceCast'and actor==p and objects[a['card']]['card']==name and stage=='setup':
     source=a['card'];baseline=dict(drawn=o['players'][pi]['cards_drawn_this_turn'],life=o['players'][pi]['life']);stage='wait_source'
    if a['kind']=='AnnounceActivate'and actor==p and a['card']==source:
     baseline=dict(bear=bears[0]['id'],bear_power=bears[0]['power'],element=elements[0]['id'],element_power=elements[0]['power']);stage='wait_effect'
    if a['kind']=='PayCosts'and(o['announcing']or{}).get('card')==source:
     self.assertEqual(sum(a['payment']),8 if minstrel and stage=='wait_effect'else 2 if minstrel else 3)
     if minstrel and stage=='wait_effect':paid_boost=True
     else:paid_spell=True
    if a['kind']=='PutHandLand':selected=a['object']
    if a['kind']=='ChooseEntryReplacement':replacements.append(a)
    if a['kind']=='ChooseEntryLife'and selected:payment_seen=True
    if a['kind']=='Attack'and actor==p:attacked=True
    self.frame(g,o,j);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
