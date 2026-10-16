"""Actual paid Mobilize bodies, attacking entries and ordinary delayed sacrifice."""
import unittest
from rustmage import Game, EngineError
NAMES=['Avenger of the Fallen','Dalkovan Packbeasts','Dragonback Lancer','Nightblade Brigade','Reigning Victor','Shock Brigade','Stadium Headliner',"Zurgo's Vanguard"]
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class MobilizeTests(unittest.TestCase):
 def replay(self,g,o,j):
  saved=g.snapshot();actor=o['actor']
  with g.clone() as f: expected=f.step(f.observe(actor),j)
  g.step(o,j);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,j)
  after=g.step(g.observe(actor),j);expected['revision']=after['revision'];self.assertEqual(after,expected);g.drop_snapshot(saved);return after
 def test_all_eight_paid_bodies_attacking_entries_and_real_end_step_sacrifices(self):
  for i in range(8):
   for p in ['P0','P1']:
    with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p)
 def scenario(self,i,p):
  name=NAMES[i];other='P1' if p=='P0' else 'P0';pi=0 if p=='P0' else 1
  deck=sum(([b]*12 for b in BASICS),[])+[name]*12+['Bear Cub']*24+['Thrill of Possibility']*16
  enemy=sum(([b]*12 for b in BASICS),[])+['Bear Cub']*24
  with Game([deck,enemy] if p=='P0' else [enemy,deck],seed=235+i,starting_player=p,deck_rules='Freeform') as g:
   paid=False;source=None;attacked=False;batch=None;queued=False;night=False;victor=False;baseline=None
   for _ in range(24000):
    o=g.observe('P0');self.assertIsNone(o['outcome'],(name,p));o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};quiet=not o['stack'] and not o['announcing'] and not o['resolving'] and o['decision']=='priority'
    grave=[v for v in objects.values() if v['owner']==p and v['zone']=='Graveyard' and (v['card'] in NAMES+['Bear Cub'] or bool(v.get('token')) and v['token']['subtypes']=='Warrior')]
    if quiet and paid and not attacked:
     body=objects[source]
     if i==3:self.assertTrue(night)
     if i==4:self.assertTrue(victor)
     if i==1:self.assertTrue(body['keywords']&8)
     if i==2:self.assertTrue(body['keywords']&1)
     if i in [0,3]:self.assertTrue(body['keywords']&128)
     if i==5:self.assertTrue(body['keywords']&32768)
    if batch and queued and quiet:
     self.assertFalse(o['delayed_sacrifices'])
     for r in batch['tokens']:
      v=objects.get(r['card']);self.assertTrue(v is None or v['zone']=='Outside' or v['reference']!=r)
     self.assertGreaterEqual(o['creatures_died_this_turn'],len(batch['tokens']));return
    def score(a):
     k=a['kind']
     if k in ['AnnounceCast','Cast','BeginCast']:
      n=objects[a['card']]['card']
      if actor!=p:return -1000
      if n==name:return 1800 if not paid and (i!=0 or grave) and k=='AnnounceCast' else -1000
      if n=='Thrill of Possibility':return 1700 if i==0 and not grave else -1000
      return -1000
     if k=='ChooseTarget':
      t=a['target'].get('Card');return 2000 if t==source else 1000
     if k=='Attack':return 1800 if actor==p and a['card']==source and not attacked else -1000
     if k=='ChooseSpellCost':return 2000-a['cost']
     if k=='PayCosts':return 2000
     if k=='TapMana':return 1900 if o['decision']=='pay_costs' else -1000
     if k=='PlayLand':return 1300-20*sum(v['owner']==actor and v['zone']=='Battlefield' and v['card']==objects[a['card']]['card'] for v in objects.values())
     if k=='ChooseAdditionalCost':return 2100 if objects[a['object']['card']]['card']=='Bear Cub' else 1000
     if k=='Discard':return 2100 if objects[a['card']]['card']=='Bear Cub' else 1000
     if k=='ArrangeLibrary':return 2100 if a['destination']=='Graveyard' else 1000
     return {'Keep':2500,'Mulligan':-10000,'Concede':-10000,'Pass':0,'FinishAttackers':1000,'FinishBlockers':1000,'PutTrigger':2100,'FinishTargets':1000}.get(k,-1000)
    j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,o['decision'],o['actions']))
    ann=o['announcing']
    if a['kind']=='PayCosts' and ann and ann['source_card']==name and ann['kind']=='Spell':paid=True;source=ann['card'];baseline=o['players'][pi]['library_count']
    if a['kind']=='Attack' and a['card']==source:attacked=True
    if a['kind']=='ArrangeLibrary' and paid and i==3:night=True
    before=o;after=self.replay(g,o,j);objs={v['id']:v for v in after['objects']}
    if paid and i==4 and source in objs and objs[source]['zone']=='Battlefield' and objs[source]['keywords']&8192:victor=True
    if not batch and after['delayed_sacrifices']:
     batch=after['delayed_sacrifices'][0];self.assertEqual(batch['source']['card'],source)
     amount=3 if i==1 else len(grave) if i==0 else 1;self.assertEqual(len(batch['tokens']),amount)
     self.assertEqual(after['creatures_attacked_this_turn'],before['creatures_attacked_this_turn'])
     self.assertEqual(after['players'][pi]['nonland_permanents_entered_this_turn'],before['players'][pi]['nonland_permanents_entered_this_turn']+amount)
     for r in batch['tokens']:
      v=objs[r['card']];self.assertEqual(v['reference'],r);self.assertEqual((v['power'],v['toughness']),(1,1));self.assertTrue(v['tapped']);self.assertTrue(v['attacking']);self.assertEqual(v['token']['colors'],8);self.assertEqual(v['token']['subtypes'],'Warrior')
     if i==1:self.assertFalse(objs[source]['tapped'])
     if i==7:self.assertEqual(objs[source]['power'],sum(v['owner']==p and v['zone']=='Battlefield' and (v['card'] in NAMES+['Bear Cub'] or bool(v.get('token')) and v['token']['subtypes']=='Warrior') for v in objs.values()))
    if any(s['kind']=='DelayedSacrifice' for s in after['stack']):
     queued=True;self.assertFalse(after['delayed_sacrifices']);self.assertTrue(all(objs[r['card']]['zone']=='Battlefield' for r in batch['tokens']))
   self.fail(str((name,p,paid,attacked,batch,queued)))
if __name__=='__main__':unittest.main()
