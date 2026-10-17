"""Complete paid/declined kicker bodies through owned public library calls."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError, Game

NAMES = ['Burst Lightning', 'Firebending Lesson', 'Gatekeeper of Malakir', 'Grow from the Ashes', 'Into the Roil', 'Whoosh!', 'Nullpriest of Oblivion', 'Sun-Blessed Healer', "Zuko's Conviction", 'Gnarlid Colony']
FACTS = {c['name']: c for c in json.loads((Path(__file__).resolve().parents[1] / 'data/standard/cards.json').read_text())['cards']}
BASICS = ['Plains', 'Island', 'Swamp', 'Mountain', 'Forest']


def requirements(name, pay):
    symbols = FACTS[name]['mana_cost'] + (FACTS[name]['oracle_text'].splitlines()[0].split(' ')[1] if pay else '')
    required, generic = [0] * 6, 0
    for symbol in symbols.replace('}', '').split('{')[1:]:
        if symbol.isdigit(): generic += int(symbol)
        else: required['WUBRGC'.index(symbol)] += 1
    return required, generic


class FixedKickerTests(unittest.TestCase):
    def replay(self, game, obs, index):
        saved = game.snapshot()
        with game.clone() as fork: expected = fork.step(fork.observe(obs['actor']), index)
        game.step(obs, index)
        game.restore(saved)
        with self.assertRaisesRegex(EngineError, 'StaleRevision'): game.step(obs, index)
        after = game.step(game.observe(obs['actor']), index)
        expected['revision'] = after['revision']
        self.assertEqual(after, expected)
        game.drop_snapshot(saved)
        return after

    def test_ten_complete_bodies_both_kicker_branches_and_controllers(self):
        for i, name in enumerate(NAMES):
            for pay in (False, True):
                for seat in ('P0', 'P1'):
                    with self.subTest(card=name, kicked=pay, seat=seat):
                        self.scenario(i, pay, seat)

    def scenario(self, i, pay, seat):
        name = NAMES[i]
        other = 'P1' if seat == 'P0' else 'P0'
        owner = int(seat[1])
        deck = sum(([n] * 10 for n in BASICS), []) + [name] * 12 + ['Bear Cub'] * 16 + ['Thrill of Possibility'] * 12
        foe = ['Forest'] * 40 + ['Bear Cub'] * 20
        decks = [deck, foe] if seat == 'P0' else [foe, deck]
        need_grave = i in (6, 7, 8)
        selected = paid = resolved = False
        root = baseline = None
        grave_ref = source_ref = None
        search_selected = []
        search_library = None
        with Game(decks, seed=43, starting_player=seat, deck_rules='Freeform') as game:
            for _ in range(40000):
                obs = game.observe('P0')
                self.assertIsNone(obs['outcome'], name)
                obs = game.observe(obs['actor'])
                actor = obs['actor']
                objects = {v['id']: v for v in obs['objects']}
                friends = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub']
                foes = [v for v in objects.values() if v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub']
                grave = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Graveyard' and v['card'] == 'Bear Cub']
                hand_bear = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Hand' and v['card'] == 'Bear Cub']
                required, generic = requirements(name, pay)
                potential = obs['players'][owner]['mana'][:]
                for v in objects.values():
                    if v['zone'] == 'Battlefield' and v['owner'] == seat and v['card'] in BASICS and not v['tapped']:
                        potential[['Plains', 'Island', 'Swamp', 'Mountain', 'Forest'].index(v['card'])] += 1
                enough = all(a >= b for a, b in zip(potential, required)) and sum(potential) >= sum(required) + generic
                ready = enough and (not need_grave or grave) and (i not in (1, 2, 4, 5) or foes) and (i != 9 or friends)
                ann = obs['announcing']
                if obs['decision'] == 'kicker':
                    self.assertEqual(actor, seat)
                    self.assertIsNone(obs['payment_request'])
                    self.assertEqual(ann['targets'], [])
                    self.assertIsNone(ann['cast']['kicker'])
                    self.assertIsNone(ann['cast']['offspring'])
                    self.assertEqual(game.observe(other)['actions'], [])
                    self.assertTrue(any(a['kind'] == 'ChooseKicker' and a['pay'] == pay for a in obs['actions']))
                    saved = game.observe(actor)
                    with self.assertRaises(EngineError): game.act(dict(obs, actor=other), {'kind': 'ChooseKicker', 'pay': pay})
                    self.assertEqual(game.observe(actor), saved)
                if ann and ann['source_card'] == name and obs['decision'] == 'pay_costs':
                    self.assertEqual((obs['payment_request']['required'], obs['payment_request']['generic']), (required, generic))
                    self.assertEqual(ann['cast'], {'payment': [0]*6, 'origin': 'Hand', 'method': 'Normal', 'offspring': None, 'kicker': pay, 'bargain': None, 'evidence': None})
                    source_ref = ann['object']
                    if grave: grave_ref = grave[0]['reference']
                if obs['search_choice']:
                    self.assertEqual(i, 3)
                    search = obs['search_choice']
                    self.assertEqual(search['maximum'], 2 if pay else 1)
                    self.assertFalse(search['tapped'])
                    self.assertEqual(search['destination'], 'Battlefield')
                    self.assertIsNone(game.observe(other)['search_choice'])
                    self.assertEqual(game.observe(other)['actions'], [])
                    if search_library is None: search_library = obs['players'][owner]['library_count']
                    self.assertEqual(obs['players'][owner]['library_count'], search_library)
                    self.assertEqual(search['selected'], search_selected)
                    for card in search_selected: self.assertNotIn(card, objects)
                def score(a):
                    k = a['kind']
                    if k in ('Cast', 'AnnounceCast'):
                        n = objects[a['card']]['card']
                        if actor != seat: return 700 if n == 'Bear Cub' and len(foes) < 2 else -1000
                        if n == name: return 1000 if ready and not paid and k == 'AnnounceCast' else -1000
                        if n == 'Thrill of Possibility': return 950 if need_grave and not grave and hand_bear and k == 'AnnounceCast' else -1000
                        return 700 if n == 'Bear Cub' and len(friends) < 1 and not (need_grave and not grave) else -1000
                    if k == 'ChooseKicker': return 1500 if a['pay'] == pay else -1000
                    if k == 'ChooseTarget':
                        t = a['target']
                        if 'Player' in t: return 1500 if i == 0 and t['Player'] == other else 1200 if t['Player'] == other else -1000
                        if 'Card' in t:
                            v = objects[t['Card']]
                            if i in (6, 7, 8): return 1400 if v['owner'] == seat and v['card'] == 'Bear Cub' and v['zone'] == 'Graveyard' else -1000
                            return 1300 if v['owner'] == other and v['card'] == 'Bear Cub' else -1000
                    if k == 'ChooseAdditionalCost': return 1500 if objects[a['object']['card']]['card'] == 'Bear Cub' else -1000
                    if k == 'PlayLand':
                        n = objects[a['card']]['card']
                        count = sum(v['zone'] == 'Battlefield' and v['owner'] == actor and v['card'] == n for v in objects.values())
                        return 1200 - count * 10 + int(actor == seat and required[BASICS.index(n)] > count) * 100
                    if k == 'TapMana': return 800 if obs['decision'] == 'pay_costs' else -1000
                    if k == 'ChooseSearch': return 1300
                    if k == 'Sacrifice': return 1300
                    if k == 'Discard': return 1000 if objects[a['card']]['card'] in BASICS else 900
                    return {'Keep': 1600, 'PayCosts': 1500, 'PutTrigger': 1400, 'FinishTargets': 1100, 'FinishSearch': 900, 'FinishAttackers': 500, 'FinishBlockers': 500, 'Pass': 0, 'Attack': -1000, 'Block': -1000, 'Concede': -10000, 'Mulligan': -1000}.get(k, -1000)
                j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                action = obs['actions'][j]
                if action['kind'] == 'ChooseKicker': selected = True
                if action['kind'] == 'PayCosts' and ann and ann['source_card'] == name:
                    self.assertEqual(sum(action['payment']), sum(required) + generic)
                    paid = True
                if action['kind'] == 'ChooseSearch': search_selected.append(action['card'])
                if action['kind'] == 'Pass' and obs['stack']:
                    packet = obs['stack'][-1]
                    if packet['source_card'] == name:
                        root, baseline = packet, obs
                after = self.replay(game, obs, j)
                self.assertEqual(after['schema'], 76)
                if root and not any(p['id'] == root['id'] for p in after['stack']) and not (after['resolving'] and after['resolving']['id'] == root['id']):
                    if root['kind'] != 'Spell':
                        self.assertTrue(pay)
                        if i == 2: self.assertEqual(sum(v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub' for v in after['objects']), len(foes) - 1)
                        if i in (6, 7): self.assertEqual(next(v for v in after['objects'] if v['id'] == grave_ref['card'])['zone'], 'Battlefield')
                        resolved = True
                    elif i in (2, 6, 7):
                        if not pay:
                            self.assertFalse(any(p['source_card'] == name for p in after['stack'] + after['waiting_triggers']))
                            if need_grave: self.assertEqual(next(v for v in after['objects'] if v['id'] == grave_ref['card'])['zone'], 'Graveyard')
                            resolved = True
                    else:
                        if i == 0: self.assertEqual(after['players'][int(other[1])]['life'], baseline['players'][int(other[1])]['life'] - (4 if pay else 2))
                        if i == 1: self.assertEqual(next(v for v in after['objects'] if v['id'] == root['targets'][0]['object']['card'])['zone'], 'Graveyard')
                        if i == 3:
                            self.assertEqual(len(search_selected), 2 if pay else 1)
                            for card in search_selected:
                                v = next(v for v in after['objects'] if v['id'] == card)
                                self.assertEqual(v['zone'], 'Battlefield'); self.assertFalse(v['tapped'])
                        if i in (4, 5):
                            self.assertEqual(after['players'][owner]['cards_drawn_this_turn'] - baseline['players'][owner]['cards_drawn_this_turn'], int(pay))
                            self.assertEqual(next(v for v in after['objects'] if v['id'] == root['targets'][0]['object']['card'])['zone'], 'Hand')
                        if i == 8:
                            v = next(v for v in after['objects'] if v['id'] == grave_ref['card'])
                            self.assertEqual(v['zone'], 'Battlefield' if pay else 'Hand'); self.assertEqual(v['tapped'], pay)
                            self.assertEqual(v['reference']['generation'], grave_ref['generation'] + 1)
                        if i == 9:
                            v = next(v for v in after['objects'] if v['id'] == source_ref['card'])
                            self.assertEqual(v['reference']['generation'], source_ref['generation'] + 1)
                            self.assertEqual(v['counters']['plus_one'], 2 if pay else 0)
                            self.assertEqual(v['power'], 4 if pay else 2)
                            self.assertEqual(bool(v['keywords'] & 256), pay)
                            self.assertTrue(all(not (v['keywords'] & 256) for v in after['objects'] if v['owner'] == seat and v['card'] == 'Bear Cub'))
                        resolved = True
                    root = baseline = None
                if selected and paid and resolved:
                    if i in (6, 7):
                        v = next(v for v in after['objects'] if v['id'] == source_ref['card'])
                        self.assertTrue(v['keywords'] & 64)
                        self.assertEqual(bool(v['keywords'] & 32768), i == 6)
                    break
            else: self.fail('no full kicker witness: ' + name)


    def test_real_paid_impulse_spell_casts_kicked_from_exile_in_both_seats(self):
        for seat in ('P0', 'P1'):
            with self.subTest(seat=seat):
                other = 'P1' if seat == 'P0' else 'P0'
                deck = ['Mountain'] * 40 + ["Alania's Pathmaker"] * 12 + ['Burst Lightning'] * 20
                decks = [deck, ['Mountain'] * 60] if seat == 'P0' else [['Mountain'] * 60, deck]
                paid = False
                root = baseline = None
                with Game(decks, seed=43, starting_player=seat, deck_rules='Freeform') as game:
                    for _ in range(16000):
                        obs = game.observe('P0')
                        self.assertIsNone(obs['outcome'])
                        obs = game.observe(obs['actor'])
                        actor = obs['actor']
                        objects = {v['id']: v for v in obs['objects']}
                        available = obs['players'][int(seat[1])]['mana'][3] + sum(v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == 'Mountain' and not v['tapped'] for v in objects.values())
                        total_lands = sum(v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == 'Mountain' for v in objects.values())
                        ann = obs['announcing']
                        def score(a):
                            k = a['kind']
                            if k == 'AnnounceExile': return 1500 if actor == seat and objects[a['card']]['card'] == 'Burst Lightning' and available >= 5 and not paid else -1000
                            if k in ('Cast', 'AnnounceCast'): return 1000 if actor == seat and objects[a['card']]['card'] == "Alania's Pathmaker" and total_lands >= 5 else -1000
                            if k == 'ChooseKicker': return 1500 if a['pay'] else -1000
                            if k == 'ChooseTarget': return 1400 if a['target'] == {'Player': other} else -1000
                            if k == 'PlayLand': return 1200
                            if k == 'TapMana': return 800 if obs['decision'] == 'pay_costs' else -1000
                            if k == 'Discard': return 900 if objects[a['card']]['card'] == 'Mountain' else 800
                            return {'Keep': 1600, 'PayCosts': 1500, 'PutTrigger': 1400, 'FinishTargets': 1100, 'Pass': 0, 'FinishAttackers': 500, 'FinishBlockers': 500}.get(k, -1000)
                        j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                        action = obs['actions'][j]
                        if action['kind'] == 'PayCosts' and ann and ann['source_card'] == 'Burst Lightning':
                            self.assertEqual(ann['cast'], {'payment': [0]*6, 'origin': 'Exile', 'method': 'Normal', 'offspring': None, 'kicker': True, 'bargain': None, 'evidence': None})
                            self.assertEqual(obs['payment_request']['required'], [0, 0, 0, 1, 0, 0])
                            self.assertEqual(obs['payment_request']['generic'], 4)
                            self.assertEqual(sum(action['payment']), 5)
                            self.assertFalse(any(v['object'] == ann['object'] for v in obs['exile_permissions']))
                            paid = True
                        if action['kind'] == 'Pass' and obs['stack'] and obs['stack'][-1]['source_card'] == 'Burst Lightning':
                            root, baseline = obs['stack'][-1], obs
                        after = self.replay(game, obs, j)
                        if root and not any(v['id'] == root['id'] for v in after['stack']):
                            self.assertTrue(paid)
                            self.assertEqual(after['players'][int(other[1])]['life'], baseline['players'][int(other[1])]['life'] - 4)
                            self.assertEqual(next(v for v in after['objects'] if v['id'] == root['card'])['zone'], 'Graveyard')
                            break
                    else: self.fail('no actual paid kicker cast from impulse exile')


if __name__ == '__main__':
    unittest.main()
