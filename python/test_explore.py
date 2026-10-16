"""Explore through paid public-library continuations and owned replay."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError, Game

FACTS = {c['name']: c for c in json.loads((Path(__file__).resolve().parents[1] / 'data/standard/cards.json').read_text())['cards']}
BASICS = ['Plains', 'Island', 'Swamp', 'Mountain', 'Forest']
ENTER_EXPLORERS = ['Cenote Scout', "Kinjalli's Dawnrunner", 'Pathfinding Axejaw', 'River Herald Guide', 'River Herald Scout']


class ExploreTests(unittest.TestCase):
    def replay(self, game, obs, index):
        saved = game.snapshot()
        with game.clone() as fork:
            expected = fork.step(fork.observe(obs['actor']), index)
            other = 'P1' if obs['actor'] == 'P0' else 'P0'
            expected_other = fork.observe(other)
        game.step(obs, index)
        game.restore(saved)
        with self.assertRaisesRegex(EngineError, 'StaleRevision'):
            game.step(obs, index)
        actual = game.step(game.observe(obs['actor']), index)
        expected['revision'] = actual['revision']
        self.assertEqual(actual, expected)
        observed_other = game.observe(other)
        expected_other['revision'] = observed_other['revision']
        self.assertEqual(observed_other, expected_other)
        game.drop_snapshot(saved)
        return actual

    def choice(self, game, obs, counter_added=True):
        choice = obs['explore_choice']
        actor = obs['actor']
        other = 'P1' if actor == 'P0' else 'P0'
        self.assertEqual(choice, game.observe(other)['explore_choice'])
        self.assertEqual(game.observe(other)['actions'], [])
        revealed = choice['revealed']
        self.assertNotIn('Land', FACTS[revealed['card']]['type_line'].split(' — ')[0])
        self.assertEqual(obs['known_library_top'][int(actor[1])], revealed)
        self.assertEqual(choice['explorer']['controller'], actor)
        self.assertEqual(choice['counter_added'], counter_added)
        if counter_added:
            explorer = next(v for v in obs['objects'] if v['reference'] == choice['explorer']['object'])
            self.assertGreater(explorer['counters']['plus_one'], 0)
        actions = [a for a in obs['actions'] if a['kind'] == 'ChooseExplore']
        self.assertEqual({a['graveyard'] for a in actions}, {False, True})
        self.assertTrue(all(a['object'] == revealed['object'] for a in actions))
        wrong = dict(revealed['object'], generation=revealed['object']['generation'] + 1)
        before = [game.observe(v) for v in ('P0', 'P1')]
        for offered, context in [
            ({'kind': 'ChooseExplore', 'object': wrong, 'graveyard': False}, obs),
            ({'kind': 'ChooseExplore', 'object': revealed['object'], 'graveyard': True}, dict(obs, actor=other)),
        ]:
            with self.assertRaises(EngineError):
                game.act(context, offered)
            self.assertEqual([game.observe(v) for v in ('P0', 'P1')], before)

    def test_paid_entering_explorers_both_seats_and_nonland_choices(self):
        results = {'Hand': 0, 'Library': 0, 'Graveyard': 0}
        for name in ENTER_EXPLORERS:
            for seat in ('P0', 'P1'):
                for graveyard in (False, True):
                    with self.subTest(card=name, seat=seat, graveyard=graveyard):
                        destination = self.enter_scenario(name, seat, graveyard)
                        results[destination] += 1
        self.assertGreater(results['Library'], 0)
        self.assertGreater(results['Graveyard'], 0)

    def enter_scenario(self, name, seat, graveyard, seed=43, source_left=False, follow_draw=False):
        other = 'P1' if seat == 'P0' else 'P0'
        deck = sum(([n] * 6 for n in BASICS), []) + [name] * 16 + ['Bear Cub'] * 16 + ['Shock'] * 8
        if name == 'Merfolk Cave-Diver': deck += ['Cenote Scout'] * 12
        foe = ['Forest'] * 40 + ['Bear Cub'] * 30
        decks = [deck, foe] if seat == 'P0' else [foe, deck]
        paid = False
        reveal = None
        departed_explorer = None
        with Game(decks, seed=seed, starting_player=seat, deck_rules='Freeform') as game:
            for _ in range(30000):
                obs = game.observe('P0')
                self.assertIsNone(obs['outcome'], name)
                obs = game.observe(obs['actor'])
                actor = obs['actor']
                objects = {v['id']: v for v in obs['objects']}
                friends = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub']
                body = next((v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == name), None)
                if obs['explore_choice']:
                    self.choice(game, obs, counter_added=not source_left)
                    reveal = obs['explore_choice']['revealed']
                def score(a):
                    k = a['kind']
                    if k in ('AnnounceCast', 'Cast'):
                        n = objects[a['card']]['card']
                        if actor != seat:
                            foe_bears = [v for v in objects.values() if v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub']
                            return 1050 if name == 'Merfolk Cave-Diver' and n == 'Bear Cub' and not foe_bears else -1000
                        if n == name:
                            ready = name not in ("Miner's Guidewing", 'Over the Edge') or friends
                            if source_left:
                                available = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and not v['tapped'] and v['card'] in BASICS]
                                ready = any(v['card'] == 'Mountain' for v in available) and any(v['card'] == 'Forest' for v in available) and any(v['owner'] == seat and v['zone'] == 'Hand' and v['card'] == 'Shock' for v in objects.values())
                            return 1100 if ready and not paid else -1000
                        if n == 'Bear Cub': return 1050 if not friends else -1000
                        if n == 'Cenote Scout' and name == 'Merfolk Cave-Diver': return 1100 if body else -1000
                        if n == 'Shock' and (name == "Miner's Guidewing" or source_left): return 1200 if body else -1000
                        return -1000
                    if k == 'PlayLand':
                        n = objects[a['card']]['card']
                        count = sum(v['owner'] == actor and v['zone'] == 'Battlefield' and v['card'] == n for v in objects.values())
                        return 1300 - 10 * count
                    if k == 'TapMana': return 800 if obs['decision'] == 'pay_costs' else -1000
                    if k in ('AnnounceActivate', 'BeginActivate', 'Activate'):
                        return 1400 if actor == seat and objects[a['card']]['card'] == name and name == 'Seeker of Sunlight' else -1000
                    if k == 'ChooseMode': return 1500 if a['mode'] == 1 else -1000
                    if k == 'ChooseTarget':
                        if 'Card' not in a['target']: return -1000
                        target = objects[a['target']['Card']]
                        target_name = name if obs['announcing'] and obs['announcing']['source_card'] == 'Shock' else 'Bear Cub'
                        return 1500 if target['card'] == target_name and target['owner'] == seat else -1000
                    if k == 'ChooseExplore': return 1500 if a['graveyard'] == graveyard else -1000
                    if k == 'Discard': return 900 if objects[a['card']]['card'] in BASICS else 800
                    return {'Concede': -10000, 'Mulligan': -10000, 'Keep': 2000, 'PayCosts': 1500, 'PutTrigger': 1400, 'FinishTargets': 1100, 'FinishAttackers': 500, 'FinishBlockers': 500, 'Pass': 0}.get(k, -1000)
                j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                action = obs['actions'][j]
                if source_left and action['kind'] == 'ChooseTarget' and obs['announcing'] and obs['announcing']['source_card'] == 'Shock':
                    departed_explorer = objects[action['target']['Card']]['reference']
                if action['kind'] == 'PayCosts' and obs['announcing'] and obs['announcing']['source_card'] == name:
                    paid = True
                if action['kind'] == 'Cast' and objects[action['card']]['card'] == name:
                    paid = True
                after = self.replay(game, obs, j)
                event = after['last_explore']
                explorer_name = 'Bear Cub' if name in ("Miner's Guidewing", 'Over the Edge') else 'Cenote Scout' if name == 'Merfolk Cave-Diver' else name
                expected_count = 2 if name == 'Over the Edge' else 1
                observer_done = name != 'Merfolk Cave-Diver' or any(v['card'] == name and v['owner'] == seat and v['power'] == 3 for v in after['objects'])
                if event and event['explorer']['card'] == explorer_name and event['explorer']['controller'] == seat and event['sequence'] == expected_count and observer_done:
                    self.assertTrue(paid)
                    self.assertEqual(game.observe(other)['last_explore'], event)
                    self.assertEqual(event['sequence'], expected_count)
                    self.assertIsNotNone(event['revealed'])
                    r = event['revealed']
                    land = 'Land' in FACTS[r['card']]['type_line'].split(' — ')[0]
                    destination = 'Hand' if land else 'Graveyard' if graveyard else 'Library'
                    self.assertEqual(event['destination'], destination)
                    self.assertEqual(event['counter_added'], not land and not source_left)
                    if source_left:
                        self.assertEqual(event['explorer']['object'], departed_explorer)
                        old = next(v for v in after['objects'] if v['id'] == departed_explorer['card'])
                        self.assertEqual(old['zone'], 'Graveyard')
                        self.assertGreater(old['reference']['generation'], departed_explorer['generation'])
                        self.assertEqual(old['counters']['plus_one'], 0)
                    if land:
                        self.assertIn(event['result'], after['known_hand'])
                        self.assertGreater(event['result']['generation'], r['object']['generation'])
                        self.assertEqual(after['players'][int(seat[1])]['cards_drawn_this_turn'], obs['players'][int(seat[1])]['cards_drawn_this_turn'])
                    elif graveyard:
                        self.assertEqual(reveal, r)
                        moved = next(v for v in after['objects'] if v['reference'] == event['result'])
                        self.assertEqual(moved['zone'], 'Graveyard')
                        self.assertGreater(event['result']['generation'], r['object']['generation'])
                    else:
                        self.assertEqual(event['result'], r['object'])
                        self.assertEqual(after['known_library_top'][int(seat[1])], r)
                    if name == 'Merfolk Cave-Diver':
                        self.follow_unblockable(game, seat)
                    if follow_draw:
                        self.assertEqual(destination, 'Library')
                        self.follow_known_draw(game, event, seat)
                    return destination
            self.fail('No complete paid enter/explore continuation for ' + name)

    def follow_unblockable(self, game, seat):
        other = 'P1' if seat == 'P0' else 'P0'
        attacked = checked = False
        for _ in range(1000):
            obs = game.observe('P0')
            self.assertIsNone(obs['outcome'])
            obs = game.observe(obs['actor'])
            diver = next(v for v in obs['objects'] if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == 'Merfolk Cave-Diver')
            if checked and diver['power'] == 2:
                return
            if obs['decision'] == 'blockers' and diver['attacking']:
                bears = [v for v in obs['objects'] if v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub' and not v['tapped']]
                self.assertTrue(bears)
                self.assertEqual(obs['actor'], other)
                self.assertFalse(any(a['kind'] == 'Block' and a['attacker'] == diver['id'] for a in obs['actions']))
                before = game.observe(other)
                with self.assertRaises(EngineError):
                    game.act(obs, {'kind': 'Block', 'blocker': bears[0]['id'], 'attacker': diver['id']})
                self.assertEqual(game.observe(other), before)
                checked = True
            def score(a):
                if a['kind'] == 'Attack': return 1500 if a['card'] == diver['id'] and not attacked else -1000
                return {'FinishAttackers': 500, 'FinishBlockers': 500, 'Pass': 100, 'Discard': 600, 'Concede': -10000}.get(a['kind'], -1000)
            j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
            if obs['actions'][j]['kind'] == 'Attack':
                self.assertFalse(diver['summoning_sick'])
                self.assertEqual(diver['power'], 3)
                attacked = True
            self.replay(game, obs, j)
        self.fail('Cave-Diver unblockable/cleanup witness did not complete')

    def follow_known_draw(self, game, event, seat):
        reference = event['revealed']['object']
        other = 'P1' if seat == 'P0' else 'P0'
        for _ in range(1000):
            obs = game.observe('P0')
            self.assertIsNone(obs['outcome'])
            obs = game.observe(obs['actor'])
            def score(a):
                return {'Pass': 100, 'FinishAttackers': 500, 'FinishBlockers': 500, 'Discard': 600}.get(a['kind'], -1000)
            j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
            after = self.replay(game, obs, j)
            self.assertEqual(after['last_explore'], event)
            drawn = next((v for v in after['objects'] if v['id'] == reference['card'] and v['zone'] == 'Hand'), None)
            if drawn:
                self.assertGreater(drawn['reference']['generation'], reference['generation'])
                self.assertIn(drawn['reference'], after['known_hand'])
                self.assertIsNone(after['known_library_top'][int(seat[1])])
                visible = game.observe(other)
                self.assertIn(drawn['reference'], visible['known_hand'])
                self.assertTrue(any(v['reference'] == drawn['reference'] for v in visible['objects']))
                return
            self.assertEqual(after['known_library_top'][int(seat[1])]['object'], reference)
        self.fail('Publicly known top did not retain knowledge on actual draw')

    def test_known_top_remains_public_on_later_draw_both_seats(self):
        for seat in ('P0', 'P1'):
            with self.subTest(seat=seat):
                self.assertEqual(self.enter_scenario('Cenote Scout', seat, False, follow_draw=True), 'Library')

    def test_empty_library_still_counters_and_completes_both_seats(self):
        for seat in ('P0', 'P1'):
            with self.subTest(seat=seat):
                deck = ['Forest'] * 6 + ['Cenote Scout']
                foe = ['Forest'] * 40
                with Game([deck, foe] if seat == 'P0' else [foe, deck], seed=3, starting_player=seat, deck_rules='Freeform') as game:
                    for _ in range(400):
                        obs = game.observe('P0')
                        self.assertIsNone(obs['outcome'])
                        obs = game.observe(obs['actor'])
                        self.assertIsNone(obs['explore_choice'])
                        self.assertEqual(obs['players'][int(seat[1])]['library_count'], 0)
                        def score(a):
                            k = a['kind']
                            if k in ('Cast', 'AnnounceCast'): return 1200
                            if k == 'TapMana': return 800 if obs['decision'] == 'pay_costs' else -1000
                            return {'Concede': -10000, 'Mulligan': -10000, 'Keep': 2000, 'PlayLand': 1300, 'PayCosts': 1500, 'PutTrigger': 1400, 'FinishTargets': 1100, 'FinishAttackers': 500, 'FinishBlockers': 500, 'Pass': 0}.get(k, -1000)
                        j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                        after = self.replay(game, obs, j)
                        event = after['last_explore']
                        if event:
                            self.assertEqual(event['explorer']['controller'], seat)
                            self.assertEqual(event['explorer']['card'], 'Cenote Scout')
                            self.assertTrue(event['counter_added'])
                            self.assertIsNone(event['revealed'])
                            self.assertIsNone(event['destination'])
                            self.assertIsNone(event['result'])
                            explorer = next(v for v in after['objects'] if v['reference'] == event['explorer']['object'])
                            self.assertEqual(explorer['counters']['plus_one'], 1)
                            self.assertEqual(after['known_library_top'], [None, None])
                            break
                    else: self.fail('Empty-library explore did not complete')

    def test_real_source_departure_still_explores_without_counter_both_seats(self):
        for seat in ('P0', 'P1'):
            for graveyard in (False, True):
                with self.subTest(seat=seat, graveyard=graveyard):
                    self.enter_scenario('Cenote Scout', seat, graveyard, source_left=True)

    def test_public_land_put_into_hand_without_draw_both_seats(self):
        for seat in ('P0', 'P1'):
            with self.subTest(seat=seat):
                self.assertEqual(self.enter_scenario('Cenote Scout', seat, False, seed=0), 'Hand')

    def test_real_death_activation_double_explore_and_completed_observer(self):
        for name in ("Miner's Guidewing", 'Seeker of Sunlight', 'Over the Edge', 'Merfolk Cave-Diver'):
            for seat in ('P0', 'P1'):
                for graveyard in (False, True):
                    with self.subTest(card=name, seat=seat, graveyard=graveyard):
                        self.enter_scenario(name, seat, graveyard)

    def test_over_the_edge_actual_destroy_mode_both_seats(self):
        for seat in ('P0', 'P1'):
            with self.subTest(seat=seat):
                deck = ['Forest'] * 15 + ['Island'] * 15 + ['Candlestick'] * 16 + ['Over the Edge'] * 16
                foe = ['Forest'] * 40
                target = root = None
                paid = False
                with Game([deck, foe] if seat == 'P0' else [foe, deck], seed=43, starting_player=seat, deck_rules='Freeform') as game:
                    for _ in range(10000):
                        obs = game.observe('P0')
                        self.assertIsNone(obs['outcome'])
                        obs = game.observe(obs['actor'])
                        objects = {v['id']: v for v in obs['objects']}
                        candle = next((v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == 'Candlestick'), None)
                        def score(a):
                            k = a['kind']
                            if k in ('Cast', 'AnnounceCast'):
                                n = objects[a['card']]['card']
                                if n == 'Over the Edge': return 1100 if candle and not paid else -1000
                                return 1000 if n == 'Candlestick' and not candle and not paid else -1000
                            if k == 'ChooseMode': return 1600 if a['mode'] == 0 else -1000
                            if k == 'ChooseTarget': return 1500 if a['target'] == {'Card': candle['id']} else -1000
                            if k == 'TapMana': return 800 if obs['decision'] == 'pay_costs' else -1000
                            return {'Concede': -10000, 'Mulligan': -10000, 'Keep': 2000, 'PlayLand': 1300, 'PayCosts': 1500, 'PutTrigger': 1400, 'ArrangeLibrary': 1000, 'FinishTargets': 1100, 'FinishAttackers': 500, 'FinishBlockers': 500, 'Pass': 0, 'Discard': 600}.get(k, -1000)
                        j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                        a = obs['actions'][j]
                        if a['kind'] == 'ChooseTarget': target = candle['reference']
                        if a['kind'] == 'PayCosts' and obs['announcing'] and obs['announcing']['source_card'] == 'Over the Edge': paid = True
                        if a['kind'] == 'Pass' and obs['stack'] and obs['stack'][-1]['source_card'] == 'Over the Edge': root = obs['stack'][-1]
                        after = self.replay(game, obs, j)
                        if root and not any(v['id'] == root['id'] for v in after['stack']):
                            self.assertTrue(paid)
                            self.assertIsNotNone(target)
                            moved = next(v for v in after['objects'] if v['id'] == target['card'])
                            self.assertEqual(moved['zone'], 'Graveyard')
                            self.assertGreater(moved['reference']['generation'], target['generation'])
                            self.assertIsNone(after['last_explore'])
                            self.assertIsNone(after['explore_choice'])
                            break
                    else: self.fail('Over the Edge destroy mode did not resolve')

    def test_complete_map_providers_and_real_token_activation_both_seats(self):
        names = ["Cartographer's Companion", 'Fanatical Offering', 'Get Lost', 'Sentinel of the Nameless City', 'Spyglass Siren', 'Waterwind Scout']
        for name in names:
            for seat in ('P0', 'P1'):
                with self.subTest(card=name, seat=seat):
                    self.map_scenario(name, seat)

    def test_get_lost_still_gives_maps_after_indestructible_destroy_both_seats(self):
        for seat in ('P0', 'P1'):
            with self.subTest(seat=seat):
                self.map_scenario('Get Lost', seat, indestructible=True)

    def follow_sentinel_attack(self, game, card, seat):
        attacked = False
        old = {v['reference']['card']: v['reference'] for v in game.observe(seat)['objects'] if v['zone'] == 'Battlefield' and v['token'] and v['token']['subtypes'] == 'Map'}
        for _ in range(1000):
            obs = game.observe('P0')
            self.assertIsNone(obs['outcome'])
            obs = game.observe(obs['actor'])
            sentinel = next(v for v in obs['objects'] if v['id'] == card)
            def score(a):
                if a['kind'] == 'Attack': return 1500 if a['card'] == card and not attacked else -1000
                return {'FinishAttackers': 500, 'FinishBlockers': 500, 'Pass': 100, 'PutTrigger': 1400, 'Discard': 600, 'Concede': -10000}.get(a['kind'], -1000)
            j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
            if obs['actions'][j]['kind'] == 'Attack':
                self.assertFalse(sentinel['summoning_sick'])
                self.assertFalse(sentinel['tapped'])
                attacked = True
            after = self.replay(game, obs, j)
            maps = [v for v in after['objects'] if v['owner'] == seat and v['zone'] == 'Battlefield' and v['token'] and v['token']['subtypes'] == 'Map' and old.get(v['id']) != v['reference']]
            if maps:
                self.assertTrue(attacked)
                self.assertEqual(len(maps), 1)
                live = next(v for v in after['objects'] if v['id'] == card)
                self.assertFalse(live['tapped'])
                self.assertTrue(live['attacking'])
                return
        self.fail('Actual vigilance attack did not create Sentinel Map')

    def map_scenario(self, name, seat, indestructible=False):
        other = 'P1' if seat == 'P0' else 'P0'
        map_seat = other if name == 'Get Lost' else seat
        deck = sum(([n] * 6 for n in BASICS), []) + [name] * 16 + ['Bear Cub'] * 24
        if indestructible: deck += ['Adamant Will'] * 12
        foe = ['Forest'] * 40 + ['Bear Cub'] * 30
        decks = [deck, foe] if seat == 'P0' else [foe, deck]
        paid = False
        activated = None
        destroyed = None
        sacrificed = None
        source = None
        maps_created = False
        draw_baseline = None
        with Game(decks, seed=43, starting_player=seat, deck_rules='Freeform') as game:
            for _ in range(40000):
                obs = game.observe('P0')
                self.assertIsNone(obs['outcome'], name)
                obs = game.observe(obs['actor'])
                actor = obs['actor']
                objects = {v['id']: v for v in obs['objects']}
                friends = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub']
                foes = [v for v in objects.values() if v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub']
                maps = [v for v in objects.values() if v['zone'] == 'Battlefield' and v['token'] and v['token']['subtypes'] == 'Map']
                own_maps = [v for v in maps if v['owner'] == map_seat]
                if own_maps and not maps_created:
                    self.assertTrue(paid)
                    self.assertEqual(len(own_maps), 2 if name == 'Get Lost' else 1)
                    maps_created = True
                    if destroyed:
                        self.assertEqual(objects[destroyed['card']]['zone'], 'Battlefield' if indestructible else 'Graveyard')
                        if indestructible:
                            self.assertEqual(objects[destroyed['card']]['reference'], destroyed)
                            self.assertTrue(objects[destroyed['card']]['keywords'] & 8192)
                    if sacrificed:
                        self.assertEqual(objects[sacrificed['card']]['zone'], 'Graveyard')
                    if draw_baseline is not None:
                        self.assertEqual(obs['players'][int(seat[1])]['cards_drawn_this_turn'] - draw_baseline, 2)
                if obs['explore_choice']:
                    self.choice(game, obs)
                ann = obs['announcing']
                if activated and ann and ann['object'] == activated:
                    if obs['decision'] == 'targets':
                        offered = [a['target']['Card'] for a in obs['actions'] if a['kind'] == 'ChooseTarget' and 'Card' in a['target']]
                        self.assertTrue(offered)
                        self.assertTrue(all(objects[c]['owner'] == map_seat and 'Creature' in FACTS[objects[c]['card']]['type_line'].split(' — ')[0] for c in offered))
                        forbidden = next((v for v in objects.values() if v['owner'] != map_seat and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub'), None)
                        if forbidden:
                            before = game.observe(actor)
                            with self.assertRaises(EngineError):
                                game.act(obs, {'kind': 'ChooseTarget', 'target': {'Card': forbidden['id']}})
                            self.assertEqual(game.observe(actor), before)
                    if obs['decision'] == 'pay_costs':
                        self.assertEqual(obs['payment_request']['required'], [0] * 6)
                        self.assertEqual(obs['payment_request']['generic'], 1)
                        self.assertEqual(objects[activated['card']]['reference'], activated)
                        self.assertFalse(objects[activated['card']]['tapped'])
                def score(a):
                    k = a['kind']
                    if k in ('AnnounceCast', 'Cast'):
                        n = objects[a['card']]['card']
                        if n == name and actor == seat:
                            ready = len(friends) >= (2 if name == 'Fanatical Offering' else 1) and (name != 'Get Lost' or foes)
                            if indestructible: ready = ready and any(v['keywords'] & 8192 for v in foes)
                            return 1100 if ready and not paid else -1000
                        if n == 'Adamant Will' and indestructible and actor == seat and not paid:
                            mana = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] in BASICS and not v['tapped']]
                            ready = foes and len(mana) >= 4 and sum(v['card'] == 'Plains' for v in mana) >= 2 and any(v['owner'] == seat and v['zone'] == 'Hand' and v['card'] == name for v in objects.values())
                            return 1200 if ready else -1000
                        return 1000 if n == 'Bear Cub' and (len(friends) if actor == seat else len(foes)) < 2 else -1000
                    if k in ('AnnounceActivate', 'BeginActivate', 'Activate'):
                        v = objects[a['card']]
                        return 1500 if actor == map_seat and v in own_maps and activated is None else -1000
                    if k == 'PlayLand':
                        n = objects[a['card']]['card']
                        count = sum(v['owner'] == actor and v['zone'] == 'Battlefield' and v['card'] == n for v in objects.values())
                        return 1300 - 10 * count
                    if k == 'TapMana': return 800 if obs['decision'] == 'pay_costs' else -1000
                    if k == 'ChooseAdditionalCost': return 1500 if objects[a['object']['card']]['card'] == 'Bear Cub' else -1000
                    if k == 'ChooseTarget':
                        if 'Card' not in a['target']: return -1000
                        v = objects[a['target']['Card']]
                        target_owner = other if ann and ann['source_card'] in ('Get Lost', 'Adamant Will') else actor
                        if indestructible and ann and ann['source_card'] == 'Get Lost' and not v['keywords'] & 8192: return -1000
                        return 1600 if v['owner'] == target_owner and v['card'] == 'Bear Cub' else -1000
                    if k == 'ChooseExplore': return 1500 if a['graveyard'] else 1000
                    if k == 'Discard': return 900 if objects[a['card']]['card'] in BASICS else 800
                    return {'Concede': -10000, 'Mulligan': -10000, 'Keep': 2000, 'PayCosts': 1500, 'PutTrigger': 1400, 'FinishTargets': 1100, 'FinishAttackers': 500, 'FinishBlockers': 500, 'Pass': 0}.get(k, -1000)
                j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                a = obs['actions'][j]
                if activated and a['kind'] == 'PayCosts' and ann and ann['object'] == activated:
                    self.assertEqual(sum(a['payment']), 1)
                if a['kind'] == 'ChooseAdditionalCost': sacrificed = a['object']
                if a['kind'] == 'ChooseTarget' and ann and ann['source_card'] == 'Get Lost':
                    destroyed = objects[a['target']['Card']]['reference']
                if a['kind'] == 'PayCosts' and ann and ann['source_card'] == name:
                    paid = True
                    source = ann['object']
                if a['kind'] == 'Cast' and objects[a['card']]['card'] == name:
                    paid = True
                    source = objects[a['card']]['reference']
                if a['kind'] in ('AnnounceActivate', 'BeginActivate', 'Activate') and objects[a['card']] in own_maps:
                    activated = objects[a['card']]['reference']
                    self.assertEqual(actor, map_seat)
                    self.assertEqual(obs['active'], actor)
                    self.assertIn(obs['step'], ('Main1', 'Main2'))
                    self.assertEqual(obs['stack'], [])
                if a['kind'] == 'Pass' and obs['stack'] and obs['stack'][-1]['source_card'] == 'Fanatical Offering':
                    draw_baseline = obs['players'][int(seat[1])]['cards_drawn_this_turn']
                after = self.replay(game, obs, j)
                event = after['last_explore']
                if activated and event and event['explorer']['controller'] == map_seat:
                    self.assertTrue(maps_created)
                    self.assertIsNotNone(source)
                    self.assertEqual(event['explorer']['card'], 'Bear Cub')
                    self.assertFalse(any(v['reference'] == activated and v['zone'] == 'Battlefield' for v in after['objects']))
                    if name == 'Sentinel of the Nameless City':
                        self.follow_sentinel_attack(game, source['card'], seat)
                    return
            self.fail('No complete provider and real Map activation for ' + name)


if __name__ == '__main__':
    unittest.main()
