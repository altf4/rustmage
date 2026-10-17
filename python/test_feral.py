"""Actual paid whole-pair casts with one-graveyard masks through the public C API."""
import unittest
from rustmage import Game,EngineError
from test_adventure import AdventureTests
NAME='Feral Deathgorger // Dusk Sight'
class FeralTests(unittest.TestCase):
 replay=AdventureTests.replay
 denied=AdventureTests.denied
 def test_paid_normal_zero_one_two_from_either_graveyard_both_seats(self):
  for p in ['P0','P1']:
   for foreign in [False,True]:
    for count in range(3):
     with self.subTest(seat=p,foreign=foreign,count=count):self.scenario(p,False,foreign,count)
 def test_paid_dusk_sight_optional_target_draw_shuffle_both_seats(self):
  for p in ['P0','P1']:
   for count in [0,1]:
    with self.subTest(seat=p,count=count):self.scenario(p,True,False,count)
 def test_paid_complete_sunshield_and_tracker_both_seats(self):
  for p in ['P0','P1']:
   for name in ['Arashin Sunshield','Griffnaut Tracker']:
    with self.subTest(seat=p,name=name):self.scenario(p,False,True,2,name)
 def scenario(self,p,inset,foreign,count,name=NAME):
  q='P1'if p=='P0'else'P0';pi=int(p[-1]);owner=q if foreign else p
  own=['Swamp']*36+['Mountain']*24+[name]*20+['Bear Cub']*16+['Forest']*24+['Shock']*20
  foe=['Forest']*48+['Mountain']*24+['Bear Cub']*20+['Shock']*20
  if name!=NAME:own=['Plains'if c=='Swamp'else c for c in own]
  with Game([own,foe]if p=='P0'else[foe,own],seed=2660+count+int(foreign)*10+int(inset)*20,starting_player=p,deck_rules='Freeform')as g:
   source=None;base=None;selected=[];saw_paid=False;saw_face=False;activation_started=False;activated=False
   for iteration in range(10000):
    raw=g.observe('P0');self.assertIsNone(raw['outcome'],(p,inset,foreign,count,iteration));o=g.observe(raw['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']}
    graves={v:[x for x in objects.values()if x['owner']==v and x['zone']=='Graveyard']for v in [p,q]}
    bears={v:next((x for x in objects.values()if x['owner']==v and x['zone']=='Battlefield'and x['card']=='Bear Cub'),None)for v in [p,q]}
    quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    if source is not None:
     for d in o['spell_characteristics']:
      if d['object']['card']==source:
       self.assertEqual(d['face'],'Omen'if inset else'Normal');self.assertEqual(d['mana_cost'],'{1}{B}'if inset else('{5}{B}'if name==NAME else'{3}{W}'));saw_face=True
     for spell in o['stack']:
      if spell['card']==source and spell['kind']=='Spell':
       self.assertGreater(sum(spell['cast']['payment']),0);saw_paid=True
     if quiet and (name!='Arashin Sunshield'or activated):
      self.assertTrue(saw_paid);self.assertTrue(saw_face);self.assertFalse(o['adventures']);self.assertFalse(o['exile_permissions'])
      if inset:
       self.assertNotIn(source,objects);self.assertEqual(o['players'][pi]['library_count'],base['library'])
       self.assertEqual(o['players'][pi]['hand_count'],base['hand']);self.assertEqual(objects[base['ally']]['counters']['plus_one'],count)
      else:
       body=objects[source];self.assertEqual(body['zone'],'Battlefield');self.assertEqual((body['power'],body['toughness']),(3,5 if name==NAME else 4 if name=='Arashin Sunshield'else 2));self.assertEqual(body['keywords']&(129 if name==NAME else 1),129 if name==NAME else 1 if name=='Griffnaut Tracker'else 0)
       if name=='Arashin Sunshield':self.assertTrue(body['tapped']);self.assertTrue(objects[base['enemy']]['tapped'])
       self.assertEqual(len(selected),count)
       for card in base['graves']:
        self.assertEqual(objects[card]['zone'],'Exile'if card in selected else'Graveyard')
      return
    ready=all(len(graves[v])>=2 for v in [p,q])and bears[p]and (name==NAME or bears[q])and sum(v['owner']==p and v['zone']=='Battlefield'and v['card']in ['Swamp','Mountain','Forest','Plains']and not v['tapped']for v in objects.values())>=8
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Attack','AnnounceAdventure','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -5000
     if k=='AnnounceOmen':return 3000 if source is None and inset and actor==p and n==name and ready else -1000
     if k=='AnnounceCast':
      if n==name:return 3000 if source is None and not inset and actor==p and ready else -1000
      if n=='Bear Cub':return 2200 if source is None and not bears[actor] else -1000
      if n=='Shock':return 2300 if source is None and len(graves[actor])<2 and bears[actor] else -1000
      return -1000
     if k in ['AnnounceActivate','Activate']:return 3000 if source is not None and name=='Arashin Sunshield'and actor==p and a['card']==source and not activation_started else -1000
     if k=='ChooseTarget':
      if 'Card'not in a['target']:return -1000
      target=objects[a['target']['Card']];ann=o['announcing']
      if ann and ann['source_card']=='Shock':return 3000 if bears[actor]and target['id']==bears[actor]['id']else -1000
      if source is not None:
       if activation_started:return 3000 if target['id']==base['enemy']else -1000
       if inset:return 3000 if count and target['id']==base['ally']else -1000
       return 3000 if len(selected)<count and target['owner']==owner and target['zone']=='Graveyard'else -1000
      return -1000
     if k=='FinishTargets':return 2400
     if k=='PlayLand':return 2000-30*sum(v['owner']==actor and v['zone']=='Battlefield'and v['card']==n for v in objects.values())
     if k=='TapMana':return 2500 if o['decision']=='pay_costs'else -1000
     if k in ['PayCosts','PutTrigger','Keep','ChooseSpellCost','FinishAttackers','FinishBlockers']:return 2800
     if k in ['Discard','Bottom']:return 2000 if n in ['Forest','Mountain','Swamp','Plains']else 500
     if k=='Pass':return 0
     return -1000
    scores=[score(a)for a in o['actions']];j=max(range(len(scores)),key=scores.__getitem__);a=o['actions'][j]
    if a['kind']in ['AnnounceCast','AnnounceOmen']and objects[a['card']]['card']==name:
     source=a['card'];base={'library':o['players'][pi]['library_count'],'hand':o['players'][pi]['hand_count'],'ally':bears[p]['id'],'enemy':bears[q]['id']if bears[q]else None,'graves':[x['id']for v in [p,q]for x in graves[v]]}
    if a['kind']in ['AnnounceActivate','Activate']and a['card']==source:activation_started=True
    if a['kind']=='PayCosts'and activation_started and o['announcing']and o['announcing']['card']==source:activated=True
    if source is not None and not activation_started and a['kind']=='ChooseTarget'and o['announcing']and o['announcing']['card']==source and not inset:
     selected.append(a['target']['Card'])
    before=[g.observe(v)for v in [p,q]]
    with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor='P1'if actor=='P0'else'P0'),a)
    self.assertEqual(before,[g.observe(v)for v in [p,q]])
    self.replay(g,o,j)
    if source is not None and not inset and len(selected)==1:
     now=g.observe(g.observe('P0')['actor'])
     if now['decision']=='targets':
      forbidden=next(x for x in graves['P1'if owner=='P0'else'P0'])['id']
      self.denied(g,now,{'kind':'ChooseTarget','target':{'Card':forbidden}})
   self.fail(('unreached',p,inset,foreign,count))
if __name__=='__main__':unittest.main()
