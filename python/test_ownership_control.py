"""Real Korvold theft in legal Standard games, using only observed legal actions."""
import json
from pathlib import Path
import unittest
from rustmage import Game, EngineError
import test_adventure

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
SAGA = 'Korvold and the Noble Thief'
BLOOM = 'Bloomvine Regent // Claim Territory'
MIDGAR = 'Midgar, City of Mako // Reactor Raid'
GIANT = "Grabby Giant // That's Mine"
HOLLOW = 'Hollow Scavenger // Bakery Raid'
BEANS = ['Beanstalk Wurm // Plant Beans', 'Beorn, Reluctant Host // Till and Tend']
PAIRS = [GIANT, BLOOM, MIDGAR, *BEANS, 'Bellowing Bruiser // Beat a Path', 'Ratcatcher Trainee // Pest Problem', HOLLOW]
BASICS = ['Forest','Mountain','Swamp','Island']

class OwnershipControlTests(unittest.TestCase):
    replay = test_adventure.AdventureTests.replay
    denied = test_adventure.AdventureTests.denied

    def test_paid_foreign_adventure_and_normal_control_both_seats(self):
        for player in ['P0','P1']:
            with self.subTest(player=player): self.scenario(player, 2840, mode='Adventure')

    def test_paid_foreign_omen_search_and_owner_shuffle_both_seats(self):
        for player in ['P0','P1']:
            with self.subTest(player=player): self.scenario(player, 2843 if player == 'P0' else 2841, mode='Omen')

    def scenario(self, player, seed, mode='auto', library=None, coverage=None):
        opponent = 'P1' if player=='P0' else 'P0'
        pi = int(player[-1]); qi = 1-pi
        basics = ['Forest']*10 + ['Mountain']*8 + ['Swamp']*5 + ['Island']*5
        own = basics + sum(([name]*4 for name in [SAGA,'Bear Cub','Candy Trail','Whirlwing Stormbrood // Dynamic Soar','Bygone Colossus','Chrome Companion','Thrill of Possibility','Shock']), [])
        enemy = basics + sum(([name]*4 for name in PAIRS), [])
        self.assertEqual((len(own),len(enemy)),(60,60))
        with Game([own,enemy] if player=='P0' else [enemy,own],seed=seed,starting_player=player,deck_rules='Standard',library=library) as g:
            stage='setup'; source=None; name=None; route=None; base=None
            frames=0; attacked=False; mana_witness=False; activated=False
            chapters={}; paid_sagas=set(); private=0; distributions=0
            for iteration in range(30000):
                head=g.observe('P0')
                self.assertEqual((head['schema'],head['catalogue']),(96,EXPECTED))
                if head['outcome'] is not None:
                    self.assertEqual(stage,'finish',(player,seed,mode,stage,name,iteration))
                    self.assertTrue(any({1,2,3} <= seen for seen in chapters.values()),chapters)
                    self.assertTrue(route=='Omen' or (mana_witness if name==MIDGAR or name in BASICS else attacked),(player,seed,name,route))
                    return dict(frames=frames,player=player,name=name,route=route,private=private,distributions=distributions)
                o=g.observe(head['actor']); actor=o['actor']; objects={v['id']:v for v in o['objects']}
                mine=g.observe(player); hand=[v for v in mine['objects'] if v['zone']=='Hand' and v['owner']==player]
                field=[v for v in objects.values() if v['zone']=='Battlefield']
                ours=[v for v in field if v['controller']==player]
                theirs=[v for v in field if v['controller']==opponent]
                quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
                for v in objects.values():
                    if v['card']==SAGA and v['id'] in paid_sagas and v['zone']=='Battlefield':
                        chapters.setdefault(v['id'],set()).add(v['counters']['lore'])
                body=objects.get(source)
                if quiet and stage=='wait_inset':
                    if route=='Omen':
                        self.assertIsNone(body)
                        self.assertEqual(o['players'][pi]['library_count'],base['library']-2)
                        self.assertEqual(o['players'][qi]['library_count'],base['owner_library']+1)
                        self.assertFalse(any(p['object']['card']==source for p in o['adventures']))
                        self.assertEqual(distributions,1)
                        stage='finish'
                    else:
                        self.assertEqual((body['zone'],body['owner'],body['controller']),('Exile',opponent,opponent))
                        permission=next(p for p in o['adventures'] if p['object']==body['reference'])
                        self.assertEqual(permission['player'],player)
                        self.assertFalse(any(p['object']['card']==source for p in o['exile_permissions']))
                        self.denied(g,o,dict(kind='AnnounceAdventure',card=source))
                        if name==MIDGAR:
                            self.assertEqual(o['players'][pi]['library_count'],base['library']-2)
                        if name in BEANS:
                            self.assertEqual(o['players'][pi]['extra_land_plays_remaining'],1)
                            stage='extra_land'
                        else: stage='normal'
                if quiet and stage=='extra_land' and o['players'][pi]['extra_land_plays_remaining']==0: stage='normal'
                if quiet and stage=='wait_normal':
                    self.assertEqual((body['zone'],body['owner'],body['controller']),('Battlefield',opponent,player))
                    self.assertFalse(any(p['object']['card']==source for p in o['adventures']))
                    stage='finish'
                distribution=o.get('search_distribution')
                if distribution:
                    self.assertEqual(distribution,g.observe(opponent if actor==player else player)['search_distribution'])
                    self.assertEqual(len(distribution['candidates']),2)
                    self.assertTrue(all(v['owner']==player for v in distribution['candidates']))
                if o.get('search_choice'):
                    self.assertIsNone(g.observe(opponent if actor==player else player)['search_choice']);private+=1
                eligible={r['object']['card'] for r in mine['exile_permissions'] if r['player']==player and r['expires_turn']>=o['turn']}
                candidates=[]
                if actor==player and stage=='setup' and len([v for v in ours if v['card'] in BASICS])>=8:
                    for action in o['actions']:
                        card=objects.get(action.get('card'),{})
                        if card.get('id') not in eligible or card.get('owner')!=opponent: continue
                        if card.get('card') in BASICS:
                            if mode=='auto' and action['kind']=='PlayLand':candidates.append(action)
                            continue
                        if card.get('card') not in PAIRS:continue
                        wanted='Omen' if card['card']==BLOOM else 'Adventure'
                        if mode!='auto' and wanted!=mode: continue
                        if action['kind']=='Announce'+wanted: candidates.append(action)
                preferred=None
                if candidates:
                    insets=[a for a in candidates if a['kind']!='PlayLand']
                    if insets:candidates=insets
                    preferred=min(candidates,key=lambda a:(coverage or {}).get((objects[a['card']]['card'],'Land' if a['kind']=='PlayLand' else 'Omen' if objects[a['card']]['card']==BLOOM else 'Adventure',player),0))
                finishing = stage=='finish' and (name!=MIDGAR and name not in BASICS or mana_witness)
                def score(a):
                    k=a['kind']; card=objects.get(a.get('card'),{}); n=card.get('card','')
                    if preferred is not None and a==preferred:return 3100
                    if k in ['Concede','Mulligan','Cast','BeginCast','AnnounceWarp','AnnounceFlashback','AnnouncePlot','AnnounceOmen','AnnounceAdventure']:return -2000
                    if k=='AnnounceExile':return 3000 if actor==player and stage=='normal' and a['card']==source else -1000
                    if k=='AnnounceCast':
                        if n==SAGA:return 2600 if actor==player and stage=='setup' and len([v for v in ours if v['card'] in BASICS])>=8 else -1000
                        if n=='Candy Trail':return 2300 if actor==player and not any(v['card']==n for v in ours) and stage=='setup' else -1000
                        if n=='Bear Cub':return 2200 if actor==player and (finishing or not any(v['card']==n for v in ours)) else -1000
                        if actor==opponent and n in PAIRS and n!=MIDGAR:return 2200 if finishing or sum(bool(v['kinds']&1) for v in theirs)<2 else -1000
                        if finishing and n in ['Bygone Colossus','Chrome Companion','Whirlwing Stormbrood // Dynamic Soar']:return 2200
                        return -1000
                    if k=='PlayLand':
                        if card.get('owner')!=actor:return 3000 if actor==player and stage=='normal' and a['card']==source else -1000
                        if actor==player and stage=='extra_land':return 2900
                        if actor==player and stage=='setup' and len([v for v in ours if v['card'] in BASICS])>=8 and len([v for v in hand if v['card'] in BASICS])<=2:return -1000
                        return 1800-30*sum(v['controller']==actor and v['card']==n for v in field)
                    if k=='ChooseTarget':
                        t=a['target'].get('Card');target=objects.get(t,{})
                        return 2800 if a['target'].get('Player')==opponent or target.get('controller')==opponent else 1000
                    if k=='ChooseSearch':return 2900
                    if k=='ChooseSearchBattlefield':return 2900
                    if k in ['Sacrifice','ChooseAdditionalCost']:
                        target=objects.get(a['object']['card'],{})
                        return 2900 if target.get('card')=='Candy Trail' or target.get('token') else 2000
                    if k in ['AnnounceActivate','Activate','BeginActivate']:
                        return 2800 if actor==player and a.get('card')==source and name in [GIANT,HOLLOW] and stage=='finish' and not activated else -1000
                    if k in ['TapMana','ActivateMana','AnnounceMana']:
                        if actor==player and a.get('card')==source and (name==MIDGAR or name in BASICS) and stage=='finish':return 2900
                        return 2500 if o['decision'] in ['pay_costs','resolution_mana','additional_cost'] else -1000
                    if k in ['Discard','Bottom']:
                        if actor==player and stage=='setup' and n==SAGA:return -500
                        if actor==player and n in BASICS and len([v for v in hand if v['card'] in BASICS])<=2:return -400
                        return 2300 if n in ['Shock','Thrill of Possibility','Whirlwing Stormbrood // Dynamic Soar'] else 1500
                    if k=='Attack':
                        if not finishing:return -1000
                        if route=='Omen' or attacked or mana_witness:return 2800
                        # Give the newly cast foreign creature its actual combat
                        # witness before an established army can end the game.
                        return 2900 if actor==player and a['card']==source else -1000
                    return {'Keep':3500,'PayCosts':3000,'ChooseSpellCost':2400,'PutTrigger':2400,'FinishTargets':2300,'FinishSearch':2300,'FinishAttackers':2400,'FinishBlockers':2400,'DeclineEffectPayment':500,'Pass':0}.get(k,1000)
                j=max(range(len(o['actions'])),key=lambda j:score(o['actions'][j]));action=o['actions'][j]
                self.assertGreaterEqual(score(action),0,(player,seed,mode,stage,name,o['decision'],o['actions']))
                ann=o['announcing']
                if action==preferred and preferred is not None:
                    source=action['card'];name=objects[source]['card'];route='Land' if action['kind']=='PlayLand' else action['kind'].removeprefix('Announce')
                    base=dict(library=o['players'][pi]['library_count'],owner_library=o['players'][qi]['library_count'])
                    stage='wait_inset'
                if action['kind']=='AnnounceExile' and actor==player:stage='wait_normal'
                if action['kind']=='PlayLand' and actor==player and action['card']==source:stage='wait_normal'
                if action['kind']=='PayCosts' and ann and ann['kind']=='Spell' and ann['source_card']==SAGA:paid_sagas.add(ann['card'])
                if action['kind']=='Attack' and action['card']==source:attacked=True
                if action['kind']=='ChooseSearchBattlefield':distributions+=1
                activation_payment=action['kind']=='PayCosts' and ann and ann['card']==source and ann['kind']!='Spell'
                if action['kind']=='TapMana' and action['card']==source:mana_witness=True
                before=[g.observe(v) for v in ['P0','P1']]
                with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor='P1' if actor=='P0' else 'P0'),action)
                self.assertEqual(before,[g.observe(v) for v in ['P0','P1']])
                self.replay(g,o,j);frames+=1
                if activation_payment:
                    activated=True
                    if name==MIDGAR:mana_witness=True
            self.fail((player,seed,mode,stage,name,'no natural terminal result'))

if __name__=='__main__':unittest.main()
