"""Public library acceptance for exact live target/entry conditions."""
import unittest
from rustmage import EngineError, Game

NAMES = ['Aunt May', 'Dreamdew Entrancer', 'Fear of Immobility', 'Hazardroot Herbalist', 'Ingenious Leonin', 'Little Bear', 'Thwip!', 'Wakandan Royal Guard', 'Yip Yip!']


class SubjectConditionTests(unittest.TestCase):
    def test_nine_complete_bodies_both_seats_and_both_live_branches(self):
        for i, name in enumerate(NAMES):
            for seat in ('P0', 'P1'):
                for positive in (False, True):
                    target_name = ('Bear Cub' if i == 5 else 'Three Tree Mascot') if positive else 'Llanowar Elves'
                    other = 'P1' if seat == 'P0' else 'P0'
                    target_owner = other if (i == 1 and not positive) or (i == 2 and positive) else seat
                    lands = ['Forest'] * 12 + ['Plains'] * 12 + ['Island'] * 8
                    deck = lands + [name] * 8 + [target_name] * 12 + ['Hop to It'] * 8
                    foe = ['Forest'] * 32 + [target_name] * 28
                    decks = [deck, foe] if seat == 'P0' else [foe, deck]
                    with self.subTest(card=name, seat=seat, positive=positive), Game(decks, seed=13, starting_player=seat, deck_rules='Freeform') as game:
                        done = False
                        for _ in range(18000):
                            obs = game.observe('P0')
                            self.assertIsNone(obs['outcome'], (name, positive))
                            obs = game.observe(obs['actor'])
                            objects = {v['id']: v for v in obs['objects']}
                            source_up = any(v['card'] == name and v['owner'] == seat and v['zone'] == 'Battlefield' for v in objects.values())
                            ready = any(v['card'] == target_name and v['owner'] == target_owner and v['zone'] == 'Battlefield' for v in objects.values())
                            def score(a):
                                k = a['kind']
                                if k in ('Cast', 'AnnounceCast'):
                                    n = objects[a['card']]['card']
                                    if n == name:
                                        return 800 if (i == 0 or ready) and not source_up else -100
                                    if n == 'Hop to It': return 750 if i == 3 and positive and source_up else -100
                                    return 600 if n == target_name and not (i == 4 and source_up and ready and obs['actor'] == seat) else -100
                                if k in ('Activate', 'BeginActivate', 'AnnounceActivate'):
                                    return 950 if i == 4 and objects[a['card']]['card'] == name else -100
                                if k == 'ChooseTarget' and 'Card' in a['target']:
                                    v = objects[a['target']['Card']]
                                    if i == 3 and positive: return 1000 if v['token'] and v['owner'] == seat else -100
                                    return 1000 if v['card'] == target_name and v['owner'] == target_owner else -100
                                if k == 'Attack': return 500 if i in (3, 4) and obs['actor'] == seat and source_up and objects[a['card']]['card'] == target_name else -100
                                if k == 'PlayLand':
                                    n = objects[a['card']]['card']
                                    count = sum(v['card'] == n and v['owner'] == obs['actor'] and v['zone'] == 'Battlefield' for v in objects.values())
                                    return 900 - count
                                if k == 'TapMana': return 400 if obs['decision'] == 'pay_costs' else -100
                                return {'Keep': 1200, 'PayCosts': 1100, 'PutTrigger': 850, 'FinishTargets': 700, 'FinishAttackers': 400, 'FinishBlockers': 400, 'Pass': 0, 'Concede': -10000, 'Mulligan': -100, 'Block': -100, 'ActivateMana': -100, 'AnnounceMana': -100}.get(k, 100)
                            j = max(range(len(obs['actions'])), key=lambda k: score(obs['actions'][k]))
                            action = obs['actions'][j]
                            packet = obs['stack'][-1] if action['kind'] == 'Pass' and obs['stack'] else None
                            body = packet and packet['source_card'] == name and (i in (6, 8) or packet['kind'] != 'Spell')
                            subject = None
                            if body:
                                ref = packet['entering_object'] if i == 0 else (packet['targets'][0]['object'] if packet['targets'] else None)
                                subject = next((v for v in objects.values() if v['reference'] == ref), None)
                                if subject and i == 3 and positive and not subject['token']: subject = None
                                if subject and not (i == 3 and positive) and subject['card'] != target_name: subject = None
                            if obs['decision'] == 'targets':
                                wrong = dict(obs, actor='P1' if obs['actor'] == 'P0' else 'P0')
                                with self.assertRaises(EngineError): game.act(wrong, action)
                                self.assertEqual(game.observe(obs['actor']), obs)
                                with self.assertRaises(EngineError): game.act(obs, {'kind': 'ChooseTarget', 'target': {'Card': 2**32 - 1}})
                                self.assertEqual(game.observe(obs['actor']), obs)
                                alternate = next((a for a in obs['actions'] if a['kind'] == 'ChooseTarget' and a != action), None)
                                if alternate:
                                    with game.clone() as fork: fork.act(fork.observe(obs['actor']), alternate)
                                    self.assertEqual(game.observe(obs['actor']), obs)
                            saved = game.snapshot()
                            with game.clone() as fork: expected = fork.step(fork.observe(obs['actor']), j)
                            after = game.step(obs, j)
                            self.assertEqual(after, expected)
                            game.restore(saved)
                            with self.assertRaisesRegex(EngineError, 'StaleRevision'): game.step(obs, j)
                            after = game.step(game.observe(obs['actor']), j)
                            expected['revision'] = after['revision']
                            self.assertEqual(after, expected)
                            game.drop_snapshot(saved)
                            if subject and not any(v['id'] == packet['id'] for v in after['stack']):
                                current = next((v for v in after['objects'] if v['reference'] == subject['reference']), None)
                                self.assertIsNotNone(current)
                                self.assertEqual(after['schema'], 84)
                                owner = int(seat[1])
                                plus = current['counters']['plus_one'] - subject['counters']['plus_one']
                                if i == 0:
                                    self.assertEqual(after['players'][owner]['life'], obs['players'][owner]['life'] + 1)
                                    self.assertEqual(plus, int(positive))
                                elif i == 1:
                                    self.assertTrue(current['tapped'])
                                    self.assertEqual(current['counters']['stun'] - subject['counters']['stun'], 3)
                                    self.assertEqual(after['players'][owner]['hand_count'] - obs['players'][owner]['hand_count'], 2 if positive else 0)
                                elif i == 2:
                                    self.assertTrue(current['tapped'])
                                    self.assertEqual(current['counters']['stun'] - subject['counters']['stun'], int(positive))
                                elif i == 3:
                                    self.assertEqual(current['power'] - subject['power'], 1)
                                    self.assertEqual(bool(current['keywords'] & 128), positive)
                                elif i == 4:
                                    self.assertEqual(plus, 1)
                                    self.assertEqual(bool(current['keywords'] & 512), positive)
                                elif i == 5:
                                    self.assertFalse(current['tapped'])
                                    self.assertEqual(plus, int(positive))
                                elif i == 6:
                                    self.assertEqual((current['power'] - subject['power'], current['toughness'] - subject['toughness']), (2, 2))
                                    self.assertTrue(current['keywords'] & 1)
                                    self.assertEqual(after['players'][owner]['life'] - obs['players'][owner]['life'], 2 if positive else 0)
                                elif i == 7: self.assertEqual(plus, 2 if positive else 1)
                                else:
                                    self.assertEqual((current['power'] - subject['power'], current['toughness'] - subject['toughness']), (2, 2))
                                    self.assertEqual(bool(current['keywords'] & 1), positive)
                                done = True
                                break
                        self.assertTrue(done, name)


    def test_optional_zero_targets_never_gain_subject_dependent_benefits(self):
        for name in ('Dreamdew Entrancer', 'Fear of Immobility'):
            for seat in ('P0', 'P1'):
                lands = ['Forest'] * 16 + ['Island'] * 16 if name == 'Dreamdew Entrancer' else ['Plains'] * 32
                deck = lands + [name] * 28
                decks = [deck, ['Forest'] * 60] if seat == 'P0' else [['Forest'] * 60, deck]
                with self.subTest(card=name, seat=seat), Game(decks, seed=9, starting_player=seat, deck_rules='Freeform') as game:
                    finished = False
                    for _ in range(3000):
                        obs = game.observe('P0')
                        self.assertIsNone(obs['outcome'])
                        obs = game.observe(obs['actor'])
                        def score(a):
                            return {'Keep': 1200, 'FinishTargets': 1150, 'PayCosts': 1100, 'PutTrigger': 1050, 'PlayLand': 1000, 'Cast': 900, 'AnnounceCast': 900, 'Pass': 0, 'Concede': -10000, 'ChooseTarget': -1000, 'ActivateMana': -100, 'AnnounceMana': -100, 'TapMana': 800 if obs['decision'] == 'pay_costs' else -100}.get(a['kind'], -100)
                        j = max(range(len(obs['actions'])), key=lambda k: score(obs['actions'][k]))
                        packet = obs['stack'][-1] if obs['actions'][j]['kind'] == 'Pass' and obs['stack'] else None
                        empty = packet and packet['source_card'] == name and packet['kind'] != 'Spell'
                        if empty: self.assertEqual(packet['targets'], [])
                        after = game.step(obs, j)
                        if empty and not any(v['id'] == packet['id'] for v in after['stack']):
                            owner = int(seat[1])
                            self.assertEqual(after['players'][owner]['hand_count'], obs['players'][owner]['hand_count'])
                            source = next(v for v in after['objects'] if v['reference'] == packet['object'])
                            self.assertFalse(source['tapped'])
                            self.assertEqual(source['counters']['stun'], 0)
                            finished = True
                            break
                    self.assertTrue(finished)


if __name__ == '__main__':
    unittest.main()
