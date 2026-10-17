"""Independent public ABI acceptance for target-owner library destinations."""
import unittest
from rustmage import EngineError, Game

NAMES = [
    'Chrome Companion', 'Hoverstone Pilgrim', 'Jade-Cast Sentinel',
    'Malevolent Chandelier', "The Spot's Portal", 'Lost in Space',
    'Misleading Motes', 'Return to the Sewers', 'Riverwalk Technique',
    'Trip Up', 'Uncharted Voyage', 'Unlucky Drop', 'Vanish from Sight',
]


class LibraryTuckTests(unittest.TestCase):
    def test_complete_bodies_owner_choices_private_continuations_and_replay(self):
        for i, name in enumerate(NAMES):
            for seat in ('P0', 'P1'):
                for destination in (('Bottom',) if i < 5 else ('Top', 'Bottom')):
                    other = 'P1' if seat == 'P0' else 'P0'
                    lands = ['Island'] * 16 + ['Swamp'] * 8 + ['Forest'] * 8
                    deck = lands + [name] * 12 + ['Bear Cub'] * 12 + ['Fell'] * 8
                    opponent = ['Forest'] * 32 + ['Bear Cub'] * 28
                    decks = [deck, opponent] if seat == 'P0' else [opponent, deck]
                    with self.subTest(card=name, seat=seat, destination=destination), Game(
                        decks, seed=7, starting_player=seat, deck_rules='Freeform'
                    ) as game:
                        done = False
                        replayed = 0
                        seen_choice = False
                        for _ in range(16000):
                            obs = game.observe('P0')
                            self.assertIsNone(obs['outcome'], name)
                            obs = game.observe(obs['actor'])
                            objects = {v['id']: v for v in obs['objects']}
                            sources = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == name]
                            dead = [v for v in objects.values() if v['zone'] == 'Graveyard' and v['card'] == 'Bear Cub']
                            bears = [v for v in objects.values() if v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub']
                            def score(a):
                                kind = a['kind']
                                if kind in ('Cast', 'AnnounceCast'):
                                    n = objects[a['card']]['card']
                                    if obs['actor'] == seat and n == name:
                                        return 700 if (i >= 4 or not sources) and (i < 4 or dead) else -100
                                    if n == 'Bear Cub':
                                        return 500 if len(bears) < 4 else -100
                                    if n == 'Fell':
                                        return 600 if i < 4 and not dead and bears else -100
                                    return -100
                                if kind in ('Activate', 'BeginActivate', 'AnnounceActivate'):
                                    return 800 if objects[a['card']]['card'] == name and i < 4 else -100
                                if kind == 'ChooseTarget':
                                    if 'Card' in a['target']:
                                        v = objects[a['target']['Card']]
                                        return 900 if v['card'] == 'Bear Cub' and (i >= 4 or v['zone'] == 'Graveyard' or obs['announcing']['source_card'] == 'Fell') else -100
                                    return -100
                                if kind == 'ChooseLibraryDestination':
                                    return 1000 if a['destination'] == destination else -100
                                if kind == 'ChooseMode': return 900 if a['mode'] == 0 else -100
                                if kind == 'PayCosts': return 1000
                                if kind == 'TapMana': return 400 if obs['decision'] == 'pay_costs' else -100
                                if kind == 'PlayLand':
                                    n = objects[a['card']]['card']
                                    have = sum(v['zone'] == 'Battlefield' and v['owner'] == obs['actor'] and v['card'] == n for v in objects.values())
                                    return 850 - have
                                if kind == 'ArrangeLibrary': return 900 if a['destination'] == 'Top' else -100
                                return {'Keep': 1200, 'PutTrigger': 600, 'FinishTargets': 500, 'Pass': 0, 'Concede': -10000, 'Attack': -100, 'Block': -100, 'ActivateMana': -100, 'AnnounceMana': -100, 'Mulligan': -100}.get(kind, 100)
                            choice = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                            action = obs['actions'][choice]
                            frame = obs['target_library']
                            target = None
                            if frame:
                                seen_choice = True
                                target = objects[frame['target']['card']]
                                self.assertEqual(target['reference'], frame['target'])
                                self.assertEqual((frame['player'], obs['actor']), (target['owner'], target['owner']))
                                private = game.observe('P1' if obs['actor'] == 'P0' else 'P0')
                                self.assertIsNone(private['target_library'])
                                self.assertEqual(private['actions'], [])
                                self.assertIsNone(obs['library_choice'])
                                for bad in ({'kind': 'Pass'}, {'kind': 'ChooseLibraryDestination', 'object': dict(frame['target'], generation=frame['target']['generation'] + 1), 'destination': destination}):
                                    with self.assertRaisesRegex(EngineError, 'IllegalAction'): game.act(obs, bad)
                                    self.assertEqual(game.observe(obs['actor']), obs)
                                wrong = dict(obs, actor='P1' if obs['actor'] == 'P0' else 'P0')
                                with self.assertRaises(EngineError): game.act(wrong, action)
                                self.assertEqual(game.observe(obs['actor']), obs)
                                with game.clone() as fork:
                                    fork.act(fork.observe(obs['actor']), dict(action, destination='Bottom' if destination == 'Top' else 'Top'))
                                    self.assertEqual(game.observe(obs['actor']), obs)
                            packet = obs['stack'][-1] if action['kind'] == 'Pass' and obs['stack'] else None
                            if packet and packet['source_card'] == name and i < 5 and packet['targets']:
                                r = packet['targets'][0]['object']
                                target = objects.get(r['card']) if r else None
                            saved = game.snapshot()
                            with game.clone() as fork: expected = fork.step(fork.observe(obs['actor']), choice)
                            self.assertEqual(game.step(obs, choice), expected)
                            game.restore(saved)
                            with self.assertRaisesRegex(EngineError, 'StaleRevision'): game.step(obs, choice)
                            after = game.step(game.observe(obs['actor']), choice)
                            expected['revision'] = after['revision']
                            self.assertEqual(after, expected)
                            game.drop_snapshot(saved)
                            replayed += 1
                            if target and not any(v['reference'] == target['reference'] for v in after['objects']):
                                owner = int(target['owner'][1])
                                self.assertEqual(after['players'][owner]['library_count'], obs['players'][owner]['library_count'] + 1)
                                self.assertEqual(after['players'][owner]['hand_count'], obs['players'][owner]['hand_count'])
                                self.assertEqual(after['schema'], 96)
                                movement = after['last_target_library_move']
                                self.assertEqual(movement['target_before'], target['reference'])
                                self.assertEqual(movement['card'], target['card'])
                                self.assertEqual(movement['target_after'], dict(target['reference'], generation=target['reference']['generation'] + 1))
                                self.assertEqual((movement['owner'], movement['destination']), (target['owner'], destination))
                                self.assertEqual(movement['source'], frame['source'] if frame else packet['object'])
                                self.assertEqual(game.observe('P0')['last_target_library_move'], game.observe('P1')['last_target_library_move'])
                                if i in (5, 10, 12):
                                    controller_view = game.observe(seat)
                                    self.assertIsNotNone(controller_view['library_choice'])
                                    self.assertIsNone(game.observe(other)['library_choice'])
                                if i == 7:
                                    self.assertTrue(any(v['zone'] == 'Battlefield' and v['token'] and v['token']['subtypes'] == 'Mutagen' for v in after['objects']))
                                if destination == 'Top':
                                    # The next draw of that owner must consume this exact top card.
                                    # Surveil keeps the top card; no unrelated casts or activations run.
                                    drawn = False
                                    for _ in range(600):
                                        future = game.observe('P0')
                                        self.assertIsNone(future['outcome'])
                                        future = game.observe(future['actor'])
                                        self.assertEqual(game.observe('P0')['last_target_library_move'], movement)
                                        self.assertEqual(game.observe('P1')['last_target_library_move'], movement)
                                        visible = next((v for v in game.observe(target['owner'])['objects'] if v['id'] == target['id'] and v['zone'] == 'Hand'), None)
                                        if visible:
                                            self.assertEqual(visible['reference']['generation'], target['reference']['generation'] + 2)
                                            drawn = True
                                            break
                                        def draw_score(a):
                                            if a['kind'] == 'ArrangeLibrary': return 900 if a['destination'] == 'Top' else -100
                                            return {'Pass': 800, 'Keep': 1000, 'PutTrigger': 900, 'FinishTargets': 700, 'FinishAttackers': 700, 'FinishBlockers': 700, 'Concede': -10000}.get(a['kind'], -100)
                                        j = max(range(len(future['actions'])), key=lambda k: draw_score(future['actions'][k]))
                                        game.step(future, j)
                                    self.assertTrue(drawn, 'top destination was not the next owner draw')
                                done = True
                                break
                        self.assertTrue(done, name)
                        self.assertGreater(replayed, 0)
                        self.assertEqual(seen_choice, i >= 5)


if __name__ == '__main__':
    unittest.main()
