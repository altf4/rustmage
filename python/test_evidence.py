"""Paid collect-evidence casts, activations, reflexive observers and opponent ward."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError,Game
NAMES=['Analyze the Pollen','Axebane Ferox','Bite Down on Crime','Crimestopper Sprite','Evidence Examiner','Extract a Confession','Forensic Researcher','Sample Collector','Surveillance Monitor','Vitu-Ghazi Inspector']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
class EvidenceTests(unittest.TestCase):
 def replay(self,g,o,j):
  saved=g.snapshot()
  with g.clone() as f:
   expected=f.step(f.observe(o['actor']),j);other='P1' if o['actor']=='P0' else 'P0';expected_other=f.observe(other)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(o['actor']),j);expected['revision']=after['revision'];self.assertEqual(after,expected);visible=g.observe(other);expected_other['revision']=visible['revision'];self.assertEqual(visible,expected_other);g.drop_snapshot(saved);return after
 def reject(self,g,o,a,actor=None):
  before=[g.observe(p) for p in ('P0','P1')]
  with self.assertRaises(EngineError):g.act(dict(o,actor=actor or o['actor']),a)
  self.assertEqual([g.observe(p) for p in ('P0','P1')],before)
 def test_all_ten_bodies_paid_public_casts_declarations_and_real_remaining_abilities(self):
  for i,name in enumerate(NAMES):
   for p in ('P0','P1'):
    for pay in (False,True):
     with self.subTest(card=name,seat=p,collect=pay):self.scenario(i,p,pay)
 def test_real_zero_value_extra_cards_remain_selectable_after_reaching_threshold(self):
  for p in ('P0','P1'):
   for i in (0,2):
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,True,zero_extra=True)
 def test_actual_ward_evidence_after_mandatory_activated_evidence_payment(self):
  for p in ('P0','P1'):
   for pay in (False,True):
    with self.subTest(seat=p,pay=pay):self.scenario(6,p,pay,ward_ability=True)
 def scenario(self,i,p,pay,zero_extra=False,ward_ability=False):
  name=NAMES[i];other='P1' if p=='P0' else 'P0';owner=int(p[1]);enemy=1-owner
  deck=sum(([b]*12 for b in BASICS),[])+[name]*16+['Hamlet Glutton']*16+['Thrill of Possibility']*16+['Bear Cub']*16
  if zero_extra:deck+=['Evolving Wilds']*16
  foe=['Forest']*24+['Mountain']*24+['Bear Cub']*16+['Hamlet Glutton']*16+['Thrill of Possibility']*16+['Shock']*16
  if ward_ability:foe+=['Axebane Ferox']*16
  with Game([deck,foe] if p=='P0' else [foe,deck],seed=43 if i==1 and p=='P1' else 67,starting_player=p,deck_rules='Freeform') as g:
   paid=False;source=None;choice_done=False;search_seen=False;untap_paid=False;untap_done=False;tap_paid=False;target_tapped=None;ward_seen=False;sacrificed=None;frames=0;casts=0;zero_used=False;ward_ability_seen=False
   for _ in range(24000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p,pay));o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};ann=o['announcing'];root=o['resolving']
    bf=lambda who,n:[v for v in objects.values() if v['owner']==who and v['zone']=='Battlefield' and v['card']==n]
    friends=bf(p,'Bear Cub');foes=bf(other,'Axebane Ferox' if ward_ability else 'Bear Cub');bigfoes=bf(other,'Hamlet Glutton');body=next(iter(bf(p,name)),None)
    mana=o['players'][owner]['mana'][:]
    for v in objects.values():
     if v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS and not v['tapped']:mana[BASICS.index(v['card'])]+=1
    required=[0]*6;generic=0
    for symbol in FACTS[name]['mana_cost'].replace('}','').split('{')[1:]:
     if symbol.isdigit():generic+=int(symbol)
     else:required['WUBRGC'.index(symbol)]+=1
    if i==2 and pay:generic-=2
    enough=all(x>=y for x,y in zip(mana,required)) and sum(mana)>=sum(required)+generic
    grave=lambda who:sum(FACTS[v['card']]['cmc'] for v in objects.values() if v['owner']==who and v['zone']=='Graveyard' and v['card'] in FACTS)
    threshold=8 if i==0 else 6 if i in (2,3,5,9) else 4 if i in (1,4,8) else 3
    if ward_ability:threshold=14
    ready=enough and (i==1 or not (pay or i in (4,6,7,8)) or grave(p)>=threshold) and (i not in (2,3,7,9) or friends) and (i not in (2,5,6) or foes) and (i!=5 or bigfoes)
    if o['decision']=='evidence_intent':
     self.assertEqual(ann['source_card'],name);self.assertIsNone(ann['cast']['evidence']);self.assertEqual(ann['targets'],[]);self.assertIsNone(o['payment_request']);self.assertEqual(g.observe(other)['actions'],[])
     self.reject(g,o,{'kind':'ChooseCollectEvidence','collect':pay},other);self.reject(g,o,{'kind':'ChooseTarget','target':{'Player':other}})
    e=o['evidence']
    if e:
     frames+=1;self.assertEqual(actor,e['player']);self.assertIsNone(g.observe('P1' if actor=='P0' else 'P0')['evidence']);self.assertEqual(e['total_mana'],sum(FACTS[objects[r['card']]['card']]['cmc'] for r in e['selected']))
     for r in e['selected']:self.assertEqual(objects[r['card']]['reference'],r);self.assertEqual(objects[r['card']]['zone'],'Graveyard')
     self.reject(g,o,{'kind':'ChooseEvidence','object':{'card':4294967295,'generation':0}})
     if e['candidates']:
      stale=dict(e['candidates'][0]);stale['generation']+=1;self.reject(g,o,{'kind':'ChooseEvidence','object':stale})
     if e['total_mana']<e['threshold']:self.reject(g,o,{'kind':'FinishEvidence'})
     if e['selected']:self.reject(g,o,{'kind':'ChooseEvidence','object':e['selected'][0]})
     if i==1 and root and root['source_card']==name:ward_seen=True;self.assertEqual(actor,other)
     if ward_ability and e['optional'] and root and root['source_card']=='Axebane Ferox':ward_ability_seen=True;self.assertEqual(actor,p)
    if o['search_choice'] and root and root['source_card']==name:
     search_seen=True;self.assertIsNone(g.observe(other)['search_choice']);self.assertEqual(o['search_choice']['destination'],'Hand');self.assertTrue(o['search_choice']['reveal'])
    if paid and body:
     if i==1 and ward_seen and not o['stack'] and not o['resolving'] and not o['announcing']:
      self.assertEqual(body['damage'],2 if pay else 0);self.assertEqual((body['power'],body['toughness']),(4,4));self.assertTrue(body['keywords']&4);self.assertTrue(body['keywords']&128);return
     if i==3 and not o['stack'] and not root and not ann and friends and friends[0]['tapped']:
      self.assertEqual(friends[0]['counters']['stun'],int(pay));return
     if i in (4,8) and choice_done and not o['stack'] and not root and not ann:
      subtype='Clue' if i==4 else 'Thopter';tokens=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield' and v['token'] and subtype+' Token'==v['token']['name']];self.assertEqual(len(tokens),int(pay))
      if tokens and i==8:self.assertEqual((tokens[0]['power'],tokens[0]['toughness']),(1,1));self.assertTrue(tokens[0]['keywords']&1)
      return
     if i==6 and tap_paid and not o['stack'] and not root and not ann:
      self.assertTrue(untap_done);self.assertEqual(foes[0]['tapped'],pay if ward_ability else True);self.assertTrue(body['tapped']);self.assertEqual(o['last_evidence']['threshold'],4 if ward_ability and pay else 3);self.assertEqual(ward_ability_seen,ward_ability);return
     if i==7 and choice_done and not o['stack'] and not root and not ann:
      self.assertEqual(friends[0]['counters']['plus_one'],int(pay));return
     if i==9 and not o['stack'] and not root and not ann:
      self.assertEqual(friends[0]['counters']['plus_one'],int(pay));self.assertEqual(o['players'][owner]['life'],20+2*int(pay));return
    if paid and source and objects[source['card']]['zone']=='Graveyard' and not o['stack'] and not root and not ann:
     if i==0:
      self.assertTrue(search_seen);self.assertTrue(o['known_hand']);
      if zero_extra:self.assertTrue(zero_used)
      return
     if i==2:
      self.assertEqual(friends[0]['power'],4);self.assertTrue(any(v['card']=='Bear Cub' and v['owner']==other and v['zone']=='Graveyard' for v in objects.values()));
      if zero_extra:self.assertTrue(zero_used)
      return
     if i==5:
      self.assertIsNotNone(sacrificed);v=objects[sacrificed['card']];self.assertEqual(v['zone'],'Graveyard');self.assertEqual(v['card'],'Hamlet Glutton' if pay else 'Bear Cub');return
    if i==6 and untap_paid and not untap_done and target_tapped:
     v=objects[target_tapped['card']]
     if v['reference']==target_tapped and not v['tapped']:untap_done=True
    def score(a):
     k=a['kind']
     if k in ('AnnounceCast','Cast'):
      n=objects[a['card']]['card']
      if actor==other:
       if n=='Axebane Ferox':return 950 if ward_ability and not foes else -1000
       if n=='Bear Cub':return 900 if not bf(other,'Bear Cub') else -1000
       if n=='Hamlet Glutton':return 950 if i==5 and not bigfoes else -1000
       if n=='Thrill of Possibility':return 1100 if i==1 and grave(other)<4 and any(v['owner']==other and v['zone']=='Hand' and v['card']=='Hamlet Glutton' for v in objects.values()) else -1000
       if n=='Shock':return 1300 if i==1 and body and not ward_seen and grave(other)>=4 else -1000
       return -1000
      if n==name:return 1200 if ready and not paid and k=='AnnounceCast' else -1000
      if n=='Bear Cub':return 850 if i in (2,3,7,9) and not friends else -1000
      if n=='Thrill of Possibility':return 1100 if (pay or i in (4,6,7,8)) and grave(p)<threshold and any(v['owner']==p and v['zone']=='Hand' and v['card']=='Hamlet Glutton' for v in objects.values()) else -1000
      return -1000
     if k=='ChooseCollectEvidence':return 1800 if a['collect']==pay else -1000
     if k=='ChooseEvidence':
      if zero_extra and e['total_mana']>=e['threshold'] and FACTS[objects[a['object']['card']]['card']]['cmc']==0 and not any(FACTS[objects[r['card']]['card']]['cmc']==0 for r in e['selected']):return 2100
      return 1600+int(FACTS[objects[a['object']['card']]['card']]['cmc']) if pay or i==6 and not e['optional'] else -1000
     if k=='FinishEvidence':return 1900
     if k=='DeclineEvidence':return 2000 if not pay and e['optional'] else -1000
     if k=='ChooseAdditionalCost':return 1800 if objects[a['object']['card']]['card']=='Hamlet Glutton' else 1000
     if k=='ChooseTarget':
      t=a['target']
      if 'Player' in t:return 1700 if t['Player']==other else 0
      if 'Card' not in t:return -1000
      v=objects[t['Card']]
      if ann and ann['source_card']=='Shock':return 1800 if v['owner']==p and v['card']==name else -1000
      if ann and ann['source_card']=='Forensic Researcher':
       return 1800 if (ann['kind']=={'ActivatedAbility':{'ability':0}} and v['owner']==p and v['card'] in BASICS and v['tapped']) or (ann['kind']=={'ActivatedAbility':{'ability':1}} and v in foes) else -1000
      return 1800 if v in friends else 1600 if v in foes else -1000
     if k in ('AnnounceActivate','BeginActivate','Activate'):
      if zero_extra and objects[a['card']]['card']=='Evolving Wilds':return 1550
      if i!=6 or objects[a['card']]['card']!=name:return -1000
      return 1500 if a['ability']==(1 if untap_done else 0) and (untap_done or any(v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS and v['tapped'] for v in objects.values())) else -1000
     if k=='TapMana':
      if o['decision']=='pay_costs':return 1100
      return 1300 if i==6 and actor==p and body and not untap_done and not any(v['owner']==p and v['zone']=='Battlefield' and v['card'] in BASICS and v['tapped'] for v in objects.values()) else -1000
     if k=='Attack':return 1400 if i==7 and body and a['card']==body['id'] and not choice_done else -1000
     if k=='Sacrifice':return 1800 if objects[a['object']['card']]['card']==('Hamlet Glutton' if pay else 'Bear Cub') else 1000
     if k=='ChooseSearch':return 1500
     if k=='PlayLand':
      n=objects[a['card']]['card'];count=sum(v['owner']==actor and v['zone']=='Battlefield' and v['card']==n for v in objects.values());return 1500 if n=='Evolving Wilds' else 1400-10*count
     if k=='Discard':return 1700 if objects[a['card']]['card']=='Hamlet Glutton' else 1500 if objects[a['card']]['card'] in BASICS else 100
     return {'Concede':-10000,'Mulligan':-10000,'ChooseBargain':1800,'Keep':2000,'PayCosts':1800,'PutTrigger':1700,'FinishTargets':1400,'FinishSearch':1300,'FinishAttackers':700,'FinishBlockers':700,'Pass':0}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j]
    if score(a)<0:self.fail('unhandled public decision '+str((name,p,pay,o['decision'],ann,o['actions'])))
    if a['kind']=='ChooseEvidence' and e['total_mana']>=e['threshold'] and FACTS[objects[a['object']['card']]['card']]['cmc']==0:zero_used=True
    if a['kind']=='PayCosts' and ann and ann['source_card']==name and ann['kind']=='Spell':
     paid=True;source=ann['object'];casts+=1
     self.assertEqual(ann['cast']['evidence'],pay if i in (0,2,3,5,9) else None)
    if a['kind']=='PayCosts' and ann and ann['source_card']==name and i==6:
     if ann['kind']=={'ActivatedAbility':{'ability':0}}:
      untap_paid=True;target_tapped=ann['targets'][0]['object']
     if ann['kind']=={'ActivatedAbility':{'ability':1}}:tap_paid=True
    if a['kind'] in ('FinishEvidence','DeclineEvidence') and root and root['source_card']==name:choice_done=True
    if a['kind']=='Sacrifice' and root and root['source_card']==name:sacrificed=a['object']
    self.replay(g,o,j)
   self.fail('no full evidence continuation '+str((name,p,pay,frames,casts)))
if __name__=='__main__':unittest.main()
