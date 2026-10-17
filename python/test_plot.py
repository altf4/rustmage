"""Paid special actions and complete later-turn Plot casts through the public library."""
import unittest
import json
from pathlib import Path
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
from rustmage import EngineError, Game

NAMES = ['Beastbond Outcaster','Blacksnag Buzzard','Cunning Coyote',"Djinn of Fool's Fall",'Irascible Wolverine','Loan Shark','Slickshot Show-Off','Spinewoods Paladin','Stagecoach Security','Stingerback Terror','Demonic Ruckus','Rictus Robber','Rise of the Varmints']
BASICS = ['Plains','Island','Swamp','Mountain','Forest']

class PlotTests(unittest.TestCase):
 def replay(self,g,o,j):
  saved=g.snapshot();actor=o['actor'];other='P1' if actor=='P0' else 'P0'
  with g.clone() as f:expected=f.step(f.observe(actor),j);expected_other=f.observe(other)
  if o['plot_payment'] or o['plotted_cards']:
   parent=g.clone();child=parent.clone();parent.close()
   with child:self.assertEqual(child.step(child.observe(actor),j),expected)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(after,expected)
  actual_other=g.observe(other);expected_other['revision']=actual_other['revision'];self.assertEqual(actual_other,expected_other)
  g.drop_snapshot(saved);return after
 def denied(self,g,o,a):
  before=[g.observe(v) for v in ['P0','P1']]
  with self.assertRaises(EngineError):g.act(o,a)
  self.assertEqual([g.observe(v) for v in ['P0','P1']],before)
 def test_all_thirteen_paid_plot_and_complete_later_turn_casts_both_seats(self):
  for i in range(13):
   for p in ['P0','P1']:
    with self.subTest(name=NAMES[i],seat=p):self.scenario(i,p,True)
 def test_all_thirteen_ordinary_hardcasts_keep_complete_residuals_both_seats(self):
  for i in range(13):
   for p in ['P0','P1']:
    with self.subTest(name=NAMES[i],seat=p):self.scenario(i,p,False)
 def scenario(self,i,p,plotted):
  name=NAMES[i];pi=int(p[1]);other='P1' if p=='P0' else 'P0'
  preferred=['Forest','Swamp','Mountain','Island','Mountain','Island','Mountain','Forest','Plains','Mountain','Mountain','Swamp','Forest'][i]
  deck=sum(([b]*4 for b in BASICS),[])+[preferred]*20+['Forest']*16+['Mountain']*12+[name]*16+['Bear Cub']*16+['Shock']*16+['Ambush Wolf']*8
  enemy=['Forest']*45+['Bear Cub']*15
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=240+i,starting_player=p,deck_rules='Freeform') as g:
   source=None;plotted_turn=None;special=False;cast=False;death=False;death_turn=None;loan_turn=None;paid=False;baseline=None
   for _ in range(30000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,plotted));o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']}
    quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    own=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield'];bears=[v for v in own if v['card']=='Bear Cub'];giants=[v for v in own if v['card']=='Ambush Wolf'];lands=[v for v in own if v['card'] in BASICS and not v['tapped']]
    hand=[v for v in objects.values() if v['owner']==p and v['zone']=='Hand'];shock=any(v['card']=='Shock' for v in hand)
    source_fundable=any(a['kind']=='AnnounceCast' and objects[a['card']]['card']==name for a in o['actions'])
    ready=(i!=0 or giants) and (i not in [2,8,10] or bears) and (i not in [1,11,12] or death_turn==o['turn']) and (i!=5 or loan_turn==o['turn']) and (i!=9 or len(hand)<=(6 if plotted else 7))
    if paid and quiet:
     body=objects[source]
     self.assertEqual(body['zone'],'Graveyard' if i==12 else 'Battlefield')
     if i in [0,5]:self.assertEqual(o['players'][pi]['library_count'],baseline['library']-1)
     if i==1:self.assertEqual((body['power'],body['toughness']),(3,2))
     if i==2:self.assertTrue(any(v['power']==3 and v['toughness']==3 and v['keywords']&4 for v in bears))
     if i==3:self.assertEqual((body['power'],body['toughness'],body['keywords']&1),(4,3,1))
     if i==4:self.assertEqual(len(o['exile_permissions']),baseline['rights']+1);self.assertEqual(o['players'][pi]['library_count'],baseline['library']-1)
     if i==6:self.assertTrue(body['keywords']&1 and body['keywords']&4)
     if i==7:self.assertEqual(o['players'][pi]['life'],baseline['life']+3)
     if i==8:self.assertTrue(body['keywords']&8);self.assertTrue(any(v['card']=='Bear Cub' and v['power']==3 and v['keywords']&8 for v in own))
     if i==9:self.assertEqual((body['power'],body['toughness']),(7-o['players'][pi]['hand_count'],7-o['players'][pi]['hand_count']))
     if i==10:self.assertTrue(any(v['card']=='Bear Cub' and v['power']==3 and v['keywords']&32768 and v['keywords']&256 for v in own))
     if i in [11,12]:
      tokens=[v for v in own if v.get('token') and v['token']['subtypes']==('Zombie Rogue' if i==11 else 'Varmint')]
      self.assertEqual(len(tokens),baseline['grave_creatures'] if i==12 else 1);self.assertGreater(len(tokens),0);self.assertEqual((tokens[0]['power'],tokens[0]['toughness'],tokens[0]['token']['colors']),(2,2 if i==11 else 1,6 if i==11 else 16))
     self.assertFalse(any(x['object']['card']==source for x in o['plotted_cards']));return
    request=o['plot_payment']
    if request:
     self.assertEqual(actor,p);self.assertIsNone(g.observe(other)['plot_payment']);self.assertFalse(o['stack']);self.assertIsNone(o['announcing']);self.assertEqual(objects[request['object']['card']]['zone'],'Hand')
     self.denied(g,o,{'kind':'Pass'});self.denied(g,dict(o,actor=other),{'kind':'PayPlot','payment':[0]*6})
    if special and o['turn']==plotted_turn:
     self.denied(g,o,{'kind':'AnnouncePlotted','card':source})
    def score(a):
     k=a['kind']
     if k in ['Cast','BeginCast','AnnounceExile','AnnounceFlashback']:return -1000
     if k=='AnnouncePlot':return 1800 if actor==p and plotted and not special and objects[a['card']]['card']==name and len(lands)>=5 else -1000
     if k=='AnnouncePlotted':return 1900 if actor==p and a['card']==source and ready else -1000
     if k=='AnnounceCast':
      n=objects[a['card']]['card']
      if actor!=p:return -1000
      if n==name:return 1800 if not plotted and not cast and ready else -1000
      if n=='Ambush Wolf':return 1750 if i==0 and not giants else -1000
      if n=='Bear Cub':
       need=(i in [1,2,8,10,11,12] and not bears and not death) or (i==9 and len(hand)>6)
       loan=i==5 and loan_turn!=o['turn'] and len(lands)>=7 and (not plotted or special and o['turn']>plotted_turn) and (plotted or source_fundable)
       return 1750 if need or loan else -1000
      if n=='Shock':return 1850 if i in [1,11,12] and bears and not death and len(lands)>=6 and (special and o['turn']>plotted_turn if plotted else source_fundable) else -1000
      return -1000
     if k=='ChooseTarget':return 2100 if a['target']=={'Card':bears[0]['id']} and bears else 1000
     if k=='PlayLand':
      n=objects[a['card']]['card'];count=sum(v['card']==n for v in own)
      return 1600 if n==preferred and count<2 else 1580 if n=='Forest' and count<2 else 1500-count*20
     if k=='TapMana':return 1300 if o['step'] in ['Main1','Main2'] or o['plot_payment'] or o['announcing'] else -1000
     if k in ['PayCosts','PayPlot','PutTrigger']:return 2200
     if k in ['FinishTargets','Keep','FinishAttackers','FinishBlockers']:return 1000
     if k in ['Bottom','Discard']:
      n=objects[a['card']]['card'];return -500 if n==name else 100 if n in ['Shock','Bear Cub','Ambush Wolf'] else 1100
     if k=='Pass':return 0
     if k in ['Attack','Concede']:return -2000
     return 500
    # Guard target lookup before indexing the possibly empty creature list.
    def safe_score(a):
     if a['kind']=='ChooseTarget':return 2100 if bears and a['target']=={'Card':bears[0]['id']} else 1000
     return score(a)
    j=max(range(len(o['actions'])),key=lambda j:safe_score(o['actions'][j]));a=o['actions'][j];ann=o['announcing']
    if a['kind']=='AnnouncePlot':source=a['card']
    if a['kind']=='PayPlot':
     before_count=o['players'][pi]['spells_cast_this_turn'];before_ref=request['object'];before_next=o['next_stack_id'] if 'next_stack_id'in o else None
    if a['kind'] in ['AnnounceCast','AnnouncePlotted'] and objects[a['card']]['card']==name:source=a['card'];cast=True
    if a['kind']=='PayCosts' and ann and ann['kind']=='Spell':
     if ann['source_card']=='Shock':death=True;death_turn=o['turn']
     if ann['source_card']=='Bear Cub' and i==5:loan_turn=o['turn']
     if ann['source_card']==name:paid=True;baseline={'library':o['players'][pi]['library_count'],'life':o['players'][pi]['life'],'rights':len(o['exile_permissions']),'grave_creatures':sum(v['owner']==p and v['zone']=='Graveyard' and not v.get('token') and 'Creature' in FACTS.get(v['card'],{}).get('type_line','').split(' — ')[0] for v in objects.values())}
    after=self.replay(g,o,j)
    if a['kind']=='PayPlot':
     special=True;plotted_turn=o['turn'];self.assertEqual(after['actor'],p);self.assertEqual(after['players'][pi]['spells_cast_this_turn'],before_count)
     permission=next(v for v in after['plotted_cards'] if v['object']['card']==source);self.assertEqual(permission['object']['generation'],before_ref['generation']+1);self.assertEqual(permission['plotted_turn'],plotted_turn);self.assertEqual(after['plotted_cards'],g.observe(other)['plotted_cards'])
    if a['kind']=='PayCosts' and ann and ann['kind']=='Spell' and ann['source_card']==name:
     spell=next(v for v in after['stack'] if v['card']==source and v['kind']=='Spell');self.assertEqual(spell['cast']['origin'],'Exile' if plotted else 'Hand');self.assertEqual(spell['cast']['method'],'Plot' if plotted else 'Normal')
   self.fail(('no completed body',name,p,plotted))
