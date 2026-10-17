"""Paid complete Role follow-ups; legal Standard self-play and real foreign casting."""
import json,re,unittest
from pathlib import Path
from rustmage import OBSERVATION_SCHEMA, Game
import test_role_consumers as prior
NAMES=['Become Brutes','Cursed Courtier','Cut In','Diminisher Witch',"Eriette's Whisper",'Merry Bards','Protective Parents','Shatter the Oath','Spellbook Vendor',"The Witch's Vanity",'Unassuming Sage',"Witch's Mark"]
ROLES=['Monster','Cursed','Young Hero','Cursed','Wicked','Young Hero','Young Hero','Wicked','Sorcerer','Wicked','Sorcerer','Wicked']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
SAGA='Korvold and the Noble Thief'
ROOT=Path(__file__).resolve().parents[1]
FACTS={v['name']:v for v in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
SCHEMA=OBSERVATION_SCHEMA
class RoleFollowupTests(unittest.TestCase):
 frame=prior.RoleConsumerTests.frame
 def test_all_twelve_paid_complete_bodies_both_seats(self):
  for i,name in enumerate(NAMES):
   for p in ['P0','P1']:
    with self.subTest(card=name,seat=p):self.scenario(i,p,29400+i)
 def test_all_twelve_real_foreign_paid_bodies_and_private_choices_both_seats(self):
  for i,name in enumerate(NAMES):
   for p in ['P0','P1']:
    with self.subTest(card=name,seat=p):self.scenario(i,p,29480+i,foreign=True)
 def test_witch_mark_decline_keeps_independent_role_both_seats(self):
  for p in ['P0','P1']:self.scenario(11,p,29420,decline=True)
 def scenario(self,i,p,seed,library=None,foreign=False,decline=False):
  name=NAMES[i];q='P1' if p=='P0' else 'P0';pi=int(p[-1]);qi=1-pi
  if foreign:
   own=['Plains']*10+['Island']*10+['Swamp']*10+['Mountain']*10+['Forest']*4+sum(([n]*4 for n in [SAGA,'Bear Cub','Candy Trail','Murder']),[])
   enemy=['Forest']*12+['Bear Cub']*12+[name]*36
   decks=[own,enemy] if p=='P0' else [enemy,own]
  else:
   deck=['Plains']*6+['Island']*6+['Swamp']*6+['Mountain']*6+['Forest']*8+sum(([n]*4 for n in [name,'Bear Cub','Candy Trail','Quick Study','Bygone Colossus','Murder' if i==6 else 'Chrome Companion','Thrill of Possibility']),[])
   decks=[deck,deck]
  self.assertEqual([len(d) for d in decks],[60,60])
  with Game(decks,seed=seed,starting_player=p,deck_rules='Freeform' if foreign else 'Standard',library=library) as g:
   stage='setup';source=None;target=None;primary=None;frames=0;role_seen=False;attacked=False;paid=False;optional_paid=False;scry_seen=False;young_seen=False;lifelink_seen=False;food=None;food_paid=False;food_life=None;food_done=False;chapters=set();theft_chapters={};paid_sagas=set();base=None;bargain_card=None
   for iteration in range(35000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(SCHEMA,FP))
    if head['outcome'] is not None:
     self.assertFalse(foreign,(name,p,stage,'foreign witness ended early'))
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertTrue(role_seen);self.assertTrue(paid);self.assertTrue(attacked)
     if ROLES[i]=='Young Hero':self.assertTrue(young_seen,(name,p,'missing granted attack counter'))
     if i in [8,10]:self.assertTrue(scry_seen,(name,p,'missing granted private scry'))
     if i==1:self.assertTrue(lifelink_seen)
     return frames
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p);own=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p];enemy=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==q];ally=next((v for v in own if v['card']=='Bear Cub'),None);foe=next((v for v in enemy if v['card']=='Bear Cub'),None);body=objects.get(source);hand=[v for v in mine['objects'] if v['zone']=='Hand' and v['owner']==p];quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    for v in objects.values():
     if v['zone']=='Battlefield' and v['card']==SAGA and v['id'] in paid_sagas:theft_chapters.setdefault(v['id'],set()).add(v['counters']['lore'])
     if v['id']==source and i==9 and v['zone']=='Battlefield':chapters.add(v['counters']['lore'])
    if i==9 and stage!='setup' and food is None:
     f=next((v for v in own if v.get('token') and v['token']['subtypes']=='Food'),None)
     if f is not None:food=f['id']
    if food_paid and quiet and not food_done:
     self.assertEqual(o['players'][pi]['life'],food_life+3);self.assertNotIn(food,objects);food_done=True
    if stage=='wait_body' and quiet and body is not None and body['zone']=='Battlefield':stage='kill_parent'
    if stage in ['wait_role','wait_parent'] and quiet:
     recipient=objects.get(target)
     aura=next((v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p and v.get('token') and v['token']['name']==ROLES[i] and recipient is not None and v['attached_to']==recipient['reference']),None)
     if aura is not None:
      self.assertEqual(aura['owner'],p);self.assertTrue(paid);self.assertTrue(any(v['aura']==aura['reference'] and v['recipient']==recipient['reference'] for v in o['role_timestamps']))
      self.assertEqual(recipient['power'],1 if ROLES[i]=='Cursed' else 2 if ROLES[i]=='Young Hero' else int(FACTS[recipient['card']]['power'])+1)
      self.assertEqual(body['owner'],q if foreign else p)
      if body['zone']=='Battlefield':self.assertEqual(body['controller'],p)
      if i in [2,7,9]:self.assertEqual(objects[primary]['zone'],'Graveyard')
      if i==0:
       second=next(v for v in objects.values() if v['zone']=='Battlefield' and v.get('token') and v['token']['name']=='Monster' and v['attached_to']==objects[primary]['reference']);self.assertEqual((second['owner'],second['controller']),(p,p));self.assertTrue(recipient['keywords']&4);self.assertTrue(objects[primary]['keywords']&4)
      if i==1:self.assertTrue(recipient['keywords']&64)
      if i==3:self.assertEqual(objects[bargain_card]['zone'],'Graveyard')
      if i==4:self.assertEqual(o['players'][qi]['hand_count'],max(0,base['opponent_hand']-2))
      if i==6:self.assertEqual(body['zone'],'Graveyard')
      if i==8:self.assertTrue(body['keywords']&8);self.assertTrue(optional_paid)
      if i==9:self.assertTrue({1,2,3}<=chapters);self.assertTrue(food_done);self.assertEqual(body['zone'],'Graveyard')
      if i in [5,10]:self.assertTrue(optional_paid)
      if i==11:
       self.assertEqual(o['players'][pi]['cards_drawn_this_turn']-base['drawn'],0 if decline else 2);self.assertEqual(o['players'][pi]['cards_discarded_this_turn']-base['discarded'],0 if decline else 1)
      role_seen=True;stage='finish'
    if stage=='finish':
     if ROLES[i]=='Young Hero' and objects.get(target,{}).get('counters',{}).get('plus_one',0)>0:young_seen=True
     if i==1 and o['players'][pi]['life_gained_this_turn']>0:lifelink_seen=True
     complete=attacked and (ROLES[i]!='Young Hero' or young_seen) and (i not in [8,10] or scry_seen) and (i!=1 or lifelink_seen)
     if foreign and complete:
      self.assertTrue(any({1,2,3}<=v for v in theft_chapters.values()));return frames
    if o['library_choice'] is not None:
     self.assertIsNone(g.observe('P1' if actor=='P0' else 'P0')['library_choice'])
     if role_seen and attacked and i in [8,10]:self.assertEqual(actor,p);scry_seen=True
    if o['decision']=='targets' and o['announcing'] and o['announcing']['card']==source:
     if i in [1,10]:self.fail((name,'inline source effect incorrectly targeted'))
     if i in [5,8]:self.assertTrue(optional_paid,(name,'reflexive target selected before payment'))
    if i==4 and o['decision']=='discard' and o['resolving'] and o['resolving']['source_card']==name:
     self.assertEqual(actor,q);self.assertEqual(g.observe(p)['actions'],[]);self.assertFalse(any(v['owner']==q and v['zone']=='Hand' for v in g.observe(p)['objects']))
    def available(player):
     mana=list(o['players'][int(player[-1])]['mana'])
     for v in objects.values():
      if v['zone']=='Battlefield' and v['controller']==player and not v['tapped'] and v['card'] in BASICS:mana[BASICS.index(v['card'])]+=1
     return mana
    def can_pay(n,player):
     mana=available(player);tokens=re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[n].get('mana_cost',''));colored=[tokens.count(c) for c in 'WUBRGC'];generic=sum(int(c) for c in tokens if c.isdecimal());return all(a>=b for a,b in zip(mana,colored)) and sum(mana)>=sum(colored)+generic
    ready=bool(quiet and ally and foe and sum(available(p))>=8 and can_pay(name,p) and (i!=3 or any(v['card']=='Candy Trail' for v in own)))
    eligible={r['object']['card'] for r in mine['exile_permissions'] if r['player']==p and r['expires_turn']>=o['turn']}
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceAdventure','AnnounceOmen','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='AnnounceExile':return 3500 if foreign and actor==p and stage=='setup' and ready and a['card'] in eligible and n==name else -1000
     if k=='AnnounceCast':
      if actor==p and stage=='setup' and not foreign and n==name and ready:return 3400
      if actor==p and foreign and stage=='setup' and n==SAGA and ready and can_pay(n,p) and not any(v['card']==SAGA for v in own):return 3300
      if actor==p and stage=='kill_parent' and n=='Murder' and can_pay(n,p):return 3400
      if actor==p and i==3 and stage=='setup' and n=='Candy Trail' and not any(v['card']==n for v in own) and can_pay(n,p):return 2600
      if n=='Bear Cub' and can_pay(n,actor) and not (ally if actor==p else foe):return 2500
      if stage=='finish' and n in ['Bear Cub','Bygone Colossus','Chrome Companion'] and can_pay(n,actor):return 2400
      return -1000
     if k=='AnnounceActivate':return 3300 if actor==p and i==9 and a['card']==food and not food_paid and sum(available(p))>=2 else -1000
     if k=='ChooseTarget':
      if o['announcing'] and o['announcing']['source_card']==SAGA:return 3200 if a['target']=={'Player':q} else -1000
      if o['announcing'] and o['announcing']['source_card']=='Murder':return 3200 if a['target']=={'Card':source} else -1000
      wanted=[{'Card':target}]
      if i in [0,2,3,7,9]:wanted.append({'Card':primary})
      if i==4:wanted.append({'Player':q})
      if i in [0,2,3,7,9] and a['target']=={'Card':primary}:return 3200
      if i==4 and a['target']=={'Player':q}:return 3200
      return 3100 if a['target'] in wanted else -1000
     if k=='ChooseBargain':return 3200 if a['sacrifice'] is not None and objects[a['sacrifice']['card']]['card']=='Candy Trail' else -1000
     if k=='DeclineResolutionMana':return 1000
     if k=='PayResolutionMana':return 3200
     if k=='DeclineEffectPayment':return 3200 if i==11 and decline else -1000
     if k=='PlayLand':return 2200-20*sum(v['zone']=='Battlefield' and v['controller']==actor and v['card']==n for v in objects.values())
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2400 if o['decision'] in ['pay_costs','resolution_mana'] and n in BASICS else -1000
     if k in ['PayCosts','PutTrigger','FinishTargets','FinishSearch','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost']:return 3000
     if k in ['Discard','Bottom']:
      if actor==p and stage=='setup' and n==name and sum(v['card']==n for v in hand)<=1:return -1000
      return 2200 if n in ['Candy Trail','Thrill of Possibility','Quick Study','Bygone Colossus','Chrome Companion'] else 1000
     if k=='Attack':return 3600 if stage=='finish' else -1000
     if k=='Block':return -1000
     if k=='Pass':return 0
     return 1000
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,foreign,seed,stage,o['decision'],a))
    if a['kind'] in ['AnnounceCast','AnnounceExile'] and actor==p and stage=='setup' and objects[a['card']]['card']==name:
     source=a['card'];primary=foe['id'];target=source if i in [1,10] else primary if i==3 else ally['id'];stage='wait_body' if i==6 else 'wait_role';base=dict(opponent_hand=o['players'][qi]['hand_count'],drawn=o['players'][pi]['cards_drawn_this_turn'],discarded=o['players'][pi]['cards_discarded_this_turn'])
    if a['kind']=='AnnounceCast' and actor==p and objects[a['card']]['card']==SAGA:paid_sagas.add(a['card'])
    if a['kind']=='AnnounceCast' and actor==p and stage=='kill_parent' and objects[a['card']]['card']=='Murder':stage='wait_parent'
    if a['kind']=='ChooseBargain' and a['sacrifice'] is not None:bargain_card=a['sacrifice']['card']
    if a['kind']=='PayCosts' and o['announcing']:
     packet=o['announcing']
     if packet['card']==source and packet['kind']=='Spell':
      tokens=re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[name]['mana_cost']);self.assertEqual(sum(a['payment']),sum(int(v) if v.isdecimal() else 1 for v in tokens));paid=True
      if i==3:self.assertTrue(packet['cast']['bargain'])
     if packet['card']==food:self.assertEqual(sum(a['payment']),2);food_paid=True;food_life=o['players'][pi]['life']
    if a['kind']=='PayResolutionMana' and o['resolving'] and o['resolving']['card']==source:self.assertEqual(sum(a['payment']),2 if i==10 else 1);optional_paid=True
    attack_id=source if i in [1,3,10] else target
    if a['kind']=='Attack' and actor==p and a['card']==attack_id:attacked=True
    self.frame(g,o,index);frames+=1
    if a['kind']=='Attack' and actor==p and i==8 and a['card']==source:self.assertFalse(next(v for v in g.observe(p)['objects'] if v['id']==source)['tapped'])
   self.fail((name,p,foreign,seed,stage,'no complete paid terminal/witness',source,target))
if __name__=='__main__':unittest.main()
