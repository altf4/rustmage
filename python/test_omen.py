"""Real paid Omen and normal faces through an explicitly selected native library."""
import json,unittest
from pathlib import Path
from rustmage import Game,EngineError
import test_adventure
NAMES=['Dirgur Island Dragon // Skimming Strike','Disruptive Stormbrood // Petty Revenge','Marang River Regent // Coil and Catch','Purging Stormbrood // Absorb Essence','Riling Dawnbreaker // Signaling Roar','Runescale Stormbrood // Chilling Screech','Sagu Wildling // Roost Seek','Stormshriek Feral // Flush Out','Twinmaw Stormbrood // Charring Bite']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
PREFERRED=['Island','Forest','Island','Swamp','Plains','Mountain','Forest','Mountain','Plains']
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
class OmenTests(unittest.TestCase):
 replay=test_adventure.AdventureTests.replay
 denied=test_adventure.AdventureTests.denied
 def test_all_nine_actual_paid_insets_both_seats(self):
  for i in range(9):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,True)
 def test_all_nine_actual_paid_normal_bodies_both_seats(self):
  for i in range(9):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,False)
 def test_nine_internal_inset_definitions_cannot_enter_decks(self):
  for n in NAMES:
   with self.subTest(card=n),self.assertRaises(EngineError):
    with Game([[FACTS[n]['card_faces'][1]['name']]*60,['Forest']*60],seed=1,deck_rules='Freeform'):pass
 def scenario(self,i,p,inset):
  q='P1'if p=='P0'else'P0';pi=int(p[-1]);name=NAMES[i]
  own=sum(([b]*9 for b in BASICS),[])+[PREFERRED[i]]*24+[name]*20+['Bear Cub']*16+['Shock']*12+['Snakeskin Veil']*12
  foe=['Forest']*36+['Mountain']*24+['Bear Cub']*20+['Candy Trail']*12+['Shock']*12
  with Game([own,foe]if p=='P0'else[foe,own],seed=2550+i,starting_player=p,deck_rules='Freeform')as g:
   stage='setup';source=None;base=None;descriptor=False;boosted=False;activated=False;target_ids=set();search_chosen=False;enemy_spell=None
   for iteration in range(30000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,inset,stage,iteration));o=g.observe(o['actor']);self.assertEqual(o['schema'],96);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p)
    def field(owner,n):return next((v for v in objects.values()if v['owner']==owner and v['zone']=='Battlefield'and v['card']==n),None)
    ally=field(p,'Bear Cub');enemy=field(q,'Bear Cub');artifact=field(q,'Candy Trail');body=objects.get(source)
    quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    if stage=='effect'and quiet:
     self.assertTrue(descriptor);self.assertFalse(o['adventures']);self.assertFalse(o['exile_permissions'])
     if inset:
      self.assertFalse(any(v['id']==source for v in g.observe(p)['objects']));self.assertFalse(any(v['id']==source for v in g.observe(q)['objects']))
      self.assertEqual(o['players'][pi]['library_count'],base['library']+1-({0:1,2:3,6:int(search_chosen),7:2}.get(i,0)))
      if i==0:self.assertTrue(objects[base['enemy']]['tapped'])
      if i in [1,8]:self.assertEqual(objects[base['enemy']]['zone'],'Graveyard')
      if i==3:self.assertEqual((ally['power'],ally['toughness']),(base['ally_power']+2,base['ally_toughness']+2));self.assertEqual(ally['keywords']&(64|4096),64|4096)
      if i==4:
       soldiers=[v for v in objects.values()if v['owner']==p and v['zone']=='Battlefield'and v.get('token')and v['token']['subtypes']=='Soldier'];self.assertEqual(len(soldiers),1);self.assertEqual((soldiers[0]['power'],soldiers[0]['toughness']),(2,2));self.assertEqual(soldiers[0]['token']['colors'],1)
      if i==5:self.assertEqual(objects[enemy_spell]['zone'],'Graveyard');self.assertEqual(o['players'][pi]['life'],base['life'])
      return
     self.assertEqual(body['zone'],'Battlefield');self.assertEqual(body['card'],name)
     front=FACTS[name]['card_faces'][0];power=int(front['power'])+(2 if i==5 and boosted else 1 if i==7 and activated else 0)
     self.assertEqual((body['power'],body['toughness']),(power,int(front['toughness'])));self.assertTrue(body['keywords']&1)
     if i==1:self.assertEqual(objects[base['artifact']]['zone'],'Graveyard')
     if i==2:self.assertEqual(len(target_ids),2);self.assertTrue(all(objects[t]['zone']=='Hand'for t in target_ids))
     if i==3:self.assertEqual(ally['counters']['plus_one'],0)
     if i in [6,8]:self.assertEqual(o['players'][pi]['life'],base['life']+(3 if i==6 else 5))
     if i==4 and ally['power']==base['ally_power']:pass
     elif i==5 and not boosted:pass
     elif i==7 and not activated:pass
     else:
      if i==4:self.assertEqual(ally['power'],base['ally_power']+1)
      if i==7:self.assertTrue(body['keywords']&4)
      return
    lands=[v for v in objects.values()if v['owner']==p and v['zone']=='Battlefield'and v['card']in BASICS and not v['tapped']]
    ready=len(lands)>=8 and ally and enemy and artifact and (inset or i!=3 or ally['counters']['plus_one']>0)
    for d in o['spell_characteristics']:
     if source is not None and d['object']['card']==source:
      f=FACTS[name]['card_faces'][int(inset)];self.assertEqual(d['physical_card'],name);self.assertEqual(d['name'],f['name']);self.assertEqual(d['type_line'],f['type_line']);self.assertEqual(d['mana_cost'],f['mana_cost']);self.assertEqual(d['face'],'Omen'if inset else'Normal');descriptor=True
      if inset:
       item=o['announcing']or next((x for x in o['stack']if x['card']==source),None)or o['resolving'];self.assertEqual(item['cast']['method'],'Normal');self.assertEqual(item['cast']['physical_card'],name)
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Attack','Cast','BeginCast','AnnounceAdventure','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
     if k=='AnnounceOmen':return 2600 if actor==p and inset and stage=='setup'and n==name and ready else -1000
     if k=='AnnounceCast':
      if actor==p:
       if n==name:return 2600 if not inset and stage=='setup'and ready else -1000
       if n=='Bear Cub':return 2000 if not ally else -1000
       if n=='Snakeskin Veil':return 2300 if not inset and i==3 and ally and ally['counters']['plus_one']==0 and stage=='setup'else -1000
       if n=='Shock':return 2500 if not inset and i==5 and stage=='effect'and body and body['zone']=='Battlefield'and not boosted else -1000
       return -1000
      if n=='Bear Cub':return 2000 if not enemy else -1000
      if n=='Candy Trail':return 2000 if not artifact else -1000
      if n=='Shock':return 2500 if inset and i==5 and stage=='setup'and ready and quiet and any(v['zone']=='Hand'and v['card']==name for v in mine['objects'])else -1000
      return -1000
     if k in ['AnnounceActivate','Activate']:return 2500 if not inset and i==7 and actor==p and stage=='effect'and a['card']==source and body and body['zone']=='Battlefield'and not activated else -1000
     if k=='ChooseTarget':
      t=a['target'];ann=o['announcing'];src=ann['source_card']if ann else''
      if 'Spell'in t:return 2700
      if 'Player'in t:return 2600 if t['Player']==(p if actor==q else q)else 1000
      v=objects[t['Card']]
      if src=='Snakeskin Veil':return 2700 if ally and v['id']==ally['id']else 500
      if not inset and i==2 and src==name:return 2700 if v['card']=='Bear Cub'else 500
      desired=artifact if not inset and i==1 else ally if i==3 or not inset else enemy
      return 2700 if desired and t['Card']==desired['id']else 500
     if k=='FinishTargets':return 2000
     if k=='PlayLand':return 1800-20*sum(v['owner']==actor and v['zone']=='Battlefield'and v['card']==n for v in objects.values())
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2100 if o['decision']in ['pay_costs','resolution_mana']and n in BASICS else -1000
     if k in ['PayCosts','PutTrigger','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost']:return 2400
     if k in ['Discard','Bottom']:return 2200 if n in BASICS else 1000
     if k=='ChooseSearch':return 2500 if not search_chosen else -1000
     if k=='FinishSearch':return 2000
     if k=='DeclineEffectPayment':return -1000
     if k=='Pass':return 0
     return 1000
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,p,inset,stage,o['decision']));ann=o['announcing']
    if a['kind']in ['AnnounceOmen','AnnounceCast']and actor==p and objects[a['card']]['card']==name:source=a['card'];stage='effect'
    if a['kind']=='PayCosts'and ann:
     if ann['kind']=='Spell'and ann['card']==source:
      base=dict(library=o['players'][pi]['library_count'],life=o['players'][pi]['life'],ally_power=ally['power'],ally_toughness=ally['toughness'],enemy=enemy['id'],artifact=artifact['id']);target_ids={b['target']['Card']for b in ann['targets']if 'Card'in b['target']}
     if ann['kind']=='Spell'and ann['source_card']=='Shock':
      if actor==q:enemy_spell=ann['card']
      elif i==5:boosted=True
     if ann['card']==source and ann['kind']!='Spell':activated=True
    if a['kind']=='ChooseTarget'and not inset and i==2 and ann and ann['source_card']==name and ann['kind']!='Spell':target_ids.add(a['target']['Card'])
    if a['kind']=='ChooseSearch':search_chosen=True
    before=[g.observe(v)for v in ['P0','P1']]
    with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor=q if actor==p else p),a)
    self.assertEqual(before,[g.observe(v)for v in ['P0','P1']]);self.replay(g,o,j)
   self.fail(('unreached',name,p,inset,stage))
if __name__=='__main__':unittest.main()
