"""Paid complete Role bodies in legal sixty-card Standard games, with owned replay."""
import json,unittest
from pathlib import Path
from rustmage import Game,EngineError
NAMES=['Besotted Knight // Betroth the Beast','Conceited Witch // Price of Beauty','Ferocious Werefox // Guard Change','Vantress Transmuter // Croaking Curse','Charmed Clothier','Charming Scoundrel','Embereth Veteran','Living Lectern','Redtooth Genealogist','Royal Treatment','Spiteful Hexmage','Splashy Spellcaster']
ROLES=['Royal','Wicked','Monster','Cursed','Royal','Wicked','Young Hero','Sorcerer','Royal','Royal','Cursed','Sorcerer']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
FACTS={v['name']:v for v in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
class RoleConsumerTests(unittest.TestCase):
 def frame(self,g,o,index):
  actor=o['actor'];other='P1' if actor=='P0' else 'P0';action=o['actions'][index];before=[g.observe(v) for v in ['P0','P1']]
  with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor=other),action)
  self.assertEqual(before,[g.observe(v) for v in ['P0','P1']]);saved=g.snapshot();parent=g.clone();child=parent.clone();parent.close()
  with child:
   child.act(child.observe(actor),action);expected=[child.observe(v) for v in ['P0','P1']]
  g.step(o,index);self.assertEqual(expected,[g.observe(v) for v in ['P0','P1']]);g.restore(saved)
  with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,index)
  g.step(g.observe(actor),index)
  actual=[g.observe(v) for v in ['P0','P1']]
  for a,b in zip(actual,expected):b['revision']=a['revision'];self.assertEqual(a,b)
  g.drop_snapshot(saved)
 def test_twelve_complete_role_consumers_paid_both_seats(self):
  for i,name in enumerate(NAMES):
   for p in ['P0','P1']:
    with self.subTest(name=name,seat=p):self.scenario(i,p,28200+i)
 def test_four_inset_aliases_are_not_deck_cards(self):
  for name in NAMES[:4]:
   with self.assertRaises(EngineError):
    with Game([[FACTS[name]['card_faces'][1]['name']]*60,['Forest']*60],deck_rules='Freeform'):pass
 def scenario(self,i,p,seed,library=None):
  name=NAMES[i];q='P1' if p=='P0' else 'P0';pi=int(p[-1]);deck=['Plains']*6+['Island']*6+['Swamp']*6+['Mountain']*6+['Forest']*8+sum(([n]*4 for n in [name,'Bear Cub','Candy Trail','Quick Study','Bygone Colossus','Chrome Companion','Thrill of Possibility']),[])
  self.assertEqual(len(deck),60)
  with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library) as g:
   stage='setup';source=None;target=None;frames=0;attacked=False;spell_faces=set();role_seen=False
   for iteration in range(30000):
    o=g.observe('P0')
    if o['outcome'] is not None:
     self.assertEqual(stage,'finish',(name,p,seed,stage,iteration));self.assertTrue(role_seen);self.assertTrue(attacked,(name,p,seed,'no actual recipient/normal attack'));return frames
    o=g.observe(o['actor']);actor=o['actor'];self.assertEqual(o['schema'],90);objects={v['id']:v for v in o['objects']};mine=g.observe(p);hand=[v for v in mine['objects'] if v['zone']=='Hand'];own=[v for v in objects.values() if v['controller']==p and v['zone']=='Battlefield'];enemy=[v for v in objects.values() if v['controller']==q and v['zone']=='Battlefield'];ally=next((v for v in own if v['card']=='Bear Cub'),None);foe=next((v for v in enemy if v['card']=='Bear Cub'),None);lands=[v for v in own if v['card'] in BASICS and not v['tapped']];quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
    for d in o['spell_characteristics']:
     if d['object']['card']==source:spell_faces.add(d['face'])
    if quiet and stage in ['wait_role','wait_activation','wait_feed','wait_inset']:
     aura=next((v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p and v['token'] and v['token']['name']==ROLES[i] and v['attached_to']==objects[target]['reference']),None)
     if aura is not None:
      role_seen=True;self.assertTrue(any(v['aura']==aura['reference'] and v['recipient']==objects[target]['reference'] for v in o['role_timestamps']));self.assertEqual(objects[target]['power'],1 if ROLES[i]=='Cursed' else 2 if ROLES[i]=='Young Hero' else 3)
      if i<4:self.assertEqual(objects[source]['zone'],'Exile');self.assertTrue(any(v['object']==objects[source]['reference'] for v in o['adventures']));stage='normal'
      else:
       if i in [6,7]:self.assertEqual(objects[source]['zone'],'Graveyard')
       if i==9:self.assertTrue(objects[target]['keywords']&4096)
       stage='finish'
    if quiet and stage=='wait_body' and objects[source]['zone']=='Battlefield':stage='activate' if i in [6,7] else 'feed'
    if quiet and stage=='wait_normal' and objects[source]['zone']=='Battlefield':self.assertTrue({'Adventure','Normal'}<=spell_faces);stage='finish'
    if o['library_choice'] is not None:self.assertIsNone(g.observe('P1' if actor=='P0' else 'P0')['library_choice'])
    ready=bool(quiet and ally and foe and len(lands)>=8)
    def score(a):
     k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
     if k in ['Concede','Mulligan','Cast','BeginCast','BeginActivate','Activate','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen']:return -2000
     if k=='AnnounceAdventure':return 3000 if actor==p and i<4 and stage=='setup' and n==name and ready else -1000
     if k=='AnnounceExile':return 3000 if actor==p and stage=='normal' and a['card']==source else -1000
     if k=='AnnounceCast':
      if actor==p and stage=='setup' and i>=4 and n==name and ready:return 3000
      if actor==p and stage=='feed' and n=='Quick Study':return 3000
      if stage=='finish':return 2400 if n in ['Bear Cub','Bygone Colossus','Chrome Companion'] else -1000
      if n=='Bear Cub':return 2300 if not (ally if actor==p else foe) else -1000
      return -1000
     if k=='AnnounceActivate':return 3000 if actor==p and stage=='activate' and a['card']==source and a['ability']==0 else -1000
     if k=='ChooseMode':return 3000 if a['mode']==2 else 100
     if k=='ChooseTarget':return 3000 if a['target'].get('Card')==target else -1000
     if k=='PlayLand':return 1800-20*sum(v['controller']==actor and v['zone']=='Battlefield' and v['card']==n for v in objects.values())
     if k in ['TapMana','ActivateMana','AnnounceMana']:return 2100 if o['decision'] in ['pay_costs','resolution_mana'] and n in BASICS else -1000
     if k in ['PayCosts','PutTrigger','FinishTargets','FinishSearch','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost']:return 2400
     if k in ['Discard','Bottom']:
      if actor==p and stage!='finish' and n==name and sum(v['card']==n for v in hand)<=1:return -1000
      if actor==p and stage=='feed' and n=='Quick Study' and sum(v['card']==n for v in hand)<=1:return -1000
      return 2200 if n in ['Candy Trail','Thrill of Possibility','Bygone Colossus','Chrome Companion'] else 1000
     if k=='Attack':return 2800 if stage=='finish' else -1000
     if k=='Pass':return 0
     return 1000
    index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index];self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision']))
    if a['kind'] in ['AnnounceAdventure','AnnounceCast'] and actor==p and stage=='setup' and objects[a['card']]['card']==name:
     source=a['card'];target=(foe if i==3 else ally)['id'];stage='wait_inset' if i<4 else 'wait_body' if i in [6,7,11] else 'wait_role'
    elif a['kind']=='AnnounceExile' and actor==p:stage='wait_normal'
    elif a['kind']=='AnnounceActivate' and actor==p and stage=='activate':stage='wait_activation'
    elif a['kind']=='AnnounceCast' and actor==p and stage=='feed':stage='wait_feed'
    if a['kind']=='Attack' and actor==p and a['card']==(source if i<4 else target):attacked=True
    self.frame(g,o,index);frames+=1
   self.fail((name,p,seed,stage,'no natural terminal game'))
if __name__=='__main__':unittest.main()
