"""Complete paid temporary death-exile bodies in naturally drawn Standard games."""
import json,os,unittest
from pathlib import Path
from rustmage import Game
from test_role_consumers import RoleConsumerTests
FP=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
ROWS=[('Feed the Flames',None),('Bot Bashing Time',None),("Elspeth's Smite",None),("Narset's Rebuke",None),('Obliterating Bolt',None),('Scorching Dragonfire',None),('Essence Burn',None),('Combustion Technique',None),('Suplex',0),('Suplex',1),('Agate Assault',0),('Agate Assault',1),('Pinecone Strike',0),('Pinecone Strike',1),('Pinecone Strike',2)]
class DeathExileTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_all_eleven_complete_bodies_and_all_modes_both_seats(self):
  for i,(name,mode) in enumerate(ROWS):
   for seat in ['P0','P1']:
    with self.subTest(name=name,mode=mode,seat=seat):self.scenario(i,seat,31300+i)
 def scenario(self,i,p,seed,library=None):
  name,mode=ROWS[i];q='P1' if p=='P0' else 'P0';damage=mode!=1;artifact=mode==1 or mode==2
  deck=['Mountain']*8+['Island']*6+['Forest']*6+['Plains']*4+sum(([n]*4 for n in [name,'Bear Cub','Bygone Colossus','Quick Study','Candy Trail','Plundering Pirate','Chrome Companion','Disenchant','Llanowar Elves']),[])
  enemy=['Forest']*8+['Island']*8+['Mountain']*8+sum(([n]*4 for n in ['Bear Cub','Bygone Colossus','Quick Study','Candy Trail','Plundering Pirate','Chrome Companion','Ankle Biter','Basilisk Collar','Llanowar Elves']),[])
  self.assertEqual((len(deck),len(enemy)),(60,60))
  with Game([deck,enemy]if p=='P0'else[enemy,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY'))as g:
   stage='setup';source=None;subject=None;relic=None;paid=False;frames=0;mana_paid=None
   for iteration in range(30000):
    head=g.observe('P0')
    if head['outcome']is not None:
     self.assertEqual(stage,'finish',(name,p,mode,seed,stage,iteration));self.assertTrue(paid);return dict(name=name,mode=mode,seat=p,seed=seed,frames=frames)
    o=g.observe(head['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(95,FP));objects={v['id']:v for v in o['objects']};quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    def own(player):return [v for v in objects.values()if v['zone']=='Battlefield' and v['controller']==player]
    def hand(player):return [v for v in g.observe(player)['objects']if v['zone']=='Hand' and v['owner']==player]
    def available(player):
     mana=list(o['players'][int(player[-1])]['mana'])
     for v in own(player):
      if v['card']in BASICS and not v['tapped']:mana[BASICS.index(v['card'])]+=1
     return mana
    bear=next((v for v in own(q)if v['card']=='Bear Cub'),None)
    treasure=next((v for v in own(q)if v['token']and 'Treasure'in v['token']['name']),None)
    decoration=treasure if name=='Pinecone Strike'else next((v for v in own(q)if v['card']=='Candy Trail'),None)
    budget=available(p);cost=5 if name=="Narset's Rebuke"else 4 if name in ['Feed the Flames','Bot Bashing Time']else 3 if name=='Agate Assault'else 1 if name=="Elspeth's Smite"else 2
    ready=any(v['card']==name for v in hand(p))and sum(budget)>=cost and budget[0 if name=="Elspeth's Smite"else 3]>=1 and (not damage or bear is not None)and(not artifact or decoration is not None)
    if stage=='wait'and quiet:
     self.assertTrue(paid);self.assertEqual(objects[source]['zone'],'Graveyard');self.assertFalse(o['death_exile_replacements'])
     if damage:self.assertEqual(objects[subject]['zone'],'Exile',(name,p,mode,objects[subject]))
     if artifact:
      if name=='Pinecone Strike':self.assertNotIn(relic,objects)
      else:self.assertEqual(objects[relic]['zone'],'Exile')
     if name=="Narset's Rebuke":
      expected=mana_paid[:]
      for j in [0,1,3]:expected[j]+=1
      self.assertEqual(o['players'][int(p[-1])]['mana'],expected)
     stage='finish'
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceActivate','AnnounceAdventure','AnnounceOmen','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceExile']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup'and ready and n==name:return 4000
      if actor==q and stage=='setup':
       if damage and n=='Bear Cub'and bear is None:return 2600
       if artifact and n==('Plundering Pirate'if name=='Pinecone Strike'else'Candy Trail')and decoration is None:return 2600
      if actor==p and stage=='finish'and n in ['Bygone Colossus','Bear Cub','Chrome Companion','Plundering Pirate','Llanowar Elves']:return 2400
      if actor==p and stage=='setup'and n=='Quick Study'and quiet and sum(available(p))>=4:return 2200
      return -1000
     if k=='ChooseMode':return 3600 if a['mode']==mode else -1000
     if k=='ChooseTarget':return 3600 if a['target'].get('Card')in [subject,relic]else -1000
     if k=='PlayLand':return 1800-20*sum(v['card']==n for v in own(actor))+(25 if n=='Plains'and name=="Elspeth's Smite"and actor==p else 0)
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3000 if o['decision']in ['pay_costs','resolution_mana']else -1000
     if k in ['PayCosts','PutTrigger','FinishTargets','Keep','ChooseSpellCost']:return 3200
     if k=='Attack':return 3000 if stage=='finish'and actor==p or stage=='setup'and name=="Elspeth's Smite"and actor==q and ready and a['card']==bear['id']else -1000
     if k in ['FinishAttackers','FinishBlockers']:return 2500
     if k in ['Discard','Bottom']:return -500 if actor==p and stage=='setup'and n==name and sum(v['card']==name for v in hand(p))<=1 else 1500
     if k=='Pass':return 0
     return 1000
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,mode,seed,stage,o['decision']))
    if a['kind']=='AnnounceCast'and actor==p and stage=='setup'and objects[a['card']]['card']==name:
     source=a['card'];subject=bear['id']if damage else None;relic=decoration['id']if artifact else None;stage='wait'
    if a['kind']=='PayCosts'and(o['announcing']or{}).get('card')==source:
     payment=a['payment'];self.assertEqual(sum(payment),cost);self.assertGreaterEqual(payment[0 if name=="Elspeth's Smite"else 3],1);paid=True;mana_paid=[v-payment[j]for j,v in enumerate(o['players'][int(p[-1])]['mana'])]
    self.frame(g,o,index);frames+=1
   self.fail((name,p,mode,seed,stage,'no natural terminal game'))
if __name__=='__main__':unittest.main()
