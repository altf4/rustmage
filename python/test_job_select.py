"""Paid complete Job Select bodies, live recipient types and owned private choices."""
import unittest
from rustmage import EngineError, Game
NAMES=["Astrologian's Planisphere","Bard's Bow","Black Mage's Rod","Dark Knight's Greatsword","Dragoon's Lance","Machinist's Arsenal","Monk's Fist","Ninja's Blades","Paladin's Arms","Red Mage's Rapier","Sage's Nouliths","Samurai's Katana","Summoner's Grimoire","Thief's Knife","Warrior's Sword","White Mage's Staff"]
TYPES=['Wizard','Bard','Wizard','Knight','Knight','Artificer','Monk','Ninja','Knight','Wizard','Cleric','Samurai','Shaman','Rogue','Warrior','Cleric']
BONUS=[(0,0),(2,2),(1,0),(3,0),(1,0),(2,2),(1,0),(1,1),(2,1),(0,0),(1,0),(2,2),(0,0),(1,1),(3,2),(1,1)]
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class JobSelectTests(unittest.TestCase):
 def replay(self,g,o,j):
  actor=o['actor'];other='P1' if actor=='P0' else 'P0';saved=g.snapshot()
  with g.clone() as f:expected=f.step(f.observe(actor),j);expected_other=f.observe(other)
  if o.get('effect_choice') or any(x.get('attachment_trigger') for x in o['stack']):
   parent=g.clone();child=parent.clone();parent.close()
   with child:self.assertEqual(child.step(child.observe(actor),j),expected)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(after,expected)
  actual_other=g.observe(other);expected_other['revision']=actual_other['revision'];self.assertEqual(actual_other,expected_other)
  g.drop_snapshot(saved);return after
 def reject(self,g,o,a,actor=None):
  before=[g.observe(p) for p in ['P0','P1']]
  with self.assertRaises(EngineError):g.act(dict(o,actor=actor or o['actor']),a)
  self.assertEqual(before,[g.observe(p) for p in ['P0','P1']])
 def test_all_sixteen_paid_job_select_and_real_targeted_reequip_both_seats(self):
  for i in range(16):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,reequip=True)
 def test_all_recipient_granted_residuals_under_both_controllers(self):
  for i in [0,2,7,9,10,13,15]:
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,residual=True)
 def test_summoner_private_normal_enchantment_and_decline_routes(self):
  for p in ['P0','P1']:
   for route in ['normal','enchantment','decline']:
    with self.subTest(seat=p,route=route):self.scenario(12,p,residual=True,route=route)
 def test_actual_opponent_spell_pays_or_declines_recipient_ward(self):
  for p in ['P0','P1']:
   for pay in [False,True]:
    with self.subTest(seat=p,pay=pay):self.scenario(8,p,residual=True,ward=pay)
 def test_summoner_each_current_defender_choice_is_private_and_exact(self):
  for p in ['P0','P1']:
   for defender in range(3):
    with self.subTest(seat=p,defender=defender):self.scenario(12,p,residual=True,route='enchantment',walkers=True,defender=defender)
 def scenario(self,i,p,reequip=False,residual=False,route='normal',ward=None,walkers=False,defender=0):
  name=NAMES[i];other='P1' if p=='P0' else 'P0';pi=0 if p=='P0' else 1
  deck=sum(([b]*16 for b in BASICS),[])+[name]*16+['Bear Cub']*16+['Shock']*12+['Divination']*12+['Warrior\'s Sword']*12+['Appendage Amalgam']*12
  enemy=['Forest']*160 if ward is None else ['Mountain']*80+['Shock']*24
  if walkers:enemy=['Plains']*32+['Swamp']*32+['Forest']*32+['Ajani, Caller of the Pride']*16+['Professor Dellian Fel']*16
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=236+i,starting_player=p,deck_rules='Freeform') as g:
   source=hero=recipient=None;paid=False;job=False;equipped=False;triggered=False;attacked=False;selected=False;discarded=False;declined=False;entry=None;entry_before=None;cast_effect=False;baseline=None;checked_private=False;witness=False;chosen_defender=None;defender_checked=False
   for step in range(18000):
    public=g.observe('P0');self.assertIsNone(public['outcome'],(name,p,step));o=g.observe(public['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    walkers_now=sorted([v for v in objects.values() if v['owner']==other and v['zone']=='Battlefield' and v['card'] in ['Ajani, Caller of the Pride','Professor Dellian Fel']],key=lambda v:v['id'])
    own=[v for v in objects.values() if v['owner']==p]
    hand=[v for v in own if v['zone']=='Hand'];friends=[v for v in own if v['zone']=='Battlefield']
    if source in objects and objects[source]['zone']=='Stack':paid=True
    if quiet and source in objects and objects[source]['zone']=='Battlefield' and objects[source]['attached_to']:
     linked=objects[source]['attached_to'];body=objects[linked['card']]
     if not job:
      self.assertTrue(paid);hero=body['id'];recipient=hero;self.assertEqual(body['token']['subtypes'],'Hero');self.assertEqual(body['token']['colors'],0)
      self.assertIn(TYPES[i],body['granted_subtypes']);self.assertEqual((int(body['power']),int(body['toughness'])),(1+BONUS[i][0],1+BONUS[i][1]));job=True
      for viewer in [p,other]:self.assertIn(TYPES[i],next(v for v in g.observe(viewer)['objects'] if v['id']==hero)['granted_subtypes'])
     if reequip and body['card']=='Bear Cub':
      self.assertEqual((int(body['power']),int(body['toughness'])),(2+BONUS[i][0],2+BONUS[i][1]));self.assertIn(TYPES[i],body['granted_subtypes']);self.assertNotIn(TYPES[i],objects[hero]['granted_subtypes']);return
     if not reequip and not residual:return
    packet_list=o['stack']+o.get('ordering_triggers',[])+([o['announcing']] if o['announcing'] else [])+([o['resolving']] if o['resolving'] else [])
    for packet in packet_list:
     if packet and (ctx:=packet.get('attachment_trigger')) and ctx['granting_card']==name:
      triggered=True;self.assertTrue(ctx['granted']);self.assertEqual(ctx['recipient'],packet['object']);self.assertEqual(packet['controller'],p);self.assertEqual(packet['card'],recipient);self.assertEqual(packet['source_card'],objects[recipient]['card'])
    if residual and job and quiet:
     if i==8 and ward is not None and cast_effect and triggered:
      target=objects.get(recipient)
      if ward:self.assertTrue(target is None or target['zone']=='Outside')
      else:self.assertEqual(target['zone'],'Battlefield');self.assertEqual(int(target['damage']),0)
      return
     body=objects[recipient]
     if i==0 and cast_effect and int(body['counters']['plus_one'])==2:self.assertTrue(triggered);return
     if i==2 and cast_effect and int(o['players'][1-pi]['life'])==17:self.assertTrue(triggered);return
     if i==9 and cast_effect and int(body['power'])==3:self.assertTrue(triggered);return
     if i==10 and attacked and not body['tapped']:self.assertTrue(triggered);return
     if i==15 and attacked and int(o['players'][pi]['life'])==21:self.assertTrue(triggered);return
     if i==13 and attacked and int(o['players'][pi]['cards_drawn_this_turn'])>=2:self.assertTrue(triggered);return
     if i==7 and discarded and int(o['players'][1-pi]['life'])==14:self.assertTrue(triggered);return
     if i==12 and declined:self.assertTrue(triggered);self.assertTrue(any(v['id']==entry and v['zone']=='Hand' and v['reference']==entry_before for v in hand));return
     if i==12 and selected and entry in objects and objects[entry]['zone']=='Battlefield':
      self.assertTrue(triggered);self.assertNotEqual(objects[entry]['reference'],entry_before)
      if route=='enchantment':self.assertTrue(objects[entry]['tapped']);self.assertTrue(any(a['card']==entry for a in o['attack_declarations']));self.assertIsNone(o['library_choice']);self.assertEqual(next(a['defender'] for a in o['attack_declarations'] if a['card']==entry),chosen_defender or {'Player':other})
      else:self.assertFalse(objects[entry]['tapped']);self.assertFalse(any(a['card']==entry for a in o['attack_declarations']))
      return
    if (choice:=o.get('effect_choice')) and actor==p and i in [7,12]:
     self.assertIsNone(g.observe(other)['effect_choice'])
     if not checked_private:
      a=next(a for a in o['actions'] if a['kind'] in ['Discard','PutHandCreature','DeclineHandEntry']);self.reject(g,o,a,other)
      if i==12:self.reject(g,o,dict(kind='PutHandCreature',object=objects[source]['reference']))
      checked_private=True
    if o['decision']=='hand_defender' and not defender_checked:
     self.assertIsNone(g.observe(other)['effect_choice']);self.assertEqual(len([a for a in o['actions'] if a['kind']=='ChooseHandDefender']),3)
     stale=dict(walkers_now[0]['reference']);stale['generation']+=1;self.reject(g,o,dict(kind='ChooseHandDefender',defender={'Planeswalker':stale}));defender_checked=True
    ready=job and (not reequip or equipped)
    needs_attack=residual and i in [7,10,12,13,15]
    desired_entry='Appendage Amalgam' if route=='enchantment' else 'Bear Cub'
    if i==12 and not entry:
     chosen=next((v for v in hand if v['card']==desired_entry),None)
     if chosen:entry=chosen['id'];entry_before=chosen['reference']
    def score(a):
     k=a['kind']
     if k=='Concede':return -100000
     if k in ['AnnounceCast','Cast','BeginCast']:
      if actor!=p:
       n=objects[a['card']]['card']
       if walkers and n in ['Ajani, Caller of the Pride','Professor Dellian Fel']:return 1800 if not any(v['card']==n for v in walkers_now) else -10000
       return 2000 if ward is not None and job and not cast_effect and n=='Shock' else -10000
      n=objects[a['card']]['card']
      if n==name and source is None:return 2000
      if reequip and job and n=='Bear Cub' and not any(v['card']=='Bear Cub' for v in friends):return 1800
      if residual and job and not cast_effect and (i==0 and n=='Divination' or i in [2,9] and n=='Shock'):return 1900
      return -10000
     if k in ['AnnounceActivate','Activate','BeginActivate']:
      return 1900 if actor==p and reequip and job and a['card']==source and objects[source]['attached_to']['card']==hero and any(v['card']=='Bear Cub' for v in friends) else -10000
     if k=='ChooseTarget':
      t=a['target']
      if ward is not None:return 3000 if t==dict(Card=recipient) else -10000
      if i==10 and o['announcing'] and o['announcing'].get('attachment_trigger'):return 3000 if t==dict(Card=recipient) else -10000
      if reequip and o['announcing'] and o['announcing']['kind'].get('ActivatedAbility') is not None:return 3000 if isinstance(t,dict) and objects.get(t.get('Card'),{}).get('card')=='Bear Cub' else -10000
      return 3000 if t==dict(Player=other) else -10000
     if k=='PayResolutionMana':return 2000 if ward else -10000
     if k=='DeclineResolutionMana':return -10000 if ward else 2000
     if k=='PutHandCreature':return 2000 if a['object']['card']==entry and route!='decline' else -10000
     if k=='DeclineHandEntry':return 3000 if route=='decline' else -10000
     if k=='ChooseHandDefender':
      wanted={'Player':other} if defender==0 else {'Planeswalker':walkers_now[defender-1]['reference']}
      return 2000 if a['defender']==wanted else -10000
     if k=='Discard':
      if i==7 and o.get('effect_choice'):return 3000 if objects[a['card']]['card']=="Warrior's Sword" else -10000
      return 100 if objects[a['card']]['card'] not in [name,desired_entry,"Warrior's Sword"] else -100
     if k=='Attack':
      wait=walkers and len(walkers_now)!=2 or i==7 and not any(v['card']=="Warrior's Sword" for v in hand) or i==12 and entry is None
      return 2000 if actor==p and needs_attack and ready and not attacked and a['card']==recipient and not wait else -10000
     if k=='FinishAttackers':return 100
     if k in ['AttackPlaneswalker','Block']:return -10000
     if k in ['Keep','PayCosts','PutTrigger','ChooseSpellCost','FinishTargets']:return 1000
     if k=='PlayLand':
      color=BASICS.index(objects[a['card']]['card']);have=sum(v['card']==BASICS[color] and v['owner']==actor and v['zone']=='Battlefield' for v in objects.values());return 300-have*20
     if k in ['TapMana','ActivateMana']:return 10 if o['decision'] in ['pay_costs','resolution_mana'] else -100
     if k=='ChooseOptional':return 1000 if a['accept'] else -100
     if k=='Pass':return 0
     return 1
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j]
    if actor==p and a['kind'] in ['AnnounceCast','Cast','BeginCast']:
     if objects[a['card']]['card']==name:source=a['card']
     elif objects[a['card']]['card'] in ['Divination','Shock']:cast_effect=True
    if ward is not None and actor==other and a['kind'] in ['AnnounceCast','Cast','BeginCast']:cast_effect=True
    if a['kind']=='Attack' and actor==p:attacked=True
    if a['kind']=='PutHandCreature':selected=True;entry_before=a['object']
    if a['kind']=='ChooseHandDefender':chosen_defender=a['defender']
    if a['kind']=='DeclineHandEntry':declined=True
    if a['kind']=='Discard' and o.get('effect_choice') and i==7:discarded=True
    after=self.replay(g,o,j)
   self.fail((name,p,'pilot exhausted',source,job,triggered,attacked))
