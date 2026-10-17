"""Real paid complete Werefox games: late reflexive targets and owned replay."""
import json,re,unittest
from pathlib import Path
from rustmage import Game
import test_role_consumers as prior
ROOT=Path(__file__).resolve().parents[1]
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
FACTS={v['name']:v for v in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
SCHEMA=94
NAME='Curse of the Werefox'
SAGA='Korvold and the Noble Thief'
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class RoleSuccessFightTests(unittest.TestCase):
 frame=prior.RoleConsumerTests.frame
 def test_paid_complete_standard_body_optional_fight_both_seats(self):
  for p in ['P0','P1']:
   for decline in [False,True]:
    with self.subTest(seat=p,decline=decline):self.scenario(p,30400,decline=decline)
 def test_real_foreign_cast_role_creator_source_lki_and_reflexive_controller_both_seats(self):
  for p in ['P0','P1']:
   for decline in [False,True]:
    with self.subTest(seat=p,decline=decline):self.scenario(p,30480,foreign=True,decline=decline)
 def scenario(self,p,seed,library=None,foreign=False,decline=False):
  q='P1'if p=='P0'else'P0';pi=int(p[-1]);qi=1-pi
  if foreign:
   own=['Plains']*10+['Island']*10+['Swamp']*10+['Mountain']*10+['Forest']*4+sum(([n]*4 for n in [SAGA,'Shattered Acolyte','Bear Cub','Bygone Colossus']),[])
   enemy=['Forest']*12+['Bear Cub']*12+[NAME]*36
   decks=[own,enemy]if p=='P0'else[enemy,own]
  else:
   deck=['Forest']*16+['Plains']*10+['Island']*6+sum(([n]*4 for n in [NAME,'Shattered Acolyte','Bear Cub','Bygone Colossus','Quick Study','Chrome Companion','Llanowar Elves']),[])
   decks=[deck,deck]
  self.assertEqual([len(d)for d in decks],[60,60])
  with Game(decks,seed=seed,starting_player=p,deck_rules='Freeform'if foreign else'Standard',library=library)as g:
   stage='setup';source=recipient=foe_id=None;recipient_ref=None;spell_ref=None;paid=False;reflexive=False;role_seen=False;attacked=False;combat_life=False;frames=0;life_before=None;paid_sagas=set();chapters={}
   for iteration in range(35000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(SCHEMA,FP))
    if head['outcome']is not None:
     self.assertFalse(foreign,('foreign witness ended early',p,seed,stage));self.assertEqual(stage,'finish');self.assertTrue(paid and reflexive and role_seen and attacked and combat_life,(p,seed,paid,reflexive,role_seen,attacked,combat_life));return frames
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p);hand=[v for v in mine['objects']if v['zone']=='Hand'];own=[v for v in objects.values()if v['zone']=='Battlefield'and v['controller']==p];enemy=[v for v in objects.values()if v['zone']=='Battlefield'and v['controller']==q];ally=next((v for v in own if v['card']=='Shattered Acolyte'),None);foe=next((v for v in enemy if v['card']=='Bear Cub'),None);quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    for v in objects.values():
     if v['card']==SAGA and v['id']in paid_sagas and v['zone']=='Battlefield':chapters.setdefault(v['id'],set()).add(v['counters']['lore'])
    packet=o['announcing']
    if packet and packet['card']==source:
     if packet['kind']=='Spell':self.assertIsNone(packet['reflexive_subject']);self.assertEqual(packet['controller'],p)
     elif packet['reflexive_subject']is not None:
      reflexive=True;self.assertEqual(packet['reflexive_subject'],{'object':recipient_ref});self.assertEqual(packet['object'],spell_ref);self.assertEqual(packet['controller'],p);self.assertEqual(packet['source_card'],NAME);self.assertEqual(objects[source]['zone'],'Graveyard')
      aura=next(v for v in objects.values()if v['zone']=='Battlefield'and v.get('token')and v['token']['name']=='Monster'and v['attached_to']==recipient_ref);self.assertEqual((aura['owner'],aura['controller']),(p,p));role_seen=True
      self.assertEqual(objects[recipient]['power'],3);self.assertTrue(objects[recipient]['keywords']&256)
    if stage=='wait_body'and quiet:
     self.assertTrue(paid and reflexive and role_seen,(p,seed,'missing separate trigger'))
     self.assertEqual(objects[source]['zone'],'Graveyard');self.assertEqual(objects[source]['owner'],q if foreign else p);self.assertEqual(objects[recipient]['reference'],recipient_ref);self.assertEqual(objects[recipient]['damage'],0 if decline else 2);self.assertEqual(objects[foe_id]['zone'],'Battlefield'if decline else'Graveyard');self.assertEqual(o['players'][pi]['life'],life_before+(0 if decline else 3));stage='finish'
    if stage=='finish'and attacked and o['players'][pi]['life_gained_this_turn']>0:combat_life=True
    if foreign and stage=='finish'and attacked and combat_life and quiet:
     self.assertTrue(any({1,2,3}<=v for v in chapters.values()));return frames
    def available(player):
     mana=list(o['players'][int(player[-1])]['mana'])
     for v in objects.values():
      if v['zone']=='Battlefield'and v['controller']==player and not v['tapped']and v['card']in BASICS:mana[BASICS.index(v['card'])]+=1
     return mana
    def can_pay(name,player):
     mana=available(player);tokens=re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[name].get('mana_cost',''));colored=[tokens.count(c)for c in'WUBRGC'];generic=sum(int(c)for c in tokens if c.isdecimal());return all(a>=b for a,b in zip(mana,colored))and sum(mana)>=sum(colored)+generic
    ready=bool(quiet and ally and foe and sum(available(p))>=8 and can_pay(NAME,p))
    eligible={x['object']['card']for x in mine['exile_permissions']if x['player']==p and x['expires_turn']>=o['turn']}
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceAdventure','AnnounceOmen','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='AnnounceExile':return 3500 if foreign and actor==p and stage=='setup'and ready and a['card']in eligible and n==NAME else -1000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and not foreign and n==NAME and ready:return 3400
      if actor==p and foreign and stage=='setup'and n==SAGA and ready and can_pay(n,p)and not any(v['card']==SAGA for v in own):return 3300
      if stage=='setup'and n=='Shattered Acolyte'and actor==p and not ally and can_pay(n,p):return 2500
      if stage=='setup'and n=='Bear Cub'and actor==q and not foe and can_pay(n,q):return 2500
      if stage=='finish'and n in ['Shattered Acolyte','Bear Cub','Bygone Colossus','Chrome Companion','Llanowar Elves']and can_pay(n,actor):return 2400
      return -1000
     if k=='ChooseTarget':
      if packet and packet['source_card']==SAGA:return 3200 if a['target']=={'Player':q}else-1000
      if packet and packet['card']==source:
       if packet['kind']=='Spell':return 3200 if a['target']=={'Card':recipient}else-1000
       self.assertTrue(reflexive);self.assertNotEqual(a['target'],{'Card':recipient})
       return 3200 if not decline and a['target']=={'Card':foe_id}else-1000
      return -1000
     if k=='PlayLand':return 2200-20*sum(v['card']==n for v in (own if actor==p else enemy))
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2400 if o['decision']=='pay_costs'and n in BASICS else-1000
     if k in ['PayCosts','PutTrigger','FinishTargets','FinishSearch','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost']:return 3000
     if k in ['Discard','Bottom']:
      if actor==p and stage=='setup'and n==NAME and sum(v['card']==n for v in hand)<=1:return -1000
      return 2200 if n in ['Quick Study','Bygone Colossus','Chrome Companion','Llanowar Elves'] else 1000
     if k=='Attack':return 3600 if stage=='finish'else-1000
     if k=='Block':return -1000
     if k=='AnnounceActivate':return -1000
     if k=='Pass':return 0
     return 1000
    index=max(range(len(o['actions'])),key=lambda i:score(o['actions'][i]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(p,seed,foreign,decline,stage,o['decision'],a))
    if a['kind']in ['AnnounceCast','AnnounceExile']and actor==p and objects[a['card']]['card']==NAME:
     source=a['card'];recipient=ally['id'];recipient_ref=ally['reference'];foe_id=foe['id'];life_before=o['players'][pi]['life'];stage='wait_body'
    if a['kind']=='AnnounceCast'and actor==p and objects[a['card']]['card']==SAGA:paid_sagas.add(a['card'])
    if a['kind']=='PayCosts'and packet and packet['card']==source and packet['kind']=='Spell':self.assertEqual(sum(a['payment']),3);self.assertGreater(a['payment'][4],0);paid=True;spell_ref=packet['object']
    if a['kind']=='Attack'and actor==p and a['card']==recipient:attacked=True
    self.frame(g,o,index);frames+=1
   self.fail((p,seed,foreign,decline,stage,'no terminal/complete witness'))
if __name__=='__main__':unittest.main()
