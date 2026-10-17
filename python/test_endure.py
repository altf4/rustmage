"""Actual paid fixed-Endure bodies through the in-process public API."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError, Game

NAMES = ['Anafenza, Unyielding Lineage', 'Descendant of Storms', 'Dusyut Earthcarver', 'Fortress Kin-Guard', 'Inspirited Vanguard', 'Kin-Tree Nurturer', 'Sandskitter Outrider', 'Sinkhole Surveyor']
AMOUNTS = [2, 1, 3, 1, 2, 1, 2, 1]
ROOT = Path(__file__).resolve().parents[1]
FACTS = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}

class EndureTests(unittest.TestCase):
    def replay(self, game, observation, index):
        saved = game.snapshot()
        actor = observation['actor']
        other = 'P1' if actor == 'P0' else 'P0'
        with game.clone() as child:
            expected = child.step(child.observe(actor), index)
            expected_other = child.observe(other)
        game.step(observation, index)
        game.restore(saved)
        with self.assertRaisesRegex(EngineError, 'StaleRevision'):
            game.step(observation, index)
        after = game.step(game.observe(actor), index)
        expected['revision'] = after['revision']
        self.assertEqual(after, expected)
        other_view = game.observe(other)
        expected_other['revision'] = other_view['revision']
        self.assertEqual(other_view, expected_other)
        game.drop_snapshot(saved)
        return after

    def reject(self, game, observation, action, actor=None):
        before = [game.observe(p) for p in ('P0', 'P1')]
        with self.assertRaises(EngineError):
            game.act(dict(observation, actor=actor or observation['actor']), action)
        self.assertEqual([game.observe(p) for p in ('P0', 'P1')], before)

    def test_all_eight_actual_paid_bodies_both_players_both_endure_results(self):
        for i, name in enumerate(NAMES):
            for player in ('P0', 'P1'):
                for counters in (False, True):
                    with self.subTest(card=name, seat=player, counters=counters):
                        self.scenario(i, player, counters)

    def test_real_unsummon_source_departure_and_optional_payment_decline(self):
        for player in ('P0', 'P1'):
            with self.subTest(seat=player, source_left=True):
                self.scenario(3, player, True, bounce=True)
            with self.subTest(seat=player, decline=True):
                self.scenario(1, player, False, decline=True)

    def scenario(self, i, player, counters, bounce=False, decline=False):
        name, amount = NAMES[i], AMOUNTS[i]
        primary = 'Forest' if i in (2, 4) else 'Swamp' if i in (5, 6, 7) else 'Plains'
        own = ['Plains'] * 24 + [primary] * 24 + ['Island'] * 12 + ['Swamp'] * 12 + [name] * 12 + ['Savannah Lions'] * 12 + ['Stab'] * 12 + ['Unsummon'] * 12
        enemy = ['Forest'] * 80
        decks = [own, enemy] if player == 'P0' else [enemy, own]
        other = 'P1' if player == 'P0' else 'P0'
        paid = False
        source = None
        attacked = False
        killed = False
        bounced = False
        declined = False
        choices = 0
        with Game(decks, seed=237 + i, starting_player=player, deck_rules='Freeform') as game:
            for _ in range(18000):
                public = game.observe('P0')
                self.assertIsNone(public['outcome'], (name, player))
                o = game.observe(public['actor'])
                actor = o['actor']
                objects = {v['id']: v for v in o['objects']}
                lands = [v for v in objects.values() if v['owner'] == player and v['zone'] == 'Battlefield' and v['card'] in ('Plains', 'Forest', 'Island', 'Swamp')]
                lions = [v for v in objects.values() if v['owner'] == player and v['zone'] == 'Battlefield' and v['card'] == 'Savannah Lions']
                tokens = [v for v in objects.values() if v['owner'] == player and v['zone'] == 'Battlefield' and v['token'] and v['token']['subtypes'] == 'Spirit']
                quiet = o['decision'] == 'priority' and not o['announcing'] and not o['stack'] and not o['resolving']
                ready = paid and (killed if i == 0 else attacked if i in (1, 4, 7) else True)
                needed = 2 if i == 4 else 1
                if ready and quiet and (declined if decline else bounced if bounce else choices == needed):
                    self.assertEqual(choices, 0 if bounce or decline else needed)
                    self.assertEqual(len(tokens), 0 if decline or counters and not bounce else needed)
                    for token in tokens:
                        self.assertEqual((token['power'], token['toughness'], token['token']['colors']), (amount, amount, 1))
                        self.assertEqual(token['keywords'] & 1, 0)
                    if bounce:
                        owned = {v['id']: v for v in game.observe(player)['objects']}
                        self.assertEqual(owned[source]['zone'], 'Hand')
                        self.assertEqual(owned[source]['counters']['plus_one'], 0)
                    elif not decline:
                        self.assertEqual(objects[source]['counters']['plus_one'], amount * needed if counters else 0)
                        stats = FACTS[name]
                        self.assertEqual(objects[source]['power'], int(stats['power']) + (amount * needed if counters else 0))
                        self.assertEqual(objects[source]['toughness'], int(stats['toughness']) + (amount * needed if counters else 0))
                    if i == 7:
                        self.assertEqual(o['players'][0 if player == 'P0' else 1]['life'], 19)
                    return
                frame = o['endure_choice']
                if frame:
                    self.assertEqual(actor, player)
                    self.assertIsNone(game.observe(other)['endure_choice'])
                    self.assertEqual(frame['source'], objects[source]['reference'])
                    self.assertEqual(frame['amount'], amount)
                    self.reject(game, o, {'kind': 'Pass'})
                    self.reject(game, o, {'kind': 'ChooseEndure', 'put_counters': counters}, other)
                    self.assertFalse(any(a['kind'] in ('TapMana', 'ActivateMana', 'PayResolutionMana') for a in o['actions']))
                def score(action):
                    kind = action['kind']
                    if kind in ('Cast', 'BeginCast'):
                        return -1000
                    if kind == 'AnnounceCast':
                        card = objects[action['card']]['card']
                        if actor != player:
                            return -1000
                        if card == name:
                            return 1800 if not paid and len(lands) >= 8 and (i != 0 or lions) and (not bounce or any(v['card'] == 'Island' and not v['tapped'] for v in lands) and any(v['owner'] == player and v['zone'] == 'Hand' and v['card'] == 'Unsummon' for v in objects.values())) else -1000
                        if card == 'Savannah Lions':
                            return 1600 if i == 0 and not paid and not lions else -1000
                        if card == 'Stab':
                            return 2100 if paid and i == 0 and not killed and lions else -1000
                        if card == 'Unsummon':
                            return 2300 if bounce and paid and not bounced and objects[source]['zone'] == 'Battlefield' and any(v['source_card'] == name and isinstance(v['kind'], dict) and 'TriggeredAbility' in v['kind'] for v in o['stack']) else -1000
                        return -1000
                    if kind == 'ChooseTarget':
                        target = action['target'].get('Card')
                        spell = o['announcing']['source_card'] if o['announcing'] else None
                        return 2300 if spell == 'Stab' and any(v['id'] == target for v in lions) or spell == 'Unsummon' and target == source else -1000
                    if kind == 'ChooseEndure':
                        return 2500 if action['put_counters'] == counters else -1000
                    if kind == 'PayCosts':
                        return 2400
                    if kind == 'PayResolutionMana':
                        return -1000 if decline else 2400
                    if kind == 'DeclineResolutionMana':
                        return 2500 if decline else -1000
                    if kind == 'TapMana':
                        if o['decision'] not in ('pay_costs', 'resolution_mana'):
                            return -1000
                        return 1900 if objects[action['card']]['card'] == 'Island' else 1950
                    if kind == 'Attack':
                        return 1800 if actor == player and paid and not attacked and i in (1, 4, 7) and action['card'] == source else -1000
                    if kind == 'PlayLand':
                        card = objects[action['card']]['card']
                        copies = sum(v['card'] == card for v in lands) if actor == player else 0
                        return 1500 - 50 * copies
                    return {'Keep': 3000, 'Mulligan': -10000, 'Concede': -10000, 'Pass': 0, 'FinishAttackers': 1000, 'FinishBlockers': 1000, 'FinishTargets': 2100, 'PutTrigger': 2200, 'Discard': 2100}.get(kind, -1000)
                index = max(range(len(o['actions'])), key=lambda j: score(o['actions'][j]))
                action = o['actions'][index]
                self.assertGreaterEqual(score(action), 0, (name, o['decision'], o['actions']))
                if action['kind'] == 'PayCosts' and o['announcing']:
                    cast = o['announcing']['source_card']
                    if cast == name:
                        paid, source = True, o['announcing']['card']
                    if cast == 'Stab':
                        killed = True
                    if cast == 'Unsummon':
                        bounced = True
                if action['kind'] == 'Attack':
                    attacked = True
                if action['kind'] == 'ChooseEndure':
                    choices += 1
                if action['kind'] == 'DeclineResolutionMana':
                    declined = True
                after = self.replay(game, o, index)
                self.assertEqual(after['schema'], 74)
            self.fail(str((name, player, paid, source, attacked, killed, bounced, choices)))
