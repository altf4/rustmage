"""Actual paid complete mana-spent bodies and actor-owned exact color allocation."""
import unittest
from rustmage import Game,EngineError
NAMES=['Aberrant Manawurm','Blazing Bomb','Cuboid Colony','Deluge Virtuoso','Exhibition Tidecaller','Expressive Firedancer','Freestrider Commando','Hungry Graffalon','Muse Seeker','Prompto Argentum','Rancorous Archaic','Sahagin','Shantotto, Tactician Magician','Spectacular Skywhale','Tackle Artist','Textbook Tabulator','Thunderdrum Soloist','Ultros, Obnoxious Octopus','Unravel','Arcane Omens']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class CastPaymentTests(unittest.TestCase):
 def replay(self,g,o,a):
  actor=o['actor'];other='P1'if actor=='P0'else'P0';saved=g.snapshot();parent=g.clone();child=parent.clone();parent.close()
  with child:expected=child.act(child.observe(actor),a);opponent=child.observe(other)
  before=[g.observe(p)for p in ['P0','P1']]
  if a['kind']=='PayCosts' and o['mana_payment']:
   invalid=dict(a,payment=list(a['payment']));invalid['payment'][5]+=1
   with self.assertRaises(EngineError):g.act(o,invalid)
   self.assertEqual(before,[g.observe(p)for p in ['P0','P1']])
  with self.assertRaises(EngineError):g.act(dict(o,actor=other),a)
  self.assertEqual(before,[g.observe(p)for p in ['P0','P1']]);g.act(o,a);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.act(o,a)
  after=g.act(g.observe(actor),a);expected['revision']=after['revision'];self.assertEqual(after,expected);v=g.observe(other);opponent['revision']=v['revision'];self.assertEqual(v,opponent);g.drop_snapshot(saved)
  if a['kind']=='PayCosts' and o['announcing'] and o['announcing']['kind']=='Spell':
   packet=next(v for v in after['stack'] if v['id']==o['announcing']['id'])
   self.assertEqual(packet['cast']['payment'],a['payment'])
  return after
 def test_eighteen_actual_creatures_and_all_paid_low_high_residuals_both_seats(self):
  for n in NAMES[:18]:
   for p in ['P0','P1']:
    with self.subTest(name=n,seat=p):self.creature(n,p)
 def test_real_zero_plot_entry_and_nonrepresentative_color_outcome_both_seats(self):
  for p in ['P0','P1']:
   with self.subTest(name='Freestrider Commando',seat=p,route='Plot'):self.creature('Freestrider Commando',p,plot=True)
   with self.subTest(name='Rancorous Archaic',seat=p,colors=3):self.creature('Rancorous Archaic',p,colors=3)
 def creature(self,name,p,plot=False,colors=5):
  pi=int(p[1]);q='P1'if p=='P0'else'P0';qi=1-pi
  own=sum(([b]*16 for b in BASICS),[])+[name]*16+['Shock']*12+['Mind Spring']*12
  enemy=['Forest']*88+['Bear Cub']*16
  with Game([own,enemy]if p=='P0'else[enemy,own],seed=2490+NAMES.index(name),starting_player=p,deck_rules='Freeform')as g:
   stage='setup';source=None;reference=None;before=None;payments={};seen=set();plot_turn=None;nonrepresentative=False;replayed=0
   for step in range(30000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,plot,stage,step));o=g.observe(o['actor']);actor=o['actor'];objs={v['id']:v for v in o['objects']};mine=[v for v in objs.values()if v['owner']==p and v['zone']=='Battlefield'];theirs=[v for v in objs.values()if v['owner']==q and v['zone']=='Battlefield'];hand=[v for v in objs.values()if v['owner']==p and v['zone']=='Hand'];lands=[v for v in mine if v['card']in BASICS and not v['tapped']];target=next((v for v in theirs if v['card']=='Bear Cub'),None);quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    domain=o['mana_payment']
    if domain:
     self.assertIsNone(g.observe(q if actor==p else p)['mana_payment']);self.assertEqual(len(domain['available']),6)
    for item in o['stack']+o['waiting_triggers']+([o['announcing']]if o['announcing']else[])+([o['resolving']]if o['resolving']else[]):
     event=item['cast_event']
     if event and item['source_card']==name:
      self.assertEqual(event['controller'],p);self.assertEqual(item['object'],reference);self.assertEqual(event['payment'],payments[event['object']['card']]);self.assertIn(event['card'],['Shock','Mind Spring']);seen.add(event['card'])
    if o['library_choice']or o['decision']=='effect_discard':
     other=q if actor==p else p;self.assertEqual(g.observe(other)['actions'],[]);self.assertIsNone(g.observe(other)['library_choice'])
    if quiet and stage=='source_effect':
     body=objs[source];self.assertEqual(body['zone'],'Battlefield');reference=body['reference']
     if name=='Rancorous Archaic':self.assertEqual(body['counters']['plus_one'],colors);self.assertEqual((body['power'],body['toughness']),(2+colors,2+colors));self.assertTrue(nonrepresentative if colors==3 else True);return
     if name=='Freestrider Commando':self.assertEqual(body['counters']['plus_one'],2 if plot else 0);self.assertEqual(body['power'],5 if plot else 3);return
     if name=='Deluge Virtuoso':self.assertEqual(target['counters']['stun'],1)
     stage='low'
    if quiet and stage in ['low_effect','high_effect']:
     high=stage=='high_effect';body=objs[source];old=before['source'];delta=5 if high else 1;counter=body['counters']['plus_one']-old['counters']['plus_one'];draws=o['players'][pi]['cards_drawn_this_turn']-before['draws'];damage=before['enemy_life']-o['players'][qi]['life'];bonus=(body['power']-old['power'],body['toughness']-old['toughness'])
     if name=='Aberrant Manawurm':self.assertEqual(bonus,(delta,0))
     if name=='Blazing Bomb':self.assertEqual(counter,int(high))
     if name=='Cuboid Colony':self.assertEqual(counter,int(high))
     if name=='Deluge Virtuoso':self.assertEqual(bonus,(2 if high else 1,)*2)
     if name=='Exhibition Tidecaller':self.assertEqual(o['players'][qi]['library_count'],before['enemy_library']-(10 if high else 3))
     if name=='Expressive Firedancer':self.assertEqual(bonus,(1,1));self.assertEqual(bool(body['keywords']&1024),high)
     if name=='Hungry Graffalon':self.assertEqual(counter,int(high))
     if name=='Muse Seeker':self.assertEqual(draws,4 if high else 1);self.assertEqual(o['players'][pi]['cards_discarded_this_turn']-before['discards'],0 if high else 1)
     if name=='Prompto Argentum':self.assertEqual(sum(bool(v.get('token')and v['token']['subtypes']=='Treasure')for v in mine),before['treasures']+int(high))
     if name=='Sahagin':self.assertEqual(counter,int(high));self.assertEqual(bool(body['keywords']&16384),high)
     if name=='Shantotto, Tactician Magician':self.assertEqual(bonus,(delta,0));self.assertEqual(draws,4 if high else 0)
     if name=='Spectacular Skywhale':self.assertEqual(bonus,(3,3 if high else 0));self.assertEqual(counter,3 if high else 0)
     if name=='Tackle Artist':self.assertEqual(counter,2 if high else 1)
     if name=='Textbook Tabulator':self.assertEqual(counter,1)
     if name=='Thunderdrum Soloist':self.assertEqual(damage,3)
     if name=='Ultros, Obnoxious Octopus':self.assertEqual(counter,0);self.assertEqual(target['counters']['stun'],before['stun']+int(high))
     if high:
      self.assertIn('Mind Spring',seen)
      if name=='Blazing Bomb':stage='bomb'
      else:return
     else:stage='high'
    if quiet and stage=='bomb_effect':self.assertEqual(objs[source]['zone'],'Graveyard');self.assertEqual(objs[before['target']]['zone'],'Graveyard');return
    source_ready=len(lands)>=8 and target and all(any(v['card']==b for v in lands)for b in ['Island','Mountain','Forest'])
    if name=='Rancorous Archaic':source_ready=source_ready and all(sum(v['card']==b for v in lands)>=(3 if colors==3 else 1)for b in BASICS)
    high_ready=len(lands)>=5 and sum(v['card']=='Island'for v in lands)>=2
    wanted=([1,1,1,1,1,0]if colors==5 else[3,1,1,0,0,0])if name=='Rancorous Archaic'else None
    float_target=[3,3,3,3,3,0]if name=='Rancorous Archaic'and colors==3 else wanted
    def score(a):
     k=a['kind']
     if k=='Keep':return 3000
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','Attack','AnnounceWarp','AnnounceFlashback']:return -10000
     if k=='AnnouncePlot':return 2300 if plot and actor==p and stage=='setup'and objs[a['card']]['card']==name and source_ready else -1000
     if k=='AnnouncePlotted':return 2300 if plot and actor==p and stage=='plotted'and a['card']==source and o['turn']>plot_turn else -1000
     if k=='AnnounceCast':
      n=objs[a['card']]['card']
      if actor!=p:return 1800 if n=='Bear Cub'and not target else -1000
      return 2300 if (n==name and stage=='setup'and source_ready and not plot)or(n=='Shock'and stage=='low'and len(lands)>=1 and any(v['card']=='Mountain'for v in lands))or(n=='Mind Spring'and stage=='high'and high_ready)else -1000
     if k=='AnnounceActivate':return 2300 if actor==p and stage=='bomb'and a['card']==source else -1000
     if k=='ChooseX':return 2500 if a['value']==3 else -1000
     if k=='ChooseTarget':
      if stage=='bomb'and a['target'].get('Card')==(target or{}).get('id'):return 2700
      if a['target'].get('Player')==q:return 2600
      if target and a['target'].get('Card')==target['id']:return 2500
      return 1000
     if k=='PayCosts':
      if wanted and o['announcing']['source_card']==name:
       mana=o['players'][pi]['mana']
       return 2800 if a['payment']==wanted and all(mana[i]>=float_target[i]for i in range(6))else -1000
      return 2600
     if k=='PayPlot':return 2600
     if k in ['TapMana','ActivateMana','AnnounceMana']:
      if o['decision']not in ['pay_costs','plot_payment']:return -1000
      if wanted and o['announcing']and o['announcing']['source_card']==name:
       color=BASICS.index(objs[a['card']]['card']);return 2500 if o['players'][pi]['mana'][color]<float_target[color]else -1000
      return 2200
     if k=='PlayLand':
      n=objs[a['card']]['card'];count=sum(v['owner']==actor and v['zone']=='Battlefield'and v['card']==n for v in objs.values());return (1900 if count==0 else 1800 if n=='Island'and count<2 else 1700)-20*count
     if k in ['Discard','Bottom']:
      n=objs[a['card']]['card'];copies=sum(v['card']==n for v in hand);return -500 if actor==p and copies==1 and n in [name,'Shock','Mind Spring']else 2000
     if k=='ArrangeLibrary':return 2600 if a['destination']=='Top'else 1000
     if k in ['PutTrigger','FinishTargets','FinishAttackers','FinishBlockers']:return 2500
     if k=='Pass':return 0
     return 500
    if wanted and domain and o['announcing']['source_card']==name and all(o['players'][pi]['mana'][i]>=float_target[i]for i in range(6)):
     a=dict(kind='PayCosts',payment=wanted)
     if colors==3:self.assertNotIn(a,o['actions']);nonrepresentative=True
    else:a=max(o['actions'],key=score)
    self.assertGreaterEqual(score(a),0,(name,p,stage,o['decision'],a))
    if a['kind']=='AnnouncePlot':source=a['card']
    if a['kind']=='PayPlot':plot_turn=o['turn'];stage='plotted'
    if a['kind']=='PayCosts'and o['announcing']:
     root=o['announcing'];self.assertIsNotNone(domain)
     if root['kind']=='Spell':
      payments[root['card']]=a['payment'];self.assertEqual(root['cast']['payment'],[0]*6)
      if actor==p and root['source_card']==name:source=root['card'];stage='source_effect'
      elif actor==p and root['source_card']in ['Shock','Mind Spring']:
       body=objs[source];before=dict(source=body,draws=o['players'][pi]['cards_drawn_this_turn'],discards=o['players'][pi]['cards_discarded_this_turn'],enemy_life=o['players'][qi]['life'],enemy_library=o['players'][qi]['library_count'],treasures=sum(bool(v.get('token')and v['token']['subtypes']=='Treasure')for v in mine),stun=target['counters']['stun']);stage='low_effect'if root['source_card']=='Shock'else'high_effect'
     elif actor==p and root['card']==source and stage=='bomb':before=dict(target=target['id']);stage='bomb_effect'
    self.replay(g,o,a);replayed+=1
   self.fail(('incomplete creature',name,p,plot,colors,stage,replayed))
 def test_arcane_omens_actual_one_and_five_color_private_discards_both_seats(self):
  for p in ['P0','P1']:
   for colors in [1,5]:
    with self.subTest(seat=p,colors=colors):self.spells(p,colors=colors)
 def test_unravel_real_plot_zero_and_uncounterable_discount_both_seats(self):
  for p in ['P0','P1']:
   for route in ['Plot','Discount']:
    with self.subTest(seat=p,route=route):self.spells(p,route=route)
 def spells(self,p,colors=1,route=None):
  q='P1' if p=='P0' else 'P0';pi=int(p[1]);qi=1-pi
  own=sum(([b]*20 for b in BASICS),[])+(['Unravel']*20 if route else ['Arcane Omens']*20)
  enemy=(['Island']*72+["Djinn of Fool's Fall"]*24 if route=='Plot' else ['Forest']*40+['Swamp']*40+['Honest Rutstein']*20+['Gigantic Big Bear']*20 if route else ['Forest']*120)
  name='Unravel' if route else 'Arcane Omens';victim=None;spell=None;stage='setup';plot_turn=None;draws=None;discards=None;cast_payment=None;seen_private=False
  with Game([own,enemy] if p=='P0' else [enemy,own],seed=24995+colors+(route=='Discount'),starting_player=q,deck_rules='Freeform')as g:
   for step in range(30000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(p,route,colors,stage,step));o=g.observe(o['actor']);actor=o['actor'];objs={v['id']:v for v in o['objects']};mine=[v for v in objs.values()if v['owner']==p and v['zone']=='Battlefield'];lands=[v for v in mine if v['card']in BASICS and not v['tapped']];enemy_bf=[v for v in objs.values()if v['owner']==q and v['zone']=='Battlefield'];qlands=[v for v in enemy_bf if v['card']in BASICS and not v['tapped']];hand=[v for v in g.observe(p)['objects']if v['owner']==p and v['zone']=='Hand'];ready=any(v['card']==name for v in hand)and len(lands)>=(3 if route else 5)and sum(v['card']=='Island'for v in lands)>=2
    if not route:ready=ready and (sum(v['card']=='Swamp'for v in lands)>=5 if colors==1 else all(any(v['card']==b for v in lands)for b in BASICS))
    quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving']
    if stage=='effect'and o['decision']=='effect_discard':
     self.assertEqual(actor,q);self.assertEqual(g.observe(p)['actions'],[]);seen_private=True
    if stage=='effect'and quiet:
     self.assertEqual(objs[spell]['zone'],'Graveyard')
     if route:
      self.assertEqual(o['players'][pi]['cards_drawn_this_turn'],draws+1)
      self.assertEqual(objs[victim]['zone'],'Battlefield' if route=='Discount'else'Graveyard')
     else:self.assertTrue(seen_private);self.assertEqual(o['players'][qi]['cards_discarded_this_turn'],discards+colors)
     return
    if route and victim is not None:
     for item in o['stack']:
      if item['card']==victim:
       cast_payment=item['cast']['payment'];self.assertEqual(sum(cast_payment),0 if route=='Plot'else 6)
    wanted=None if route else ([0,0,5,0,0,0]if colors==1 else[1,1,1,1,1,0])
    def score(a):
     k=a['kind']
     if k=='Keep':return 3000
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','Attack','AnnounceWarp','AnnounceFlashback','AnnounceActivate']:return -10000
     if k=='AnnouncePlot':return 2400 if route=='Plot'and actor==q and stage=='setup'and ready and len(qlands)>=4 else -1000
     if k=='AnnouncePlotted':return 2400 if route=='Plot'and actor==q and stage=='plotted'and a['card']==victim and o['turn']>plot_turn and ready else -1000
     if k=='AnnounceCast':
      n=objs[a['card']]['card']
      if actor==p:
       return 2600 if n==name and ((not route and stage=='setup'and ready)or(route and stage=='victim'and any(s['card']==victim for s in o['stack'])))else -1000
      if route=='Discount'and ready and len(qlands)>=8:
       return 2300 if (n=='Honest Rutstein'and not any(v['card']==n for v in enemy_bf))or(n=='Gigantic Big Bear'and stage=='setup'and any(v['card']=='Honest Rutstein'for v in enemy_bf))else -1000
      return -1000
     if k=='ChooseTarget':return 2800 if route and a['target'].get('Spell')==victim else 2600 if not route and a['target'].get('Player')==q else -1000
     if k=='PayCosts':
      if actor==p and not route:return 2700 if a['payment']==wanted else -1000
      return 2700
     if k=='PayPlot':return 2700
     if k in ['TapMana','ActivateMana','AnnounceMana']:
      if o['decision']not in ['pay_costs','plot_payment']:return -1000
      if actor==p and wanted:
       color=BASICS.index(objs[a['card']]['card']);return 2500 if o['players'][pi]['mana'][color]<wanted[color]else -1000
      return 2200
     if k=='PlayLand':
      n=objs[a['card']]['card'];count=sum(v['owner']==actor and v['zone']=='Battlefield'and v['card']==n for v in objs.values());return (1900 if count==0 else 1800 if n=='Island'and count<2 else 1700)-20*count
     if k in ['Discard','Bottom']:
      n=objs[a['card']]['card'];copies=sum(v['owner']==actor and v['zone']=='Hand'and v['card']==n for v in objs.values());return -500 if copies==1 and n in [name,'Honest Rutstein','Gigantic Big Bear',"Djinn of Fool's Fall"]else 2000
     if k in ['PutTrigger','FinishTargets','FinishAttackers','FinishBlockers']:return 2500
     if k=='ArrangeLibrary':return 2600 if a['destination']=='Top'else 1000
     if k=='Pass':return 0
     return 500
    if wanted and o['mana_payment']and o['announcing']['source_card']==name and all(o['players'][pi]['mana'][i]>=wanted[i]for i in range(6)):a=dict(kind='PayCosts',payment=wanted)
    else:a=max(o['actions'],key=score)
    self.assertGreaterEqual(score(a),0,(p,route,colors,stage,o['decision'],a))
    if a['kind']=='AnnouncePlot':victim=a['card']
    if a['kind']=='PayPlot':stage='plotted';plot_turn=o['turn']
    if a['kind']=='PayCosts'and o['announcing']:
     item=o['announcing']
     if item['source_card']in ["Djinn of Fool's Fall",'Gigantic Big Bear']:victim=item['card'];stage='victim'
     if actor==p and item['source_card']==name:
      spell=item['card'];draws=o['players'][pi]['cards_drawn_this_turn'];discards=o['players'][qi]['cards_discarded_this_turn'];stage='effect'
      if route:self.assertIsNotNone(cast_payment)
      else:self.assertEqual(a['payment'],wanted)
    self.replay(g,o,a)
   self.fail(('incomplete spell',p,route,colors,stage))
