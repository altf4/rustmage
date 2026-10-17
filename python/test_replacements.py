"""Real paid replacement sources, residual actions and owned continuations."""
import unittest
from rustmage import Game,EngineError
NAMES=['Doubling Season','Exalted Sunborn','Loading Zone','Michelangelo, Weirdness to 11','Yoshimaru, Beloved Companion','Caradora, Heart of Alacria','Quina, Qu Gourmet','Draconic Visitor','Bartolomé del Presidio']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class ReplacementTests(unittest.TestCase):
 def replay(self,g,o,j):
  actor=o['actor'];other='P1' if actor=='P0' else 'P0';saved=g.snapshot();parent=g.clone();child=parent.clone();parent.close()
  with child:expected=child.step(child.observe(actor),j);other_expected=child.observe(other)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(after,expected);other_actual=g.observe(other);other_expected['revision']=other_actual['revision'];self.assertEqual(other_actual,other_expected);g.drop_snapshot(saved);return after
 def deny(self,g,o,a,actor=None):
  before=[g.observe(p) for p in ['P0','P1']]
  with self.assertRaises(EngineError):g.act(dict(o,actor=actor or o['actor']),a)
  self.assertEqual(before,[g.observe(p) for p in ['P0','P1']])
 def test_nine_complete_paid_bodies_with_actual_activation_costs_and_search_both_seats(self):
  for i in range(9):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p)
 def test_noncommuting_orders_and_real_multiplied_job_recipient_choice_both_seats(self):
  for mode in ['counter','token','job']:
   for first in [False,True]:
    for p in ['P0','P1']:
     with self.subTest(mode=mode,double_first=first,seat=p):self.scenario(0,p,mode,first)
 def scenario(self,i,p,mode='body',first=True):
  name=NAMES[i];pi=int(p[1]);other='P1' if p=='P0' else 'P0';support='Michelangelo, Weirdness to 11' if mode=='counter' else 'Quina, Qu Gourmet'
  names=list(dict.fromkeys([name,'Bear Cub','Candy Trail','Cartographer\'s Companion','Snakeskin Veil','Alacrian Jaguar',support,"Bard's Bow"]))
  own=sum(([b]*20 for b in BASICS),[])+sum(([n]*16 for n in names),[]);enemy=['Forest']*240
  with Game([own,enemy] if p=='P0' else [enemy,own],seed=2480+i,starting_player=p,deck_rules='Freeform') as g:
   source=None;phase='setup';baseline=None;done=False;search=False;job=None;selected=None;replacements=0;activation=False
   for step in range(22000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,mode,phase,step));o=g.observe(o['actor']);actor=o['actor'];objs={v['id']:v for v in o['objects']};ownbf=[v for v in objs.values() if v['owner']==p and v['zone']=='Battlefield'];bear=next((v for v in ownbf if v['card']=='Bear Cub'),None);source_view=objs.get(source);quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving'];by_name=lambda n:next((v for v in ownbf if v['card']==n),None)
    if phase=='cast' and quiet and source_view and source_view['zone']=='Battlefield':
     phase='effect'
     if i==1:self.assertTrue(source_view['keywords']&1 and source_view['keywords']&64)
     if i==3:self.assertTrue(any(v.get('token') and v['token']['subtypes']=='Mutagen' for v in ownbf))
     if i==5:self.assertTrue(search);self.assertTrue(any(v['owner']==p and v['zone']=='Hand' and v['card']=='Alacrian Jaguar' for v in objs.values()))
     baseline=dict(tokens=sum(v.get('token') is not None for v in ownbf),counter=int(bear['counters']['plus_one']),library=o['players'][pi]['library_count'])
    if done and quiet:
     self.assertEqual(o['schema'],74)
     if mode=='job':self.assertEqual(objs[job]['attached_to'],selected);recipient=objs[selected['card']];self.assertIn('Bard',recipient['granted_subtypes']);self.assertEqual(recipient['token']['subtypes'],'Frog' if first else 'Hero');return
     if mode=='counter':self.assertEqual(int(bear['counters']['plus_one']),baseline['counter']+(3 if first else 4));self.assertGreater(replacements,0);return
     if mode=='token':self.assertEqual(sum(v.get('token') is not None for v in ownbf),baseline['tokens']+(3 if first else 4));self.assertGreater(replacements,0);return
     if i in [0,1,6,7]:
      delta=2 if i in [0,1,6] else 1
      self.assertEqual(sum(v.get('token') is not None for v in ownbf),baseline['tokens']+delta-(1 if i==6 else 0))
      if i==6:self.assertEqual(int(source_view['counters']['plus_one']),1)
      if i==7:
       dragon=next(v for v in ownbf if v.get('token') and v['token']['subtypes']=='Dragon');self.assertEqual((int(dragon['power']),int(dragon['toughness']),dragon['token']['colors']),(5,5,8));self.assertTrue(dragon['keywords']&1)
     elif i==4:self.assertEqual(int(source_view['counters']['plus_one']),2)
     elif i==8:self.assertEqual(int(source_view['counters']['plus_one']),1);self.assertIsNone(by_name('Candy Trail'))
     else:self.assertEqual(int(bear['counters']['plus_one']),baseline['counter']+2)
     return
    needs_support=mode!='body';supported=by_name(support) if needs_support else True
    lands=[v for v in ownbf if v['card']in BASICS];ready=len(lands)>=8 and bear and (i!=8 or by_name('Candy Trail')) and supported
    def score(a):
     k=a['kind']
     if k in ['Cast','BeginCast','AnnounceWarp','AnnounceExile','AnnouncePlotted','AnnounceFlashback','Attack','Concede']:return -1000
     if k=='AnnounceCast':
      n=objs[a['card']]['card']
      if actor!=p:return -1000
      if phase=='setup':
       if n=='Bear Cub' and not bear:return 1900
       if n=='Candy Trail' and i==8 and not by_name(n):return 1900
       if needs_support and n==support and not supported:return 1900
       if n==name and ready:return 2100
      if phase=='effect':
       if mode=='job' and n=="Bard's Bow":return 2100
       if (mode=='token' or mode=='body' and i in [0,1,6,7]) and n=="Cartographer's Companion":return 2100
       if (mode=='counter' or mode=='body' and i in [2,3,5]) and n=='Snakeskin Veil':return 2100
      return -1000
     if k=='AnnounceActivate':return 2100 if actor==p and source is not None and a['card']==source and phase in ['effect','sacrifice'] and mode=='body' and i in [4,6,8] and (i!=6 or phase=='sacrifice') else -1000
     if k=='ChooseTarget':return 2400 if a['target'].get('Card')==(source if i==4 and mode=='body' else bear['id'] if bear else None) else 500
     if k=='ChooseAdditionalCost':return 2400 if objs[a['object']['card']]['card']==('Candy Trail' if i==8 else next((v['card']for v in ownbf if v.get('token') and v['token']['subtypes']=='Frog'),'')) else 500
     if k=='ChooseReplacement':return 2400 if (objs[a['source']['card']]['card']=='Doubling Season')==first else 2300
     if k=='ChooseCreatedToken':return 2400 if objs[a['object']['card']]['token']['subtypes']==('Frog' if first else 'Hero') else 2300
     if k=='ChooseSearch':return 2400 if next(v for v in o['search_choice']['cards'] if v['id']==a['card'])['card']=='Alacrian Jaguar' else 2300
     if k=='ChooseOptional':return 2300 if a['accept'] else 1000
     if k=='PlayLand':
      n=objs[a['card']]['card'];count=sum(v['card']==n for v in ownbf);return 1700-50*count
     if k in ['PayCosts','ChooseSpellCost','PutTrigger','Keep','FinishAttackers','FinishBlockers']:return 2200
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2000 if o['decision']=='pay_costs' else -1000
     if k in ['FinishTargets','FinishSearch']:return 1000
     if k in ['ArrangeLibrary','Discard','Bottom']:return 1800
     if k=='Pass':return 0
     return 500
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];k=a['kind']
    if k=='AnnounceCast' and actor==p:
     n=objs[a['card']]['card']
     if phase=='setup' and n==name:source=a['card'];phase='cast'
     elif phase=='effect' and n in ['Snakeskin Veil',"Cartographer's Companion","Bard's Bow"]:done=i!=6 or mode!='body';phase='resolving_effect';job=a['card'] if mode=='job' else None
    if k=='AnnounceActivate' and actor==p:activation=True;done=True;phase='resolving_effect'
    if k=='ChooseReplacement':
     replacements+=1;self.assertEqual(o['replacement_choice']['kind'],'Counters' if mode=='counter' else 'Tokens');self.deny(g,o,a,other);stale=dict(a,source=dict(a['source'],generation=a['source']['generation']+1));self.deny(g,o,stale)
    if k=='ChooseCreatedToken':selected=a['object'];self.assertIn(selected,o['created_token_choice']['candidates']);self.deny(g,o,a,other)
    if k=='ChooseSearch':search=True;self.assertIsNone(g.observe(other)['search_choice'])
    self.replay(g,o,j)
    if phase=='resolving_effect' and not done and quiet and i==6:phase='sacrifice'
    if phase=='resolving_effect' and not done and i==6:
     after=g.observe(p)
     if after['decision']=='priority' and not after['stack'] and not after['announcing'] and not after['resolving']:phase='sacrifice'
   self.fail((name,p,mode,phase,'no complete actual paid witness'))
if __name__=='__main__':unittest.main()
