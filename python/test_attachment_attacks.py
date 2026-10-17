"""Public in-process witnesses for owned and creature-granted Equipment attacks."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError, Game

NAMES = ['Baseball Bat', "Captain America's Shield", 'Stormbeacon Blade', 'Thunder Lasso', 'Ultima Weapon', 'Candlestick', 'Diamond Pick-Axe', "Medic's Kitesail", 'Pirate Hat', 'Web-Shooters']
BONUS = [(1, 1), (0, 8), (3, 0), (1, 1), (7, 7), (1, 1), (1, 1), (1, 0), (1, 1), (1, 1)]
EQUIP = [3, 2, 2, 2, 7, 2, 2, 2, 2, 2]
FACTS = {c['name']: c for c in json.loads((Path(__file__).resolve().parents[1] / 'data/standard/cards.json').read_text())['cards']}
BASICS = ['Plains', 'Island', 'Swamp', 'Mountain', 'Forest']


def printed_cost(name):
    required = [0] * 6
    generic = 0
    for symbol in FACTS[name]['mana_cost'].replace('}', '').split('{')[1:]:
        if symbol.isdigit(): generic += int(symbol)
        else: required['WUBRGC'.index(symbol)] += 1
    return required, generic


class AttachmentAttackTests(unittest.TestCase):
    def replay(self, game, obs, index):
        saved = game.snapshot()
        with game.clone() as fork: expected = fork.step(fork.observe(obs['actor']), index)
        self.assertEqual(game.step(obs, index), expected)
        game.restore(saved)
        with self.assertRaisesRegex(EngineError, 'StaleRevision'): game.step(obs, index)
        after = game.step(game.observe(obs['actor']), index)
        expected['revision'] = after['revision']
        self.assertEqual(after, expected)
        game.drop_snapshot(saved)
        return after

    def test_all_ten_complete_bodies_both_sources_controllers_and_printed_residuals(self):
        for i, name in enumerate(NAMES):
            for seat in ('P0', 'P1'):
                variants = (False, True) if i in (0, 2, 8) else (True,)
                for variant in variants:
                    with self.subTest(card=name, seat=seat, variant=variant):
                        self.scenario(i, seat, variant)

    def scenario(self, i, seat, variant):
        name = NAMES[i]
        other = 'P1' if seat == 'P0' else 'P0'
        creature = 'Three Tree Mascot' if i == 8 and variant else 'Bear Cub'
        need = 3 if i == 2 and variant else 2 if i in (0, 2, 3) else 1
        lands = ['Plains'] * 16 + ['Forest'] * 8 + ['Island'] * 8 + ['Mountain'] * 8
        deck = lands + [name] * 12 + [creature] * 20
        foe = ['Forest'] * 32 + ['Bear Cub'] * 28
        decks = [deck, foe] if seat == 'P0' else [foe, deck]
        equip_index = 1 if i == 5 or i == 8 and not variant else 0
        cost = 1 if i == 8 and variant else EQUIP[i]
        with Game(decks, seed=17, starting_player=seat, deck_rules='Freeform') as game:
            cast_paid = equip_paid = attached = attack_done = treasure_used = sac_draw = False
            chosen_attackers = set()
            root = baseline = None
            surveil = None
            recipient_ref = None
            for _ in range(24000):
                obs = game.observe('P0')
                self.assertIsNone(obs['outcome'], name)
                obs = game.observe(obs['actor'])
                actor = obs['actor']
                objects = {v['id']: v for v in obs['objects']}
                equipment = next((v for v in objects.values() if v['card'] == name and v['owner'] == seat and v['zone'] == 'Battlefield'), None)
                friends = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == creature]
                foes = [v for v in objects.values() if v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub']
                if equipment:
                    recipient_ref = equipment['attached_to']
                    recipient = next((v for v in friends if v['reference'] == recipient_ref), None)
                    if recipient:
                        base = FACTS[creature]
                        self.assertEqual((recipient['power'], recipient['toughness']), (int(base['power']) + BONUS[i][0], int(base['toughness']) + BONUS[i][1]))
                        self.assertEqual(bool(recipient['keywords'] & 8), i == 1)
                        self.assertEqual(bool(recipient['keywords'] & 1), i == 7)
                        self.assertEqual(bool(recipient['keywords'] & 2), i == 9)
                        self.assertFalse(recipient['keywords'] & 8192)
                        if equip_paid: attached = True
                    self.assertEqual(bool(equipment['keywords'] & 8192), i in (1, 6))
                ann = obs['announcing']
                for packet in obs['stack'] + obs['waiting_triggers'] + ([ann] if ann else []) + ([obs['resolving']] if obs['resolving'] else []):
                    event = packet['attachment_trigger']
                    if event and event['granting_card'] == name:
                        self.assertEqual(event['granted'], i >= 5)
                        self.assertEqual(event['defender'], other)
                        self.assertEqual(packet['controller'], seat)
                        self.assertIsNone(packet['cast'])
                        self.assertEqual(packet['object'], event['recipient'] if i >= 5 else event['grantor'])
                        self.assertEqual(packet['card'], packet['object']['card'])
                        self.assertEqual(packet['source_card'], creature if i >= 5 else name)
                        self.assertEqual(event['recipient'], recipient_ref)
                        self.assertEqual(event['grantor'], equipment['reference'])
                        self.assertEqual(event['granting_card'], name)
                        self.assertEqual(objects[event['recipient']['card']]['tapped'], i != 1)
                frame = obs['library_choice']
                if frame:
                    self.assertEqual(i, 5)
                    self.assertEqual(actor, seat)
                    self.assertIsNone(game.observe(other)['library_choice'])
                    self.assertEqual(game.observe(other)['actions'], [])
                    if surveil is None:
                        self.assertEqual(len(frame['remaining']), 2)
                        surveil = (frame['remaining'][:], obs['players'][int(seat[1])]['library_count'], sum(v['zone'] == 'Graveyard' and v['owner'] == seat for v in objects.values()))
                if obs['decision'] == 'effect_discard':
                    self.assertEqual(i, 8)
                    private = game.observe(other)
                    self.assertEqual(private['actions'], [])
                    self.assertFalse(any(v['owner'] == seat and v['zone'] == 'Hand' for v in private['objects']))
                def score(a):
                    k = a['kind']
                    if k in ('Cast', 'AnnounceCast'):
                        n = objects[a['card']]['card']
                        if actor != seat: return 600 if n == 'Bear Cub' and len(foes) < 3 else -1000
                        if n == name: return 850 if equipment is None and not cast_paid and len(friends) >= need and k == 'AnnounceCast' else -1000
                        return 700 if n == creature and len(friends) < need else -1000
                    if k in ('Activate', 'BeginActivate', 'AnnounceActivate'):
                        if actor != seat or objects[a['card']]['card'] != name: return -1000
                        if i == 5 and a['ability'] == 0: return 1200 if attack_done and not sac_draw and k == 'AnnounceActivate' else -1000
                        return 1100 if not equip_paid and a['ability'] == equip_index and k == 'AnnounceActivate' else -1000
                    if k in ('ActivateMana', 'AnnounceMana'):
                        v = objects[a['card']]
                        return 1400 if i == 6 and actor == seat and attack_done and obs['decision'] == 'priority' and v['token'] and v['token']['subtypes'] == 'Treasure' else -1000
                    if k == 'ChooseMana': return 1300
                    if k == 'ChooseTarget' and 'Card' in a['target']:
                        v = objects[a['target']['Card']]
                        if ann and ann['kind'] == 'Spell': return -1000
                        if ann and isinstance(ann['kind'], dict) and 'ActivatedAbility' in ann['kind']:
                            return 1200 if v['owner'] == seat and v['card'] == creature and (i not in (0, 3) or v['reference'] != equipment['attached_to']) else -1000
                        if ann and ann['attachment_trigger']: return 1300 if v['owner'] == other and not (i == 0 and not variant) else -1000
                        return 900 if v['owner'] == seat and v['card'] == creature else -1000
                    if k == 'Attack':
                        v = objects[a['card']]
                        ready = attached and len(friends) >= need and (not (i in (0, 1, 3, 4, 9)) or foes)
                        if actor != seat or not ready or attack_done or len(chosen_attackers) >= need: return -1000
                        return 1000 if v['reference'] == recipient_ref else 800
                    if k == 'PlayLand':
                        n = objects[a['card']]['card']
                        count = sum(v['owner'] == actor and v['zone'] == 'Battlefield' and v['card'] == n for v in objects.values())
                        return 1000 - count
                    if k == 'ArrangeLibrary': return 1300 if a['destination'] == ('Graveyard' if len(frame['remaining']) == 2 else 'Top') else -1000
                    if k == 'TapMana': return 800 if obs['decision'] == 'pay_costs' else -1000
                    if k == 'Discard': return 1000 if objects[a['card']]['card'] in BASICS else 900
                    return {'Keep': 1500, 'PayCosts': 1400, 'PutTrigger': 1200, 'FinishTargets': 1000, 'FinishAttackers': 500, 'FinishBlockers': 500, 'Pass': 0, 'Concede': -10000, 'Mulligan': -1000, 'Block': -1000}.get(k, 100)
                j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                action = obs['actions'][j]
                if action['kind'] == 'Attack': chosen_attackers.add(action['card'])
                if action['kind'] == 'FinishAttackers': chosen_attackers.clear()
                if ann and ann['source_card'] == name and obs['decision'] == 'pay_costs':
                    want = printed_cost(name) if ann['kind'] == 'Spell' else ([0] * 6, 2 if i == 5 and ann['kind']['ActivatedAbility']['ability'] == 0 else cost)
                    self.assertEqual((obs['payment_request']['required'], obs['payment_request']['generic']), want)
                    if action['kind'] == 'PayCosts':
                        self.assertEqual(sum(action['payment']), sum(want[0]) + want[1])
                        if ann['kind'] == 'Spell':
                            self.assertEqual(ann['cast'], {'physical_card': None, 'face': 'Normal', 'payment': [0]*6, 'origin': 'Hand', 'method': 'Normal', 'offspring': None, 'kicker': None, 'bargain': None, 'evidence': None, 'flash_surcharge': False})
                            cast_paid = True
                        elif not (i == 5 and ann['kind']['ActivatedAbility']['ability'] == 0): equip_paid = True
                if obs['decision'] == 'targets':
                    for bad in [dict(action, target={'Card': 2**32 - 1})] if action['kind'] == 'ChooseTarget' else [{'kind': 'ChooseTarget', 'target': {'Card': 2**32 - 1}}]:
                        with self.assertRaises(EngineError): game.act(obs, bad)
                        self.assertEqual(game.observe(actor), obs)
                    with self.assertRaises(EngineError): game.act(dict(obs, actor='P1' if actor == 'P0' else 'P0'), action)
                    self.assertEqual(game.observe(actor), obs)
                    alternative = next((a for a in obs['actions'] if a['kind'] == 'ChooseTarget' and a != action), None)
                    if alternative:
                        with game.clone() as fork: fork.act(fork.observe(actor), alternative)
                        self.assertEqual(game.observe(actor), obs)
                if i == 8 and not variant and action['kind'] == 'AnnounceActivate' and action['ability'] == 1:
                    with self.assertRaisesRegex(EngineError, 'IllegalAction'): game.act(obs, dict(action, ability=0))
                    self.assertEqual(game.observe(actor), obs)
                if action['kind'] == 'Pass' and obs['stack']:
                    packet = obs['stack'][-1]
                    if packet['attachment_trigger'] and packet['attachment_trigger']['granting_card'] == name:
                        root, baseline = packet, obs
                    elif i == 5 and packet['source_card'] == name and isinstance(packet['kind'], dict) and packet['kind']['ActivatedAbility']['ability'] == 0:
                        root, baseline = packet, obs
                after = self.replay(game, obs, j)
                self.assertEqual(after['schema'], 91)
                if frame:
                    window, library_count, grave_count = surveil
                    if after['library_choice']:
                        self.assertEqual(after['players'][int(seat[1])]['library_count'], library_count)
                        self.assertEqual(sum(v['zone'] == 'Graveyard' and v['owner'] == seat for v in after['objects']), grave_count)
                        self.assertFalse(any(v['reference'] in [x['reference'] for x in window] for v in after['objects']))
                    else:
                        self.assertEqual(after['players'][int(seat[1])]['library_count'], library_count - 1)
                        buried = next(v for v in after['objects'] if v['id'] == window[0]['id'])
                        self.assertEqual(buried['zone'], 'Graveyard')
                        self.assertEqual(buried['reference']['generation'], window[0]['reference']['generation'] + 1)
                if i == 6 and attack_done:
                    treasures = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['token'] and v['token']['subtypes'] == 'Treasure']
                    if any(not any(w['reference'] == v['reference'] for w in after['objects']) for v in treasures):
                        self.assertEqual(sum(after['players'][int(seat[1])]['mana']) - sum(obs['players'][int(seat[1])]['mana']), 1)
                        treasure_used = True
                if root and not any(p['id'] == root['id'] for p in after['stack']) and not (after['resolving'] and after['resolving']['id'] == root['id']):
                    owner = int(seat[1])
                    if root['attachment_trigger']:
                        event = root['attachment_trigger']
                        if i == 0 and not variant:
                            self.assertEqual(root['targets'], [])
                            self.assertTrue(all(not v['tapped'] for v in after['objects'] if v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub'))
                        elif i in (0, 1, 3, 9):
                            target = root['targets'][0]['object']
                            self.assertTrue(next(v for v in after['objects'] if v['reference'] == target)['tapped'])
                        if i == 2: self.assertEqual(after['players'][owner]['cards_drawn_this_turn'] - baseline['players'][owner]['cards_drawn_this_turn'], int(variant))
                        if i == 4: self.assertEqual(next(v for v in after['objects'] if v['id'] == root['targets'][0]['object']['card'])['zone'], 'Graveyard')
                        if i == 5: self.assertIsNotNone(surveil)
                        if i == 6: self.assertTrue(any(v['owner'] == seat and v['zone'] == 'Battlefield' and v['token'] and v['token']['subtypes'] == 'Treasure' for v in after['objects']))
                        if i == 7: self.assertEqual(after['players'][owner]['life'], baseline['players'][owner]['life'] + 1)
                        if i == 8:
                            self.assertEqual(after['players'][owner]['cards_drawn_this_turn'], baseline['players'][owner]['cards_drawn_this_turn'] + 1)
                            self.assertEqual(after['players'][owner]['cards_discarded_this_turn'], baseline['players'][owner]['cards_discarded_this_turn'] + 1)
                            self.assertEqual(after['players'][owner]['hand_count'], baseline['players'][owner]['hand_count'])
                        attack_done = True
                    else:
                        self.assertEqual(after['players'][owner]['cards_drawn_this_turn'], baseline['players'][owner]['cards_drawn_this_turn'] + 1)
                        self.assertEqual(next(v for v in after['objects'] if v['card'] == name)['zone'], 'Graveyard')
                        sac_draw = True
                    root = baseline = None
                if attack_done and (i != 5 or sac_draw) and (i != 6 or treasure_used):
                    self.assertTrue(cast_paid and equip_paid and attached)
                    break
            else: self.fail(f'no complete witness for {name}')


    def test_zero_one_and_duplicate_grants_coexist_with_the_creatures_printed_ability(self):
        for seat in ('P0', 'P1'):
            for count in (0, 1, 2):
                with self.subTest(seat=seat, grants=count):
                    deck = ['Swamp'] * 32 + ['Pulse Tracker'] * 12 + ["Medic's Kitesail"] * 16
                    decks = [deck, ['Forest'] * 60] if seat == 'P0' else [['Forest'] * 60, deck]
                    other = 'P1' if seat == 'P0' else 'P0'
                    with Game(decks, seed=23, starting_player=seat, deck_rules='Freeform') as game:
                        seen, resolved, grantors = {}, set(), set()
                        attacked = False
                        complete = False
                        for _ in range(16000):
                            obs = game.observe('P0')
                            self.assertIsNone(obs['outcome'])
                            obs = game.observe(obs['actor'])
                            objects = {v['id']: v for v in obs['objects']}
                            tracker = next((v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == 'Pulse Tracker'), None)
                            equipment = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield' and v['card'] == "Medic's Kitesail"]
                            attached = [v for v in equipment if tracker and v['attached_to'] == tracker['reference']]
                            if tracker:
                                self.assertEqual((tracker['power'], tracker['toughness']), (1 + len(attached), 1))
                                self.assertEqual(bool(tracker['keywords'] & 1), bool(attached))
                            packets = obs['stack'] + obs['waiting_triggers'] + ([obs['announcing']] if obs['announcing'] else []) + ([obs['resolving']] if obs['resolving'] else [])
                            for packet in packets:
                                if tracker and packet['source_card'] == 'Pulse Tracker' and packet['kind'] != 'Spell':
                                    self.assertEqual(packet['object'], tracker['reference'])
                                    self.assertEqual(packet['controller'], seat)
                                    seen[packet['id']] = packet
                                    event = packet['attachment_trigger']
                                    if event:
                                        self.assertEqual(event['granting_card'], "Medic's Kitesail")
                                        self.assertTrue(event['granted'])
                                        self.assertEqual(event['recipient'], tracker['reference'])
                                        self.assertEqual(event['defender'], other)
                                        grantors.add(tuple(event['grantor'].values()))
                            def score(a):
                                kind = a['kind']
                                if kind in ('Cast', 'AnnounceCast'):
                                    if obs['actor'] != seat: return -1000
                                    name = objects[a['card']]['card']
                                    if name == 'Pulse Tracker': return 850 if tracker is None and kind == 'AnnounceCast' else -1000
                                    return 800 if name == "Medic's Kitesail" and len(equipment) < count and kind == 'AnnounceCast' else -1000
                                if kind in ('Activate', 'AnnounceActivate', 'BeginActivate'):
                                    return 1200 if tracker and objects[a['card']]['card'] == "Medic's Kitesail" and objects[a['card']]['attached_to'] != tracker['reference'] and kind == 'AnnounceActivate' else -1000
                                if kind == 'ChooseTarget': return 1300 if tracker and a['target'] == {'Card': tracker['id']} else -1000
                                if kind == 'Attack': return 1000 if tracker and a['card'] == tracker['id'] and len(attached) == count and not attacked else -1000
                                if kind == 'PlayLand': return 950
                                if kind == 'TapMana': return 900 if obs['decision'] == 'pay_costs' else -1000
                                return {'Keep': 1500, 'PayCosts': 1400, 'PutTrigger': 1300, 'Pass': 0, 'FinishAttackers': 500, 'FinishBlockers': 500, 'Concede': -10000, 'Block': -1000, 'Mulligan': -1000, 'ActivateMana': -1000, 'AnnounceMana': -1000}.get(kind, 100)
                            j = max(range(len(obs['actions'])), key=lambda k: score(obs['actions'][k]))
                            if obs['actions'][j]['kind'] == 'Attack': attacked = True
                            after = self.replay(game, obs, j)
                            live = {p['id'] for p in after['stack'] + after['waiting_triggers'] + ([after['announcing']] if after['announcing'] else []) + ([after['resolving']] if after['resolving'] else [])}
                            resolved |= set(seen) - live
                            if len(resolved) == count + 1:
                                self.assertEqual(len(seen), count + 1)
                                self.assertEqual(sum(p['attachment_trigger'] is None for p in seen.values()), 1)
                                self.assertEqual(len(grantors), count)
                                self.assertEqual(after['players'][int(seat[1])]['life'], 20 + count)
                                self.assertEqual(after['players'][int(other[1])]['life'], 19)
                                complete = True
                                break
                        self.assertTrue(complete)


if __name__ == '__main__':
    unittest.main()
