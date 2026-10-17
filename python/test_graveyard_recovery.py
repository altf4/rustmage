"""Paid recovery bodies in legal sixty-card Standard games, owned rollback at every frame."""
import json
from pathlib import Path
import unittest
from rustmage import Game, EngineError
import test_adventure

FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
NAMES=["Archenemy's Charm",'Auroral Procession','Barkform Harvester','Coati Scavenger','Dundoolin Weaver','Elvish Regrower','Fateshaper Aspirant','Fight On!',"Hazel's Nocturne",'Macabre Waltz','Oltec Archaeologists','Pull from the Grave','Restore with Empathy',"Rydia's Return",'Tomb Trawler','Tombstone, Career Criminal','True Ancestry','White Widow, Free Agent']
BASICS=['Plains','Island','Swamp','Mountain','Forest']

class GraveyardRecoveryTests(unittest.TestCase):
    def replay(self,g,o,index):
        actor=o['actor'];other='P1' if actor=='P0' else 'P0'
        saved=g.snapshot();parent=g.clone();child=parent.clone();parent.close()
        with child:
            expected=child.act(child.observe(actor),o['actions'][index])
            other_expected=child.observe(other)
        self.assertEqual(g.step(o,index),expected)
        g.restore(saved)
        with self.assertRaisesRegex(EngineError,'StaleRevision'):g.step(o,index)
        actual=g.step(g.observe(actor),index);expected['revision']=actual['revision']
        self.assertEqual(actual,expected)
        other_actual=g.observe(other);other_expected['revision']=other_actual['revision']
        self.assertEqual(other_actual,other_expected);g.drop_snapshot(saved)
        return actual
    denied=test_adventure.AdventureTests.denied

    def test_eighteen_paid_complete_recovery_bodies_both_seats(self):
        for i,name in enumerate(NAMES):
            for p in ['P0','P1']:
                with self.subTest(card=name,seat=p):self.scenario(i,p,2850+i)

    def scenario(self,i,p,seed,library=None):
        name=NAMES[i];q='P1' if p=='P0' else 'P0';pi=int(p[-1])
        target_name='Tombstone, Career Criminal' if name=='Fateshaper Aspirant' else 'Three Tree Mascot'
        lands=['Plains']*8+['Island']*8+['Swamp']*8+['Mountain']*6+['Forest']*6
        deck=lands+sum(([n]*4 for n in [name,target_name,'Thrill of Possibility','Bear Cub','Bygone Colossus','Shock']),[])
        self.assertEqual(len(deck),60)
        with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library) as g:
            stage='setup';source=None;chosen=None;reference=None;frames=0;cast_turn=None;activated=False;paid=False
            for iteration in range(30000):
                o=g.observe('P0')
                if o['outcome'] is not None:
                    self.assertEqual(stage,'finish',(name,p,seed,stage,iteration,[(v['card'],v['zone']) for v in g.observe(p)['objects'] if v['owner']==p and v['zone'] in ['Hand','Battlefield','Graveyard']]))
                    self.assertTrue(paid,(name,p,seed));return frames
                o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p)
                own=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield'];hand=[v for v in mine['objects'] if v['zone']=='Hand']
                graves=[v for v in objects.values() if v['owner']==p and v['zone']=='Graveyard']
                dead=next((v for v in graves if v['card']==target_name),None)
                quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
                if quiet and stage=='wait_body':
                    if name in ['Barkform Harvester','Tomb Trawler']:
                        self.assertEqual(objects[source]['zone'],'Battlefield');stage='activate'
                    else:stage='check'
                if quiet and stage=='wait_activation':stage='check'
                if quiet and stage=='check':
                    if name in ['Barkform Harvester','Tomb Trawler']:
                        move=o['last_target_library_move'];self.assertIsNotNone(move)
                        self.assertEqual((move['target_before'],move['destination']),(reference,'Bottom'))
                        self.assertEqual(move['target_after'],dict(reference,generation=reference['generation']+1));self.assertTrue(activated)
                    else:
                        recovered=next(v for v in g.observe(p)['objects'] if v['id']==chosen)
                        self.assertEqual(recovered['zone'],'Hand',(name,p,seed))
                        self.assertEqual(recovered['reference'],dict(reference,generation=reference['generation']+1))
                    if name=='True Ancestry':self.assertTrue(any(v.get('token') and v['token']['subtypes']=='Clue' for v in own))
                    stage='finish'
                have_lands=[v for v in own if v['card'] in BASICS and not v['tapped']]
                creature_count=sum('Creature' in FACTS.get(v['card'],{}).get('type_line','').split(' — ')[0] for v in own)
                permanent_graves=sum(any(k in FACTS.get(v['card'],{}).get('type_line','').split(' — ')[0].split() for k in ['Creature','Land','Artifact','Enchantment','Planeswalker']) for v in graves)
                ready=bool(dead and len(have_lands)>=8 and (name!='Coati Scavenger' or permanent_graves>=4) and (name!='Dundoolin Weaver' or creature_count>=3))
                def score(a):
                    k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
                    if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceWarp','AnnounceExile','AnnounceFlashback','AnnouncePlotted','AnnouncePlot']:return -10000
                    if k=='AnnounceCast':
                        if actor==p and stage=='setup' and n==name and ready:return 3000
                        if stage=='finish':return 2500 if n in ['Bear Cub','Bygone Colossus',target_name] else -1000
                        if actor==p and stage=='setup' and n=='Thrill of Possibility' and (not dead or name=='Coati Scavenger' and permanent_graves<4) and any(v['card']==target_name or name=='Coati Scavenger' and v['card'] in BASICS for v in hand):return 2400
                        if actor==p and n=='Bear Cub' and name=='Dundoolin Weaver' and creature_count<3:return 2300
                        return -1000
                    if k=='AnnounceActivate':return 3000 if actor==p and stage=='activate' and a['card']==source else -1000
                    if k=='ChooseMode':return 2700 if a['mode']==(1 if name in ["Archenemy's Charm","Rydia's Return",'White Widow, Free Agent'] else 0) else -1000
                    if k=='ChooseTarget':return 2700 if a['target']=={'Card':chosen} else -1000
                    if k=='ChooseAdditionalCost':
                        selected=objects[a['object']['card']]['card']
                        return 3000 if selected==target_name and not dead else 2400 if selected in BASICS else -1000
                    if k=='Discard':
                        if stage=='setup':return 2700 if n==target_name and not dead else 2200 if n in BASICS else -10000 if n==name and sum(v['card']==name for v in hand)==1 else -500
                        return 2700 if a['card']!=chosen else -500
                    if k=='PlayLand':return 2300-sum(v['card']==n for v in own)*4 if actor==p else 2200
                    if k=='TapMana':return 2200 if o['decision']=='pay_costs' else -1000
                    if k=='PayCosts':return 2800
                    if k=='Attack':return 1400 if stage=='finish' else -1000
                    if k=='FinishAttackers':return 800
                    if k in ['Block','AssignBlock','BeginBlockers']:return -1000
                    if k=='ArrangeLibrary':return 2400 if a['destination']=='Top' else 2300
                    if k in ['ActivateMana','AnnounceMana','AnnounceAdventure','AnnounceOmen']:return -1000
                    return {'Keep':3000,'PutTrigger':2500,'FinishTargets':1800,'Pass':0}.get(k,100)
                if stage=='setup' and ready and chosen is None:chosen=dead['id'];reference=dead['reference']
                index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));action=o['actions'][index]
                if action['kind']=='AnnounceCast' and actor==p and objects[action['card']]['card']==name and stage=='setup':
                    source=action['card'];chosen=dead['id'];reference=dead['reference'];stage='wait_body';cast_turn=o['turn']
                if action['kind']=='AnnounceActivate' and action['card']==source:stage='wait_activation';activated=True
                if action['kind']=='PayCosts' and actor==p and o['announcing'] and o['announcing'].get('source_card')==name:paid=True
                wrong=dict(o,actor=q if actor==p else p);self.denied(g,wrong,action)
                self.replay(g,o,index);frames+=1
            self.fail((name,p,seed,stage,iteration,cast_turn,g.observe(p)))

if __name__=='__main__':unittest.main()
