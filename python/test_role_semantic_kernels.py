"""Whole semantic Role bodies, real payments/timing and naturally terminal Standard games."""
import json
import re
import unittest
from pathlib import Path
from rustmage import Game
import test_role_consumers as prior

NAMES=['Return Triumphant','Twisted Sewer-Witch','Syr Armont, the Redeemer','Asinine Antics','A Tale for the Ages','Mystical Tether']
BASICS=['Plains','Island','Swamp','Mountain','Forest']
ROOT=Path(__file__).resolve().parents[1]
FACTS={v['name']:v for v in json.loads((ROOT/'data/standard/cards.json').read_text())['cards']}
FP=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']

class RoleSemanticTests(unittest.TestCase):
    frame=prior.RoleConsumerTests.frame

    def test_six_paid_complete_bodies_both_seats(self):
        for i,name in enumerate(NAMES):
            for p in ['P0','P1']:
                with self.subTest(card=name,seat=p):self.scenario(i,p,29500+i)

    def test_paid_timing_cards_keep_cheaper_normal_cast_route_both_seats(self):
        for i in [3,5]:
            for p in ['P0','P1']:
                with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,29550+i,flash=False)

    def scenario(self,i,p,seed,library=None,flash=True):
        name=NAMES[i];q='P1' if p=='P0' else 'P0';pi=int(p[-1])
        support=['Bear Cub','Three Tree Mascot','Royal Treatment','Murder','Disenchant','Quick Study']
        deck=['Plains']*7+['Island']*7+['Swamp']*6+['Mountain']*5+['Forest']*7+sum(([n]*4 for n in [name,*support]),[])
        self.assertEqual(len(deck),60)
        with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library) as g:
            stage='setup';source=None;recipient=None;foe_id=None;frames=0;paid=False;seen=False;attacked=False;pre_enchanted=False;spell_started=False;death_seen=False;returned_ref=None;young_seen=False;wicked=None;wicked_life=None
            for iteration in range(35000):
                head=g.observe('P0');self.assertEqual((head['schema'],head['catalogue']),(93,FP))
                if head['outcome'] is not None:
                    self.assertEqual(stage,'finish',(name,p,stage,iteration));self.assertTrue(paid);self.assertTrue(seen);self.assertTrue(attacked,(name,p,head['turn'],head['outcome'],recipient))
                    if i==0:self.assertTrue(young_seen)
                    return frames
                o=g.observe(head['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p)
                own=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==p]
                enemy=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==q]
                ally=next((v for v in own if v['card']=='Bear Cub'),None)
                mascot=next((v for v in own if v['card']=='Three Tree Mascot'),None)
                foe=next((v for v in enemy if v['card']=='Bear Cub'),None)
                body=objects.get(source);quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
                if i==0 and seen and objects.get(recipient,{}).get('counters',{}).get('plus_one',0)>0:young_seen=True
                def hand(player):return [v for v in g.observe(player)['objects'] if v['zone']=='Hand' and v['owner']==player]
                def available(player):
                    mana=list(o['players'][int(player[-1])]['mana'])
                    for v in objects.values():
                        if v['zone']=='Battlefield' and v['controller']==player and not v['tapped'] and v['card'] in BASICS:mana[BASICS.index(v['card'])]+=1
                    return mana
                def can_pay(card,player,extra=0):
                    mana=available(player);tokens=re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[card].get('mana_cost',''))
                    colored=[tokens.count(c) for c in 'WUBRGC'];generic=sum(int(c) for c in tokens if c.isdecimal())+extra
                    return all(a>=b for a,b in zip(mana,colored)) and sum(mana)>=sum(colored)+generic
                def role(kind,target,controller=p):
                    return next((v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==controller and v['token'] and v['token']['name']==kind and v['attached_to']==objects[target]['reference']),None)
                ready=bool(quiet and ally and mascot and foe and sum(available(p))>=8 and any(v['card']==name for v in hand(p)) and can_pay(name,p,2 if flash and i in [3,5] else 0))
                if i==0 and stage=='wait_death' and objects[recipient]['zone']=='Graveyard':
                    returned_ref=objects[recipient]['reference'];death_seen=True;stage='cast'
                if i==4 and stage=='wait_enchant' and quiet and role('Royal',recipient):pre_enchanted=True;stage='cast'
                if i==3 and stage=='wait_hexproof' and quiet and objects[foe_id]['keywords']&4096:
                    self.assertTrue(role('Royal',foe_id,q));pre_enchanted=True;stage='cast'
                if stage=='wait_source' and quiet and body is not None:
                    if i==0 and role('Young Hero',recipient):
                        self.assertTrue(death_seen);self.assertNotEqual(objects[recipient]['reference'],returned_ref);self.assertEqual(objects[recipient]['power'],2);seen=True;stage='finish'
                    elif i==1 and body['zone']=='Battlefield':
                        rats=[v for v in own if v['card']=='Three Tree Mascot' or v['token'] and v['token']['subtypes']=='Rat']
                        self.assertEqual(len(rats),2)
                        self.assertTrue(all(role('Wicked',v['id']) for v in rats));self.assertFalse(role('Wicked',recipient));seen=True;wicked=role('Wicked',mascot['id'])['id'];stage='break_wicked'
                    elif i==2 and role('Monster',recipient):
                        self.assertEqual(objects[recipient]['power'],4);self.assertEqual(objects[source]['power'],int(FACTS[name]['power']));seen=True;stage='finish'
                    elif i==3 and role('Cursed',foe_id):
                        self.assertEqual(objects[foe_id]['power'],2 if pre_enchanted else 1)
                        self.assertFalse(role('Cursed',recipient));seen=True;stage='finish'
                    elif i==4 and body['zone']=='Battlefield':
                        self.assertTrue(pre_enchanted);self.assertEqual(objects[recipient]['power'],5);self.assertEqual(objects[mascot['id']]['power'],2);seen=True;stage='finish'
                    elif i==5 and body['zone']=='Battlefield' and objects[foe_id]['zone']=='Exile':stage='remove_tether'
                if i==5 and stage=='wait_return' and quiet and objects[foe_id]['zone']=='Battlefield':
                    self.assertEqual(objects[foe_id]['controller'],q);self.assertEqual(body['zone'],'Graveyard');seen=True;stage='finish'
                if i==1 and stage=='wait_wicked' and quiet:
                    self.assertNotIn(wicked,objects);self.assertEqual(o['players'][1-pi]['life'],wicked_life-1);stage='finish'
                def score(a):
                    k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
                    if k in ['Concede','Mulligan','Cast','BeginCast','Activate','BeginActivate','AnnounceWarp','AnnounceOmen','AnnounceAdventure','AnnounceExile','AnnounceFlashback','AnnouncePlot']:return -2000
                    if k in ['AnnounceCast','AnnounceFlashSurcharge']:
                        if actor==p and n==name and not spell_started:
                            timing=not flash or i not in [3,5] or o['active']==q
                            return 4000 if timing and (stage=='cast' or stage=='setup' and ready and i not in [0,4] and not(i==3 and flash)) and k==('AnnounceFlashSurcharge' if flash and i in [3,5] else 'AnnounceCast') else -1000
                        if k=='AnnounceFlashSurcharge':return -1000
                        if i==0 and actor==q and n=='Murder' and (stage=='setup' and ready or stage=='wait_death') and recipient is not None:return 4100
                        if i==4 and actor==p and n=='Royal Treatment' and stage=='setup' and ready:return 4100
                        if i==3 and flash and actor==q and o['active']==q and n=='Royal Treatment' and stage=='setup' and ready:return 4100
                        if i==5 and actor==p and n=='Disenchant' and stage=='remove_tether':return 4100
                        if i==1 and actor==p and n=='Disenchant' and stage=='break_wicked':return 4100
                        if n in ['Bear Cub','Three Tree Mascot']:
                            existing=next((v for v in (own if actor==p else enemy) if v['card']==n),None)
                            return 2500 if stage=='finish' or existing is None and stage=='setup' else -1000
                        if n=='Quick Study':return 2200 if quiet and stage=='setup' and sum(available(actor))>=4 else -1000
                        return -1000
                    if k=='ChooseTarget':
                        wanted=recipient
                        source_name=(o['announcing']or{}).get('source_card','')
                        if source_name=='Disenchant':wanted=wicked if i==1 else source
                        elif source_name=='Royal Treatment' and i==3:wanted=foe_id
                        elif i==5 and stage=='wait_source':wanted=foe_id
                        return 3500 if a['target'].get('Card')==wanted else -1000
                    if k=='PlayLand':return 1800-20*sum(v['controller']==actor and v['zone']=='Battlefield' and v['card']==n for v in objects.values())
                    if k in ['TapMana','ActivateMana','AnnounceMana']:return 2100 if o['decision'] in ['pay_costs','resolution_mana'] else -1000
                    if k in ['FinishAttackers','FinishBlockers']:return 2400
                    if k in ['PayCosts','PutTrigger','FinishTargets','FinishSearch','Keep','ChooseSpellCost']:return 3000
                    if k in ['Discard','Bottom']:
                        protected=([name]+(['Royal Treatment'] if i==4 else ['Disenchant'] if i in [1,5] else [])) if actor==p else (['Murder'] if i==0 else ['Royal Treatment'] if i==3 and flash else [])
                        return -500 if stage!='finish' and n in protected and sum(v['card']==n for v in hand(actor))<=1 else 1500
                    if k=='Attack':return 2800 if stage=='finish' else -1000
                    if k=='Pass':return 0
                    return 1000
                if stage=='setup' and ready:
                    recipient=ally['id'];foe_id=foe['id']
                index=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][index]
                self.assertGreaterEqual(score(a),0,(name,p,seed,stage,o['decision'],o['actions'],o['announcing'],available(actor)))
                if a['kind'] in ['AnnounceCast','AnnounceFlashSurcharge']:
                    card=objects[a['card']]['card']
                    if actor==p and card==name:
                        if i==3 and flash:self.assertTrue(objects[foe_id]['keywords']&4096)
                        source=a['card'];spell_started=True;stage='wait_source'
                    elif i==0 and actor==q and card=='Murder' and stage=='setup':stage='wait_death'
                    elif i==4 and card=='Royal Treatment' and stage=='setup':stage='wait_enchant'
                    elif i==3 and card=='Royal Treatment' and stage=='setup':stage='wait_hexproof'
                    elif i==5 and card=='Disenchant' and stage=='remove_tether':stage='wait_return'
                    elif i==1 and card=='Disenchant' and stage=='break_wicked':wicked_life=o['players'][1-pi]['life'];stage='wait_wicked'
                if a['kind']=='PayCosts' and o['announcing'] and o['announcing']['card']==source:
                    self.assertEqual(sum(a['payment']),sum(int(t) if t.isdecimal() else 1 for t in re.findall(r'\{([0-9WUBRGC]+)\}',FACTS[name]['mana_cost']))+(2 if flash and i in [3,5] else 0))
                    self.assertEqual(o['announcing']['cast']['flash_surcharge'],flash and i in [3,5]);paid=True
                    if flash and i in [3,5]:self.assertEqual(o['active'],q)
                if a['kind']=='Attack' and actor==p and a['card']==recipient:attacked=True
                self.frame(g,o,index);frames+=1
            self.fail((name,p,seed,stage,'no natural terminal game'))

if __name__=='__main__':unittest.main()
