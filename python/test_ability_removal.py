"""Naturally drawn, paid direct Rust Auras in Standard, with owned action replay."""
import json, os, unittest
from pathlib import Path
from rustmage import Game, OBSERVATION_SCHEMA
from test_role_consumers import RoleConsumerTests
FACTS=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())
CASES=['Fresh Start','Frozen in Ice','Stop Cold',"Enchanted River's Grasp"]
BASICS=['Plains','Island','Swamp','Mountain','Forest']
class AbilityRemovalPublicTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_four_complete_direct_auras_paid_in_both_seats(self):
  for i in range(4):
   for p in ['P0','P1']:
    with self.subTest(card=CASES[i],seat=p):self.scenario(i,p,33700+i)
 def scenario(self,i,p,seed,library=None):
  name=CASES[i];q='P1'if p=='P0'else'P0'
  lands=['Forest']*10+['Island']*12+['Plains']*10
  own=lands+sum(([n]*4 for n in [name,'Llanowar Elves','Forensic Researcher','Quick Study','Opt','Bear Cub','Bygone Colossus']),[])
  enemy=lands+sum(([n]*4 for n in ['Bear Cub','April, Reporter of the Weird','Quick Study','Opt','Candy Trail','Bygone Colossus','Pacifism']),[])
  with Game([own,enemy]if p=='P0'else[enemy,own],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   stage='setup';source=target=None;paid=False;seen=False;frames=0
   for iteration in range(30000):
    head=g.observe('P0')
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertTrue(paid);self.assertTrue(seen)
     return dict(name=name,player=p,frames=frames,paid=True)
    o=g.observe(head['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(OBSERVATION_SCHEMA,FACTS['catalogue_version']))
    objects={v['id']:v for v in o['objects']};field=[v for v in objects.values()if v['zone']=='Battlefield'];mine=[v for v in field if v['controller']==p];quiet=o['decision']=='priority'and not o['stack']and not o['announcing']and not o['resolving'];elf=next((v for v in mine if v['card']=='Llanowar Elves'),None)
    if stage=='wait_aura'and quiet:
     body=objects.get(source);recipient=objects.get(target)
     if body is not None and body['zone']=='Battlefield':
      self.assertEqual(body['attached_to'],recipient['reference']);self.assertFalse(recipient['abilities']['printed']);self.assertEqual((recipient['power'],recipient['toughness']),(-4 if name=='Fresh Start'else 1,1))
      if name!='Fresh Start':self.assertTrue(recipient['tapped'])
      ownview=g.observe(p);self.assertFalse(any(a['kind']in ['TapMana','AnnounceActivate']and a.get('card')==target for a in ownview['actions']))
      seen=True;stage='finish'
    ready=elf is not None and len([v for v in mine if v['card']in BASICS])>=6
    discarded={a.get('card')for a in o['actions']if a['kind']in ['Discard','Bottom']}
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure','AnnounceActivate']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and n==name and ready:return 4200
      current=[v for v in field if v['controller']==actor]
      if n=='Llanowar Elves':return 2900 if actor==p and stage=='setup'and elf is None else -1000
      if n in ['Bear Cub','April, Reporter of the Weird','Bygone Colossus']:return 2800 if stage=='finish'else -1000
      if n in ['Quick Study','Opt']:return 2700 if stage=='setup'else -1000
      return -1000
     if k=='ChooseTarget':return 4300 if a['target']=={'Card':target}else -1000
     if k=='PlayLand':return 2600-20*sum(v['card']==n and v['controller']==actor for v in field)
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3000 if o['decision']in ['pay_costs','resolution_mana']else -1000
     if k in ['Discard','Bottom']:
      keep={name}if actor==p and stage=='setup'else set()
      if actor==p and stage=='setup'and elf is None:keep.add('Llanowar Elves')
      if n in keep and sum(objects[x]['card']==n for x in discarded)<=1:return -1000
      return 2800 if n not in BASICS else 1600
     if k=='Attack':return 3000 if stage=='finish'else -1000
     if k=='Block':return -1000
     return {'Keep':4000,'PayCosts':4000,'ChooseSpellCost':4000,'FinishTargets':3900,'PutTrigger':3800,'FinishAttackers':3500,'FinishBlockers':3500,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision'],a))
    if a['kind']=='AnnounceCast'and actor==p and objects[a['card']]['card']==name:source=a['card'];target=elf['id'];stage='wait_aura'
    if a['kind']=='PayCosts'and actor==p and o['announcing']is not None and o['announcing']['card']==source:paid=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
