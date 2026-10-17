"""Complete base-stat bodies, naturally drawn and paid in Standard games."""
import json,os,unittest
from pathlib import Path
from rustmage import Game
from test_role_consumers import RoleConsumerTests
FP=json.loads((Path(__file__).resolve().parents[1]/'data/standard/coverage.json').read_text())['catalogue_version']
CASES=[('Doc Ock, Sinister Scientist',None,True),('Mirkwood Meditator',None,True),('Mirkwood Meditator',None,False),('Multiply by Zero',None,True),('Perfected Theory',0,True),('Perfected Theory',1,True),('Quandrix Charm',0,True),('Quandrix Charm',0,False),('Quandrix Charm',1,True),('Quandrix Charm',2,True),('Snowmelt Stag',None,True),('Turtle-Duck',None,True),('Water Wings',None,True),('Gateway Sneak',None,True),('Snooping Page',None,True)]
BASICS=['Plains','Island','Swamp','Forest']
CREATURES={'Doc Ock, Sinister Scientist','Mirkwood Meditator','Snowmelt Stag','Turtle-Duck','Gateway Sneak','Snooping Page'}
class BaseStatPublicTests(unittest.TestCase):
 frame=RoleConsumerTests.frame
 def test_complete_ten_bodies_all_modes_and_choices_both_seats(self):
  for i,(name,mode,accept) in enumerate(CASES):
   for p in ['P0','P1']:
    with self.subTest(name=name,mode=mode,accept=accept,seat=p):self.scenario(i,p,32000+i)
 def scenario(self,i,p,seed,library=None):
  name,mode,accept=CASES[i];q='P1' if p=='P0' else 'P0';pi=int(p[-1]);qi=1-pi
  own=['Island']*4+['Forest']*8+['Swamp']*8+['Plains']*8+['Azorius Guildgate']*4+sum(([n]*4 for n in [name,'Bear Cub','Common Crook','Opt','Quick Study','Giant Growth','Pacifism']),[])
  enemy=['Forest']*12+['Island']*12+['Plains']*8+sum(([n]*4 for n in ['Bear Cub','Chrome Companion','Quick Study','Opt','Pacifism','Candy Trail','Bygone Colossus']),[])
  self.assertEqual((len(own),len(enemy)),(60,60))
  with Game([own,enemy] if p=='P0' else [enemy,own],seed=seed,starting_player=p,deck_rules='Standard',library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY')) as g:
   stage='setup';source=target=helper=None;frames=paid=0;source_paid=False;turn=drawn=life=None;chosen=False;attack=False;effect_seen=False
   for iteration in range(30000):
    head=g.observe('P0')
    if head['outcome'] is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertTrue(source_paid);self.assertTrue(effect_seen);return dict(name=name,mode=mode,accept=accept,player=p,frames=frames,paid=paid)
    o=g.observe(head['actor']);actor=o['actor'];self.assertEqual((o['schema'],o['catalogue']),(100,FP));objects={v['id']:v for v in o['objects']};field=[v for v in objects.values() if v['zone']=='Battlefield'];mine=[v for v in field if v['controller']==p];foes=[v for v in field if v['controller']==q];hand=[v for v in g.observe(actor)['objects'] if v['zone']=='Hand' and v['owner']==actor]
    ally=next((v for v in mine if v['card']=='Bear Cub'),None);foe=next((v for v in foes if v['card']=='Bear Cub'),None);villain=next((v for v in mine if v['card']=='Common Crook'),None);enchantment=next((v for v in foes if v['card']=='Pacifism'),None);body=objects.get(source)
    quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving'];main=actor==p and o['active']==p and o['step']=='Main1';ready=ally is not None and foe is not None and any(v['zone']=='Hand' and v['owner']==p and v['card']==name for v in g.observe(p)['objects']) and len([v for v in mine if v['card'] in BASICS+['Azorius Guildgate']])>=8
    if body is not None and body['zone']=='Battlefield' and name=='Snowmelt Stag':self.assertEqual((body['power'],body['toughness']),(5,2) if o['active']==p else (2,5));self.assertTrue(body['keywords']&8)
    if body is not None and body['zone']=='Battlefield' and name=='Doc Ock, Sinister Scientist':
     grave=len([v for v in objects.values() if v['zone']=='Graveyard' and v['owner']==p]);self.assertEqual((body['power'],body['toughness']),(8,8) if grave>=8 else (4,5));self.assertEqual(bool(body['keywords']&4096),villain is not None)
    if stage=='wait' and quiet:
     self.assertTrue(source_paid)
     if name in CREATURES and body is not None and body['zone']=='Battlefield':stage={'Doc Ock, Sinister Scientist':'doc','Mirkwood Meditator':'land','Gateway Sneak':'gate','Snooping Page':'feed','Snowmelt Stag':'activate','Turtle-Duck':'activate'}[name]
     elif name not in CREATURES:
      self.assertEqual(body['zone'],'Graveyard')
      if name=='Multiply by Zero':self.assertEqual(objects[target]['zone'],'Graveyard')
      elif name=='Quandrix Charm' and mode==0:self.assertEqual(o['players'][qi]['cards_drawn_this_turn'],drawn+int(accept))
      elif name=='Quandrix Charm' and mode==1:self.assertEqual(objects[target]['zone'],'Graveyard')
      else:
       expected=(1,1) if name=='Perfected Theory' and mode==0 else (4,5) if name=='Perfected Theory' else (5,5) if name=='Quandrix Charm' else (4,4)
       self.assertEqual((objects[target]['power'],objects[target]['toughness']),expected)
       self.assertTrue(any(v['recipient']==objects[target]['reference'] and not v['static_source'] for v in o['base_stat_effects']))
       if name=='Water Wings':self.assertEqual(objects[target]['keywords']&(1|4096),1|4096)
      effect_seen=True;turn=o['turn'];stage='cleanup'
    if stage=='doc' and quiet and villain is not None and len([v for v in objects.values() if v['zone']=='Graveyard' and v['owner']==p])>=8:effect_seen=True;stage='finish'
    if stage=='waitland' and quiet:
     self.assertTrue(chosen);self.assertEqual((body['power'],body['toughness']),(4,2) if accept else (2,4));effect_seen=True;turn=o['turn'];stage='cleanup'
    if stage=='waitactivate' and quiet:
     if name=='Turtle-Duck':self.assertEqual((body['power'],body['toughness']),(4,4));self.assertTrue(body['keywords']&256);effect_seen=True;turn=o['turn'];stage='cleanup'
     else:self.assertTrue(body['keywords']&16384);stage='attack'
    if stage=='waitfeed' and quiet:self.assertTrue(body['keywords']&16384);stage='attack'
    if stage=='waitgate' and quiet:self.assertTrue(body['keywords']&16384);stage='attack'
    if stage=='waitdamage' and quiet and o['step'] in ['CombatDamage','EndCombat','Main2','End','Cleanup']:
     self.assertTrue(attack)
     if name in ['Gateway Sneak','Snooping Page']:
      self.assertEqual(o['players'][pi]['cards_drawn_this_turn'],drawn+1);self.assertEqual(o['players'][pi]['life'],life-int(name=='Snooping Page'))
     effect_seen=True;turn=o['turn'];stage='cleanup'
    if stage=='cleanup' and quiet and o['turn']>turn:
     self.assertFalse([v for v in o['base_stat_effects'] if not v['static_source']])
     if body is not None and body['zone']=='Battlefield' and name in ['Mirkwood Meditator','Turtle-Duck']:self.assertEqual((body['power'],body['toughness']),(2,4) if name=='Mirkwood Meditator' else (0,4))
     if body is not None and body['zone']=='Battlefield':self.assertFalse(body['keywords']&16384)
     stage='finish'
    remaining={a.get('card') for a in o['actions'] if a['kind'] in ['Discard','Bottom']}
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure']:return -2000
     if k=='AnnounceCast':
      if actor==p and stage=='setup' and n==name and ready:
       if name=='Quandrix Charm' and mode==0:return 4100 if any(v['controller']==q and v['card']==helper for v in o['stack']) else -1000
       if name=='Quandrix Charm' and mode==1:return 4100 if enchantment is not None else -1000
       return 4100
      if stage=='finish':return 2600 if n in ['Bear Cub','Common Crook','Chrome Companion','Bygone Colossus'] else -1000
      if n=='Bear Cub':return 2800 if (ally if actor==p else foe) is None else -1000
      if actor==p and stage=='doc' and n=='Common Crook' and villain is None:return 4000
      if actor==p and stage=='doc' and n in ['Opt','Quick Study']:return 3000
      if actor==p and stage=='feed' and n=='Giant Growth' and main and not body['summoning_sick']:return 4000
      if actor==q and stage=='setup' and ready and name=='Quandrix Charm':
       if mode==0 and n=='Opt' and o['active']==q and quiet:return 3900
       if mode==1 and n=='Pacifism' and enchantment is None:return 3900
      return -1000
     if k=='AnnounceActivate':return 4100 if actor==p and stage=='activate' and main and not body['summoning_sick'] and a['card']==source and a['ability']==0 else -1000
     if k=='ChooseMode':return 4000 if a['mode']==mode else -1000
     if k=='ChooseTarget':
      wanted=helper if name=='Quandrix Charm' and mode==0 and actor==p else target if stage=='wait' and actor==p else foe['id'] if stage=='waitfeed' and actor==p and foe is not None else ally['id'] if actor==q and ally is not None else None
      return 4000 if a['target'].get('Spell' if name=='Quandrix Charm' and mode==0 and actor==p else 'Card')==wanted else -1000
     if k=='ChooseOptional':return 4000 if a['accept']==accept else -1000
     if k=='PayResolutionMana':return 4000 if accept else -1000
     if k=='DeclineResolutionMana':return 4000 if not accept else -1000
     if k=='PlayLand':
      if actor==p and name=='Gateway Sneak' and n=='Azorius Guildgate' and stage not in ['gate','finish']:return -1000
      if actor==p and stage=='gate':return 4200 if n=='Azorius Guildgate' and main and not body['summoning_sick'] else -1000
      return 2500-20*sum(v['card']==n for v in field if v['controller']==actor)
     if k in ['AnnounceMana','TapMana','ActivateMana']:return 3000 if o['decision'] in ['pay_costs','resolution_mana'] else -1000
     if k in ['Discard','Bottom']:
      keep={name} if actor==p else set()
      if stage!='finish' and (ally if actor==p else foe) is None:keep.add('Bear Cub')
      if actor==q and stage=='setup' and name=='Quandrix Charm':keep.add('Opt' if mode==0 else 'Pacifism')
      if actor==p and stage!='finish':
       if name=='Gateway Sneak':keep.add('Azorius Guildgate')
       if name=='Snooping Page':keep.add('Giant Growth')
       if name=='Doc Ock, Sinister Scientist':keep.add('Common Crook')
      if stage!='finish' and n in keep and sum(objects[x]['card']==n for x in remaining)<=1:return -1000
      return 2800 if n in ['Pacifism','Common Crook','Giant Growth'] else 1600
     if k=='Attack':return 4200 if actor==p and stage=='attack' and a['card']==source else 4300 if stage=='finish' else -1000
     if k=='Block':return -1000
     return {'Keep':4000,'PayCosts':4000,'ChooseSpellCost':4000,'FinishTargets':3900,'PutTrigger':3800,'FinishSearch':3700,'FinishAttackers':3500,'FinishBlockers':3500,'Pass':0}.get(k,1000)
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision'],a))
    if a['kind']=='AnnounceCast':
     n=objects[a['card']]['card']
     if actor==p and stage=='setup' and n==name:
      source=a['card'];target=(enchantment['id'] if name=='Quandrix Charm' and mode==1 else foe['id'] if name=='Multiply by Zero' else ally['id']);drawn=o['players'][qi]['cards_drawn_this_turn'];stage='wait'
     elif actor==q and stage=='setup' and n=='Opt' and name=='Quandrix Charm' and mode==0:helper=a['card']
     elif actor==p and stage=='feed' and n=='Giant Growth':stage='waitfeed'
    if a['kind']=='AnnounceActivate' and actor==p and a['card']==source:stage='waitactivate'
    if a['kind']=='PlayLand' and actor==p:
     if stage=='land':stage='waitland'
     elif stage=='gate':stage='waitgate'
    if a['kind']=='ChooseOptional' and actor==p and stage=='waitland':chosen=True
    if a['kind']=='Attack' and actor==p and stage=='attack' and a['card']==source:attack=True;drawn=o['players'][pi]['cards_drawn_this_turn'];life=o['players'][pi]['life'];stage='waitdamage'
    if a['kind']=='PayCosts' and actor==p:
     paid+=1
     if o['announcing'] is not None and o['announcing']['card']==source:source_paid=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal'))
if __name__=='__main__':unittest.main()
