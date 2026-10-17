"""Eight paid complete Adventure pairs in ordinary sixty-card Standard games."""
import json
from pathlib import Path
import unittest
from rustmage import Game, EngineError
import test_adventure

NAMES = ['Beanstalk Wurm // Plant Beans', 'Beorn, Reluctant Host // Till and Tend', 'Bellowing Bruiser // Beat a Path', "Beluna's Gatekeeper // Entry Denied", 'Bofur, Reliable Guardian // Concerted Care', 'Ishgard, the Holy See // Faith & Grief', 'Velvetwing Butterflies // Gaze in Wonder', 'Woodland Acolyte // Mend the Wilds']
BASICS = ['Plains','Island','Mountain','Forest']
FACTS = {c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}

class AdventureLandTargetTests(unittest.TestCase):
    replay = test_adventure.AdventureTests.replay
    denied = test_adventure.AdventureTests.denied
    def test_eight_paid_complete_pairs_both_seats(self):
        for i in range(8):
            for p in ['P0','P1']:
                with self.subTest(card=NAMES[i],seat=p):self.scenario(i,p,2770+i)
    def test_all_eight_inset_aliases_are_not_independent_cards(self):
        for n in NAMES:
            with self.assertRaises(EngineError):
                with Game([[FACTS[n]['card_faces'][1]['name']]*60,['Forest']*60],deck_rules='Freeform'):pass
    def scenario(self,i,p,seed,library=None):
        q='P1' if p=='P0' else 'P0';pi=int(p[-1]);name=NAMES[i]
        deck=['Plains']*10+['Island']*8+['Forest']*8+['Mountain']*6+sum(([n]*4 for n in [name,'Bear Cub','Candy Trail','Shock','Thrill of Possibility','Bygone Colossus','Chrome Companion']),[])
        self.assertEqual(len(deck),60)
        with Game([deck,deck],seed=seed,starting_player=p,deck_rules='Standard',library=library) as g:
            stage='setup';source=None;target=None;descriptors=set();frames=0;attacked=False;land_witness=False;played_turn=None;base=None
            for iteration in range(30000):
                o=g.observe('P0')
                if o['outcome'] is not None:
                    self.assertEqual(stage,'finish',(name,p,seed,stage,iteration))
                    self.assertTrue(land_witness if i==5 else attacked,(name,p,seed))
                    self.assertTrue({'Adventure','Normal'}<=descriptors if i!=5 else 'Adventure' in descriptors)
                    return frames
                o=g.observe(o['actor']);actor=o['actor'];objects={v['id']:v for v in o['objects']};mine=g.observe(p)
                hand=[v for v in mine['objects'] if v['zone']=='Hand']
                own=[v for v in objects.values() if v['owner']==p and v['zone']=='Battlefield'];enemy=[v for v in objects.values() if v['owner']==q and v['zone']=='Battlefield']
                ally=next((v for v in own if v['card']=='Bear Cub'),None);foe=next((v for v in enemy if v['card']=='Bear Cub'),None)
                dead=next((v for v in objects.values() if v['owner']==p and v['zone']=='Graveyard' and v['card']=='Candy Trail'),None)
                candy=next((v for v in own if v['card']=='Candy Trail'),None)
                lands=[v for v in own if v['card'] in BASICS and not v['tapped']]
                quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
                for d in o['spell_characteristics']:
                    if d['object']['card']==source:
                        descriptors.add(d['face']);face=FACTS[name]['card_faces'][1 if d['face']=='Adventure' else 0]
                        self.assertEqual((d['physical_card'],d['name'],d['mana_cost'],d['type_line']),(name,face['name'],face['mana_cost'],face['type_line']))
                if quiet and stage=='wait_inset':
                    self.assertEqual(objects[source]['zone'],'Exile');self.assertTrue(any(x['object']==objects[source]['reference'] for x in o['adventures']))
                    if i in [0,1]:self.assertEqual(o['players'][pi]['extra_land_plays_remaining'],1);stage='extra_land'
                    else:
                        if i==2:self.assertTrue(objects[target]['keywords']&2048)
                        if i==3:self.assertEqual(objects[target]['zone'],'Hand')
                        if i==4:self.assertEqual(objects[target]['keywords']&(4096|8192),4096|8192)
                        if i==5:self.assertEqual(objects[target]['zone'],'Hand')
                        if i==6:self.assertTrue(objects[target]['tapped'])
                        if i==7:self.assertEqual(o['last_target_library_move']['target_after'],objects[target]['reference']);self.assertEqual(o['last_target_library_move']['destination'],'Top')
                        stage='normal'
                if quiet and stage=='wait_extra':
                    self.assertEqual(o['players'][pi]['extra_land_plays_remaining'],0);stage='normal'
                if quiet and stage=='wait_normal':
                    self.assertEqual(objects[source]['zone'],'Battlefield');self.assertFalse(any(x['object']['card']==source for x in o['adventures']))
                    if i==5:self.assertTrue(objects[source]['tapped'])
                    else:
                        face=FACTS[name]['card_faces'][0];self.assertEqual((objects[source]['power'],objects[source]['toughness']),(int(face['power']),int(face['toughness'])))
                    if i==7:self.assertEqual(objects[target]['zone'],'Hand')
                    stage='finish'
                ready=bool(ally and foe and len(lands)>=8 and (dead if i in [5,7] else True) and (played_turn==o['turn'] and any(v['card'] in BASICS for v in hand) if i in [0,1] else True))
                def score(a):
                    k=a['kind'];n=objects.get(a.get('card'),{}).get('card','')
                    if k in ['Concede','Mulligan','Cast','BeginCast','AnnounceWarp','AnnounceFlashback','AnnouncePlot']:return -2000
                    if k=='AnnounceAdventure':return 2800 if actor==p and stage=='setup' and n==name and ready else -1000
                    if k=='AnnounceExile':return 2800 if actor==p and stage=='normal' and a['card']==source else -1000
                    if k=='AnnounceCast':
                        if stage=='finish':return 2300 if n in ['Bear Cub','Bygone Colossus','Chrome Companion',name] and i!=5 else 2300 if n in ['Bear Cub','Bygone Colossus','Chrome Companion'] else -1000
                        if n=='Bear Cub':return 2200 if not (ally if actor==p else foe) else -1000
                        if actor==p and n=='Candy Trail' and i in [5,7] and not dead and not candy:return 2300
                        return -1000
                    if k in ['AnnounceActivate','Activate','BeginActivate']:return 2700 if actor==p and n=='Candy Trail' and i in [5,7] and not dead else -1000
                    if k=='PlayLand':
                        if n==name:return 2800 if actor==p and stage=='normal' and a['card']==source else -1000
                        if actor==p and stage=='extra_land':return 2900
                        if actor==p and stage=='setup' and i in [0,1] and len(lands)>=8 and sum(v['card'] in BASICS for v in hand)<2:return -1000
                        return 1800-20*sum(v['owner']==actor and v['zone']=='Battlefield' and v['card']==n for v in objects.values())
                    if k=='ChooseTarget':return 2900 if a['target'].get('Card')==target else -1000
                    if k in ['TapMana','ActivateMana','AnnounceMana']:
                        if a.get('card')==source and i==5 and stage=='finish':return 2800
                        return 2100 if o['decision'] in ['pay_costs','resolution_mana'] and n in BASICS else -1000
                    if k in ['PayCosts','PutTrigger','FinishTargets','FinishSearch','FinishAttackers','FinishBlockers','Keep','ChooseSpellCost']:return 2400
                    if k in ['Discard','Bottom']:
                        if actor==p and stage!='finish' and n==name and sum(v['card']==n for v in hand)<=1:return -1000
                        return 2200 if n in ['Shock','Thrill of Possibility','Bygone Colossus','Chrome Companion'] else 1000
                    if k=='Attack':return 2800 if stage=='finish' else -1000
                    if k=='Pass':return 0
                    return 1000
                j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));a=o['actions'][j];self.assertGreaterEqual(score(a),0,(name,p,stage,o['decision']))
                if a['kind']=='AnnounceAdventure':
                    source=a['card'];target=(foe if i in [2,3,6] else ally if i==4 else dead)['id'] if i not in [0,1] else None;stage='wait_inset'
                if a['kind']=='PlayLand' and actor==p:
                    played_turn=o['turn']
                    if stage=='extra_land':stage='wait_extra'
                    elif a['card']==source:stage='wait_normal'
                if a['kind']=='AnnounceExile' and actor==p:stage='wait_normal'
                if a['kind']=='Attack' and a['card']==source:attacked=True
                if a['kind'] in ['TapMana','ActivateMana'] and a['card']==source:land_witness=True
                before=[g.observe(v) for v in ['P0','P1']]
                with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor='P1' if actor=='P0' else 'P0'),a)
                self.assertEqual(before,[g.observe(v) for v in ['P0','P1']]);self.replay(g,o,j);frames+=1
            self.fail((name,p,seed,stage,'no terminal game'))
