"""Paid whole hand-land bodies in naturally drawn Standard games, with owned replay."""
import json,os,re,unittest
from pathlib import Path
from rustmage import Game
import test_role_consumers as prior
ROOT=Path(__file__).resolve().parents[1]
FACTS={c['name']:c for c in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
ROWS=[('Embrace the Paradox',None,False),('Embrace the Paradox',None,True),('Lessons from Life',None,False),('Lessons from Life',None,True),('H.E.R.B.I.E. Scout Unit',None,False),('H.E.R.B.I.E. Scout Unit',None,True),('Insidious Fungus',0,False),('Insidious Fungus',1,False),('Insidious Fungus',2,False),('Insidious Fungus',2,True),('Nicanzil, Current Conductor',None,False),('Nicanzil, Current Conductor',None,True)]
class HandLandTests(unittest.TestCase):
 frame=prior.RoleConsumerTests.frame
 def test_five_complete_paid_bodies_modes_and_declines_both_seats(self):
  for i,(name,mode,decline) in enumerate(ROWS):
   for p in ['P0','P1']:
    with self.subTest(name=name,mode=mode,decline=decline,seat=p):self.scenario(i,p,32500+i)
 def scenario(self,i,p,seed,library=None):
  name,mode,decline=ROWS[i];q='P1'if p=='P0'else'P0';pi=int(p[-1]);qi=1-pi
  deck=['Forest']*12+['Island']*12+['Mountain']*8+sum(([n]*4 for n in [name,'Attercop','Bear Cub','Bygone Colossus','Quick Study','Sentinel of the Nameless City','Candy Trail']),[])
  enemy=['Forest']*16+['Island']*16+sum(([n]*4 for n in ['Candy Trail','Doubling Season','Bear Cub','Bygone Colossus','Llanowar Elves','Quick Study','Chrome Companion']),[])
  self.assertEqual((len(deck),len(enemy)),(60,60))
  with Game([deck,enemy]if p=='P0'else[enemy,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   stage='setup';source=None;victim=None;base=None;paid=False;activation_paid=False;frames=0;choice_seen=False;selected=None;selected_checked=False;declined=[];seen=set();map_context=None;mapping=None;attacked=False
   for step in range(35000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(104,FP))
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,mode,decline,p,seed,stage,step));self.assertTrue(paid);self.assertTrue(attacked)
     return dict(name=name,mode=mode,decline=decline,seat=p,seed=seed,frames=frames,explore=sorted(seen))
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p);quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    def own(player):return[v for v in objects.values()if v['zone']=='Battlefield'and v['controller']==player]
    def hand(player):return[v for v in g.observe(player)['objects']if v['zone']=='Hand'and v['owner']==player]
    def budget(player):
     mana=list(o['players'][int(player[-1])]['mana'])
     for v in own(player):
      if v['card']in BASICS and not v['tapped']:mana[BASICS.index(v['card'])]+=1
     return mana
    def can_pay(n,player):
     tokens=re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[n].get('mana_cost',''));colors=[tokens.count(c)for c in'WUBRGC'];mana=budget(player);return all(a>=b for a,b in zip(mana,colors))and sum(mana)>=sum(colors)+sum(int(c)for c in tokens if c.isdecimal())
    body=objects.get(source);lands_hand=[v for v in hand(p)if v['card']in BASICS];maps=[v for v in own(p)if v.get('token')and'Map'in v['token']['subtypes'].split()]
    target=next((v for v in own(q)if v['card']==('Candy Trail'if mode==0 else'Doubling Season')),None)
    ready=quiet and sum(budget(p))>=8 and can_pay(name,p)and bool(lands_hand)and(name!='Insidious Fungus'or mode==2 or target is not None)
    if quiet and stage=='wait_body':
     self.assertEqual(body['zone'],'Battlefield');self.assertTrue(paid)
     if name=='Insidious Fungus':stage='activate'
     else:stage='explore'
    if quiet and stage=='wait_effect':
     self.assertTrue(paid)
     if name=='Insidious Fungus':
      self.assertTrue(activation_paid);self.assertEqual(body['zone'],'Graveyard')
      if mode<2:self.assertEqual(objects[victim]['zone'],'Graveyard');self.assertFalse(choice_seen)
     elif name=='H.E.R.B.I.E. Scout Unit':self.assertEqual(body['zone'],'Battlefield');self.assertTrue(body['keywords']&1)
     else:self.assertEqual(body['zone'],'Graveyard')
     expected=0 if name=='Insidious Fungus'and mode<2 else 1 if name in['Insidious Fungus','H.E.R.B.I.E. Scout Unit']else 3
     self.assertEqual(o['players'][pi]['cards_drawn_this_turn']-base['drawn'],expected)
     if expected:
      self.assertTrue(choice_seen)
      if decline:
       self.assertIsNone(selected);self.assertTrue(declined)
       for cid in declined:self.assertEqual(objects[cid]['zone'],'Hand')
      else:self.assertIsNotNone(selected)
     stage='finish'
    if selected is not None and quiet and not selected_checked:
     land=objects[selected['card']];self.assertEqual(land['zone'],'Battlefield');self.assertTrue(land['tapped']);self.assertEqual((land['owner'],land['controller']),(p,p));self.assertNotEqual(land['reference'],selected);selected_checked=True
    if quiet and stage=='explore'and map_context is not None:
     event=o['last_explore']
     if event and event['sequence']>map_context['sequence']:
      self.assertEqual(event['explorer']['object'],body['reference']);self.assertIsNotNone(event['revealed'])
      is_land='Land'in FACTS[event['revealed']['card']]['type_line'].split(' — ')[0].split();kind='land'if is_land else'nonland';seen.add(kind)
      self.assertEqual(body['counters']['plus_one']-map_context['counters'],0 if is_land else 2)
      map_context=None;mapping=None
      if seen=={'land','nonland'}:self.assertTrue(choice_seen);stage='finish'
    effect=o.get('effect_choice')
    if effect and isinstance(effect['kind'],dict)and'HandLand'in effect['kind']:
     self.assertEqual(actor,p);self.assertIsNone(g.observe(q)['effect_choice']);self.assertTrue(effect['optional']);self.assertTrue(effect['candidates']);choice_seen=True
     if name!='Nicanzil, Current Conductor':
      self.assertEqual(o['players'][pi]['cards_drawn_this_turn']-base['drawn'],1 if name in['Insidious Fungus','H.E.R.B.I.E. Scout Unit']else 3)
     for ref in effect['candidates']:
      v=objects[ref['card']];self.assertEqual(v['zone'],'Hand');self.assertEqual(v['owner'],p);self.assertIn(v['card'],BASICS);self.assertEqual(v['reference'],ref)
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceAdventure','AnnounceOmen','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceExile']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and ready and n==name:return 5000
      if actor==q and stage=='setup'and name=='Insidious Fungus'and mode<2 and n==('Candy Trail'if mode==0 else'Doubling Season')and target is None:return 3300
      if actor==p and stage=='explore'and n=='Sentinel of the Nameless City'and not maps and can_pay(n,p):return 3500
      if actor==p and stage=='finish'and n in['Bear Cub','Bygone Colossus','Attercop','Sentinel of the Nameless City']:return 2800
      if actor==p and stage in['setup','explore']and not(name=='Nicanzil, Current Conductor'and stage=='explore')and n=='Quick Study'and quiet and sum(budget(p))>=5 and can_pay(n,p):return 2200
      if actor==p and stage=='explore'and n=='Candy Trail'and quiet and can_pay(n,p):return 5500 if maps else 2100
      return -1000
     if k=='AnnounceActivate':
      if actor==p and stage=='activate'and a['card']==source and a['ability']==0:return 5000
      if actor==p and stage=='explore'and map_context is None and any(v['id']==a['card']for v in maps)and sum(budget(p))>=1:return 5000
      return -1000
     if k=='ArrangeLibrary'and actor==p and stage=='explore':
      known=next(v['card']for v in o['library_choice']['remaining']if v['id']==a['card']);land='Land'in FACTS[known]['type_line'].split(' — ')[0].split();wanted=land if'land'not in seen else not land
      return 4500 if(a['destination']=='Top')==wanted else -1000
     if k=='ChooseMode':return 4000 if a['mode']==mode else -1000
     if k=='ChooseTarget':return 4000 if a['target'].get('Card')==(source if stage=='explore'else victim)else -1000
     if k=='PutHandLand':return 4000 if not decline else -1000
     if k=='DeclineHandEntry':return 4500 if decline else -1000
     if k=='ChooseExplore':return 4000 if a['graveyard']else -1000
     if k=='PlayLand':return 1900-15*sum(v['card']==n for v in own(actor))if actor!=p or stage=='finish'or sum(v['card']in BASICS for v in own(p))<8 else -1000
     if k in['AnnounceMana','TapMana','ActivateMana']:return 3500 if o['decision']in['pay_costs','resolution_mana']else -1000
     if k in['PayCosts','PutTrigger','FinishTargets','Keep','ChooseSpellCost']:return 4200
     if k=='Attack':return 3500 if actor==p and(stage=='finish'or stage=='explore'and not maps and objects[a['card']]['card']=='Sentinel of the Nameless City'and o['players'][qi]['life']>6)else -1000
     if k in['FinishAttackers','FinishBlockers']:return 2600
     if k in['Discard','Bottom']:
      if actor==p and stage!='finish'and(n==name or n=='Sentinel of the Nameless City'and name=='Nicanzil, Current Conductor')and sum(v['card']==n for v in hand(p))<=1:return -1000
      return 1500
     if k=='Pass':return 0
     return 1000
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,mode,decline,p,seed,stage,o['decision']))
    if a['kind']=='AnnounceCast'and actor==p and stage=='setup'and objects[a['card']]['card']==name:
     source=a['card'];victim=target['id']if name=='Insidious Fungus'and mode<2 else None;base={'drawn':o['players'][pi]['cards_drawn_this_turn']};stage='wait_body'if name in['Insidious Fungus','Nicanzil, Current Conductor']else'wait_effect'
    elif a['kind']=='AnnounceActivate'and actor==p and stage=='activate':base={'drawn':o['players'][pi]['cards_drawn_this_turn']};stage='wait_effect'
    elif a['kind']=='AnnounceActivate'and actor==p and stage=='explore':mapping=a['card'];map_context={'sequence':(o['last_explore']or{}).get('sequence',0),'counters':body['counters']['plus_one']}
    if a['kind']=='PayCosts'and(o['announcing']or{}).get('card')==source:
     expected=2 if(o['announcing']or{}).get('kind')!='Spell'else sum(int(x)if x.isdecimal()else 1 for x in re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[name]['mana_cost']))
     self.assertEqual(sum(a['payment']),expected)
     if name=='Insidious Fungus'and paid:activation_paid=True
     else:paid=True
    if a['kind']=='PayCosts'and mapping is not None and(o['announcing']or{}).get('card')==mapping:self.assertEqual(sum(a['payment']),1)
    if a['kind']=='PutHandLand':selected=a['object'];selected_checked=False
    if a['kind']=='DeclineHandEntry'and effect:declined=[r['card']for r in effect['candidates']]
    if a['kind']=='Attack'and actor==p and stage=='finish':attacked=True
    self.frame(g,o,index);frames+=1
   self.fail((name,mode,decline,p,seed,stage,'no natural terminal game'))
if __name__=='__main__':unittest.main()
