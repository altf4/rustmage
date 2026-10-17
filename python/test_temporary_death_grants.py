"""Whole temporary death spells in paid, naturally terminal Standard games."""
import json,re,unittest
from pathlib import Path
from rustmage import Game
from test_role_consumers import RoleConsumerTests
NAMES=['Not Dead After All','Fake Your Own Death','Undying Malice']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
ROOT=Path(__file__).resolve().parents[1]
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
class TemporaryDeathGrantTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_three_paid_whole_spells_both_seats(self):
  for i in range(3):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,29900+i)
 def scenario(self,i,p,seed,library=None):
  name=NAMES[i];q='P1' if p=='P0' else 'P0';deck=['Plains']*6+['Island']*6+['Swamp']*8+['Mountain']*6+['Forest']*6+sum(([n]*4 for n in [*NAMES,'Bear Cub','Bygone Colossus','Murder','Quick Study']),[])
  self.assertEqual(len(deck),60)
  with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library) as g:
   stage='setup';recipient=None;old=None;source=None;frames=0;paid=False;death=False;attacked=False;returned=False
   for iteration in range(30000):
    head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(104,FP))
    if head['outcome'] is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertTrue(paid and death and returned and attacked,(name,p,paid,death,returned,attacked))
     return dict(frames=frames,card=name,seat=p,outcome=head['outcome'])
    o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    def own(player):return [v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==player]
    def hand(player):return [v for v in g.observe(player)['objects'] if v['zone']=='Hand' and v['owner']==player]
    def available(player):
     mana=list(o['players'][int(player[-1])]['mana'])
     for v in own(player):
      if not v['tapped'] and v['card'] in BASICS:mana[BASICS.index(v['card'])]+=1
     return mana
    bear=next((v for v in own(p) if v['card']=='Bear Cub'),None)
    if recipient is not None and objects[recipient]['zone']=='Graveyard':death=True
    if stage=='wait_grant' and quiet:
     for viewer in ['P0','P1']:self.assertEqual(g.observe(viewer)['temporary_abilities'],o['temporary_abilities'])
     grant=next(v for v in o['temporary_abilities'] if v['recipient']==old)
     self.assertEqual(grant['granting_card'],name);self.assertEqual(grant['grantor_controller'],p);stage='kill'
    if stage=='wait_return' and quiet:
     v=objects[recipient];self.assertEqual(v['zone'],'Battlefield');self.assertNotEqual(v['reference'],old);self.assertEqual((v['owner'],v['controller']),(p,p));self.assertTrue(v['tapped']);self.assertEqual(v['counters']['plus_one'],int(i==2));self.assertEqual(v['power'],2 if i==1 else 3);self.assertFalse(o['temporary_abilities']);returned=True
     if i==0:self.assertTrue(any(x['token'] and x['token']['name']=='Wicked' and x['attached_to']==v['reference'] for x in own(p)))
     if i==1:self.assertTrue(any(x['token'] and x['token']['name']=='Treasure Token' for x in own(p)))
     stage='finish'
    ready=bool(quiet and bear and available(p)[2]>=3 and sum(available(p))>=5 and all(any(v['card']==n for v in hand(p)) for n in [name,'Murder']))
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceActivate','AnnounceAdventure','AnnounceOmen','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceExile']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup' and ready and n==name:return 4000
      if actor==p and stage=='kill' and n=='Murder':return 4000
      if n=='Bear Cub':return 2400 if stage=='finish' or not any(v['card']=='Bear Cub' for v in own(actor)) and stage=='setup' else -1000
      if n=='Bygone Colossus':return 2300 if stage=='finish' else -1000
      if n=='Quick Study':return 2200 if stage=='setup' and quiet and sum(available(actor))>=4 else -1000
      return -1000
     if k=='ChooseTarget':return 3500 if a['target'].get('Card')==recipient else -1000
     if k=='PlayLand':return 1800-10*sum(v['card']==n for v in own(actor))+(25 if n=='Swamp' else 0)
     if k in ['TapMana','ActivateMana','AnnounceMana']:
      if o['decision']!='pay_costs':return -1000
      casting=(o['announcing']or{}).get('source_card','')
      needed=2 if casting=='Murder' else 1 if casting in NAMES else 0
      # Preserve black lands for the following Murder by paying the grant's
      # generic requirement from another color when available.
      if n=='Swamp':return 2300 if o['players'][int(actor[-1])]['mana'][2]<needed else 1700
      return 2200
     if k in ['PayCosts','PutTrigger','FinishTargets','Keep','ChooseSpellCost']:return 3000
     if k in ['FinishAttackers','FinishBlockers']:return 2400
     if k=='Attack':return 2800 if stage=='finish' else -1000
     if k in ['Discard','Bottom']:return -500 if actor==p and stage!='finish' and n in [name,'Murder'] and sum(v['card']==n for v in hand(actor))<=1 else 1500
     if k=='Pass':return 0
     return 1000
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,stage,o['decision']))
    if a['kind']=='AnnounceCast' and actor==p:
     if stage=='setup' and objects[a['card']]['card']==name:recipient=bear['id'];old=bear['reference'];source=a['card'];stage='wait_grant'
     elif stage=='kill' and objects[a['card']]['card']=='Murder':stage='wait_return'
    if a['kind']=='PayCosts' and (o['announcing']or{}).get('card')==source:
     payment=a['payment'];self.assertEqual(sum(payment),2 if i==1 else 1);self.assertGreaterEqual(payment[2],1);paid=True
    if a['kind']=='Attack' and actor==p and a['card']==recipient:attacked=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal game'))
if __name__=='__main__':unittest.main()
