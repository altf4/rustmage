"""Public target-qualified cost proposals, actual payments, effects and replay."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError, Game

# Printed mana, discount and facts are an independent Oracle ledger, not native recipes.
NAMES = ["Ajani's Response", 'Brush Off', 'Dark Endurance', 'Depower', 'Dire Downdraft',
         'Fate of the Sun-Cryst', 'Grounded for Life', 'Grow Extra Arms', 'Luminous Rebuke',
         'Magnificent End', 'Out of Air', 'Quicksand Whirlpool', "Ride's End", 'Roadside Blowout',
         'Run Behind', 'Run Over', 'Seized from Slumber', 'Swampsnare Trap', 'Uneasy Partings']
COLORS = [0, 1, 2, 1, 1, 0, 0, 4, 0, 0, 1, 0, 0, 1, 1, 4, 0, 2, 1]
GENERIC = [4, 2, 1, 2, 3, 4, 4, 1, 4, 4, 2, 5, 4, 2, 3, 1, 4, 2, 3]
DISCOUNT = [3, 1, 1, 2, 1, 2, 3, 1, 3, 3, 2, 3, 3, 2, 1, 1, 3, 1, 1]
FACTS = {c['name']: c for c in json.loads((Path(__file__).resolve().parents[1] / 'data/standard/cards.json').read_text())['cards']}
BASICS = ['Plains', 'Island', 'Swamp', 'Mountain', 'Forest']


def kinds(name):
    return {'Creature'} if name.startswith('@token/') else set(FACTS[name]['type_line'].split(' — ')[0].split())


def subtype(name, wanted):
    c = FACTS[name]
    printed = c['type_line'].split(' — ')[1].split() if ' — ' in c['type_line'] else []
    return wanted in printed or ('Changeling' in c.get('keywords', []) and wanted in ('Mount', 'Spider'))


def qualifies(i, actor, target, obs):
    if 'Spell' in target:
        s = next((s for s in obs['stack'] if s['card'] == target['Spell'] and s['kind'] == 'Spell'), None)
        return bool(s and (bool(kinds(s['source_card']) & {'Instant', 'Sorcery'}) if i == 1 else 'Creature' in kinds(s['source_card']) if i == 10 else False))
    v = next((v for v in obs['objects'] if v['id'] == target.get('Card') and v['zone'] == 'Battlefield'), None)
    if v is None:
        return False
    creature = 'Creature' in kinds(v['card'])
    if i in (0, 5, 6, 8, 9, 11, 16): return creature and v['tapped']
    if i == 2: return creature and v['id'] in obs['blocking']
    if i in (3, 14): return creature and v['id'] in obs['attackers']
    if i == 4: return creature and (v['tapped'] or v['id'] in obs['attackers'])
    if i == 7: return subtype(v['card'], 'Spider')
    if i == 12: return v['tapped']
    if i == 13:
        symbols = FACTS[v['card']].get('mana_cost', '').replace('}', '').split('{')[1:]
        return sum(int(s) if s.isdigit() else 1 for s in symbols) == 1
    if i == 15: return v['owner'] == actor and (subtype(v['card'], 'Mount') or subtype(v['card'], 'Vehicle'))
    if i == 17: return creature and int(v['keywords']) & 1 != 0
    if i == 18: return creature and v['id'] in obs['attackers'] and v['token'] is None
    return False


def expected_cost(i, actor, targets, obs):
    discounted = any(qualifies(i, actor, t['target'], obs) for t in targets)
    colored = [0] * 6
    colored[COLORS[i]] = 2 if i in (1, 10) else 1
    if discounted and i == 1: colored[1] -= 1
    return colored, GENERIC[i] - (DISCOUNT[i] if discounted else 0), discounted


class TargetCostTests(unittest.TestCase):
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

    def effect(self, i, packet, before, after):
        objects = {v['id']: v for v in before['objects']}
        now = {v['id']: v for v in after['objects']}
        targets = [objects[t['object']['card']] for t in packet['targets'] if t['object'] and t['object']['card'] in objects]
        self.assertTrue(targets)
        v = targets[0]
        if i in (0, 1, 5, 6, 8, 10, 16): self.assertEqual(now[v['id']]['zone'], 'Graveyard')
        elif i in (4, 14, 18):
            event = after['last_target_library_move']
            self.assertEqual((event['source'], event['target_before'], event['owner']), (packet['object'], v['reference'], v['owner']))
            self.assertEqual(event['target_after']['generation'], v['reference']['generation'] + 1)
            self.assertFalse(any(x['reference'] == event['target_after'] for x in after['objects']))
        elif i in (11, 12): self.assertEqual(now[v['id']]['zone'], 'Exile')
        elif i == 13:
            self.assertEqual(now[v['id']]['zone'], 'Hand')
            self.assertEqual(int(after['players'][int(packet['controller'][1])]['cards_drawn_this_turn']), int(before['players'][int(packet['controller'][1])]['cards_drawn_this_turn']) + 1)
        elif i in (2, 3, 7):
            self.assertEqual(int(now[v['id']]['power']), int(v['power']) + {2: 2, 3: -4, 7: 4}[i])
            self.assertEqual(int(now[v['id']]['toughness']), int(v['toughness']) + (4 if i == 7 else 0))
            if i == 2: self.assertTrue(int(now[v['id']]['keywords']) & 8192)
            if i == 3: self.assertEqual(int(after['players'][int(packet['controller'][1])]['cards_drawn_this_turn']), int(before['players'][int(packet['controller'][1])]['cards_drawn_this_turn']) + 1)
        elif i == 9:
            self.assertEqual(now[v['id']]['zone'], 'Graveyard')
        elif i == 15:
            source = next(objects[t['object']['card']] for t in packet['targets'] if t['group'] == 0)
            victim = next(objects[t['object']['card']] for t in packet['targets'] if t['group'] == 1)
            if int(victim['toughness']) <= int(victim['damage']) + int(source['power']): self.assertEqual(now[victim['id']]['zone'], 'Graveyard')
            else: self.assertEqual(int(now[victim['id']]['damage']), int(victim['damage']) + int(source['power']))
        elif i == 17:
            aura = now[packet['card']]
            if int(v['toughness']) <= 3: self.assertEqual(aura['zone'], 'Graveyard')
            else:
                self.assertEqual(aura['attached_to'], v['reference'])
                self.assertEqual((int(now[v['id']]['power']), int(now[v['id']]['toughness'])), (int(v['power']) - 5, int(v['toughness']) - 3))

    def test_all_nineteen_discounted_paid_bodies_both_seats(self):
        self.denied_targets = 0
        for i in range(len(NAMES)):
            for seat in ('P0', 'P1'):
                self.scenario(i, seat)
        self.assertGreater(self.denied_targets, 0)

    def test_real_exile_permissions_keep_discounts_and_deny_direct_cast(self):
        self.denied_targets = 0
        self.denied_exile_shortcuts = 0
        for i in (0, 1, 7, 15, 17):
            for seat in ('P0', 'P1'):
                self.scenario(i, seat, origin='Exile')
        self.assertGreater(self.denied_exile_shortcuts, 0)

    def scenario(self, i, seat, origin='Hand'):
        name = NAMES[i]
        other = 'P1' if seat == 'P0' else 'P0'
        land = BASICS[COLORS[i]]
        cap = GENERIC[i] - DISCOUNT[i] + (1 if i != 10 else 2)
        cap = max(cap, 2 if i in (2, 15) else 0)
        caster_lands = [land] * 32 if i not in (2, 15) else ['Forest'] * 16 + [land if i == 2 else 'Island'] * 16
        deck = caster_lands + [name] * 16 + ['Bear Cub' if i == 2 else 'Three Tree Mascot'] * 12 + ['Opt'] * 12
        if i == 2: deck += ['Giant Growth'] * 12
        if origin == 'Exile':
            cap = 6
            deck = ['Mountain'] * 20 + [land] * 24 + [name] * 20 + ["Alania's Pathmaker"] * 12 + ['Three Tree Mascot'] * 12 + ['Opt'] * 8
        foe = ['Forest'] * 16 + ['Island'] * 16 + ['Mountain'] * 8 + ['Bear Cub'] * 12 + ['Three Tree Mascot'] * 8 + ['Malevolent Chandelier'] * 8 + ['Llanowar Elves'] * 8 + ['Opt'] * 12
        if i == 18: foe += ['Coruscation Mage'] * 16
        decks = [deck, foe] if seat == 'P0' else [foe, deck]
        with self.subTest(card=name, seat=seat, origin=origin), Game(decks, seed=7, starting_player=seat, deck_rules='Freeform') as game:
            paid = False; packet = None; baseline = None; completed = False; replays = 0
            locked = None
            for _ in range(10000):
                obs = game.observe(seat)
                self.assertIsNone(obs['outcome'], name)
                obs = game.observe(obs['actor']); actor = obs['actor']; objects = {v['id']: v for v in obs['objects']}
                self.assertTrue(any(a['kind'] != 'Concede' for a in obs['actions']))
                our = game.observe(seat)
                focal_hand = any(v['owner'] == seat and v['zone'] == origin and v['card'] == name and (origin == 'Hand' or any(r['object'] == v['reference'] for r in our['exile_permissions'])) for v in our['objects'])
                our_creatures = [v for v in our['objects'] if v['owner'] == seat and v['zone'] == 'Battlefield' and 'Creature' in kinds(v['card'])]
                announcing = obs['announcing']
                if actor == seat and obs['decision'] == 'priority' and focal_hand and origin == 'Hand':
                    pool = list(obs['players'][int(actor[1])]['mana'])
                    for v in objects.values():
                        if v['owner'] == actor and v['zone'] == 'Battlefield' and v['card'] in BASICS and not v['tapped']: pool[BASICS.index(v['card'])] += 1
                    qualified = any(qualifies(i, seat, {'Card': v['id']}, obs) for v in objects.values()) or any(qualifies(i, seat, {'Spell': v['card']}, obs) for v in obs['stack'])
                    normal_colors = [0] * 6; normal_colors[COLORS[i]] = 2 if i in (1, 10) else 1
                    full_fundable = all(a >= b for a, b in zip(pool, normal_colors)) and sum(pool) >= sum(normal_colors) + GENERIC[i]
                    if not full_fundable and not qualified:
                        for a in obs['actions']:
                            self.assertFalse(a['kind'] in ('Cast', 'AnnounceCast') and objects[a['card']]['card'] == name)
                if announcing and announcing['source_card'] == name and obs['decision'] == 'pay_costs':
                    want = expected_cost(i, seat, announcing['targets'], obs)
                    if locked is None: locked = want
                    self.assertEqual((obs['payment_request']['required'], obs['payment_request']['generic']), locked[:2])
                    with self.assertRaisesRegex(EngineError, 'IllegalAction'): game.act(obs, {'kind': 'PayCosts', 'payment': [0] * 6})
                    self.assertEqual(game.observe(actor), obs)
                if announcing and announcing['source_card'] == name and obs['decision'] == 'targets':
                    if not announcing['targets']:
                        self.assertNotIn({'kind': 'FinishTargets'}, obs['actions'])
                        with self.assertRaisesRegex(EngineError, 'IllegalAction'): game.act(obs, {'kind': 'FinishTargets'})
                        self.assertEqual(game.observe(actor), obs)
                    pool = list(obs['players'][int(actor[1])]['mana'])
                    for v in objects.values():
                        if v['owner'] == actor and v['zone'] == 'Battlefield' and v['card'] in BASICS and not v['tapped']: pool[BASICS.index(v['card'])] += 1
                    normal_colors = [0] * 6; normal_colors[COLORS[i]] = 2 if i in (1, 10) else 1
                    full_fundable = all(a >= b for a, b in zip(pool, normal_colors)) and sum(pool) >= sum(normal_colors) + GENERIC[i]
                    qualified_chosen = any(qualifies(i, seat, t['target'], obs) for t in announcing['targets'])
                    if not full_fundable and not qualified_chosen:
                        candidates = [{'Spell': s['card']} for s in obs['stack'] if s['kind'] == 'Spell' and s['card'] != announcing['card']] if i in (1, 10) else [{'Card': v['id']} for v in objects.values() if v['zone'] == 'Battlefield' and 'Creature' in kinds(v['card']) and (i != 13 or v['owner'] != actor) and (i != 15 or v['owner'] == actor)]
                        for target in candidates:
                            if qualifies(i, seat, target, obs): continue
                            self.assertNotIn({'kind': 'ChooseTarget', 'target': target}, obs['actions'])
                            with self.assertRaisesRegex(EngineError, 'IllegalAction'): game.act(obs, {'kind': 'ChooseTarget', 'target': target})
                            self.assertEqual(game.observe(actor), obs); self.denied_targets += 1
                if origin == 'Exile' and actor == seat and obs['decision'] == 'priority':
                    for v in objects.values():
                        if v['card'] != name or v['zone'] != 'Exile' or not any(r['object'] == v['reference'] for r in obs['exile_permissions']): continue
                        self.assertFalse(any(a['kind'] == 'Cast' and a['card'] == v['id'] for a in obs['actions']))
                        # A fully funded, qualified single-target packet would be legal
                        # from hand; exile must still enter AnnounceExile.
                        targets = [{'Spell': q['card']} for q in obs['stack'] if q['kind'] == 'Spell'] if i == 1 else [{'Card': q['id']} for q in objects.values() if q['zone'] == 'Battlefield']
                        target = next((t for t in targets if qualifies(i, seat, t, obs)), None)
                        if target is not None and i != 15:
                            colored, generic, _ = expected_cost(i, seat, [{'target': target}], obs)
                            pool = obs['players'][int(actor[1])]['mana']
                            if all(p >= c for p, c in zip(pool, colored)) and sum(pool) >= sum(colored) + generic:
                                payment = colored[:]
                                for color in range(6):
                                    extra = min(generic, pool[color] - payment[color]); payment[color] += extra; generic -= extra
                                direct = {'kind': 'Cast', 'card': v['id'], 'target': target, 'payment': payment}
                                with self.assertRaisesRegex(EngineError, 'IllegalAction'): game.act(obs, direct)
                                self.assertEqual(game.observe(actor), obs); self.denied_exile_shortcuts += 1
                def score(a):
                    k = a['kind']
                    if k in ('AnnounceCast', 'Cast', 'AnnounceExile'):
                        n = objects[a['card']]['card']
                        if actor == seat:
                            if n == name:
                                positive = any(qualifies(i, seat, {'Card': v['id']}, obs) for v in obs['objects']) or any(qualifies(i, seat, {'Spell': s['card']}, obs) for s in obs['stack'])
                                if i == 13: positive = positive and any(v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == 'Bear Cub' for v in obs['objects'])
                                return 1000 if k == ('AnnounceExile' if origin == 'Exile' else 'AnnounceCast') and positive and not paid else -1000
                            if origin == 'Exile' and n == "Alania's Pathmaker": return 800 if not focal_hand and k == 'AnnounceCast' else -1000
                            if i == 2 and n == 'Giant Growth': return 1100 if k == 'AnnounceCast' and obs['attackers'] and our_creatures and focal_hand and not obs['blocking'] and not paid else -1000
                            if i == 15 and n == 'Opt': return 1100 if k == 'AnnounceCast' and our_creatures and focal_hand and not paid else -1000
                            return 650 if i in (2, 15) and not our_creatures and n == ('Bear Cub' if i == 2 else 'Three Tree Mascot') else -1000
                        if i == 18 and n == 'Coruscation Mage': return 800 if not any(v['owner'] == other and v['zone'] == 'Battlefield' and v['token'] for v in objects.values()) else -1000
                        if n == 'Opt': return 700 if i in (1, 10) and focal_hand and not paid else -1000
                        if n == 'Malevolent Chandelier': return 650 if i == 17 and not any(v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == n for v in objects.values()) else -1000
                        if n in ('Bear Cub', 'Three Tree Mascot', 'Llanowar Elves'):
                            return 600 if sum(v['owner'] == other and v['zone'] == 'Battlefield' and v['card'] == n for v in objects.values()) < 2 else -1000
                        return -1000
                    if k == 'PlayLand':
                        count = sum(v['owner'] == actor and v['zone'] == 'Battlefield' and 'Land' in kinds(v['card']) for v in objects.values())
                        if actor == seat:
                            if count >= cap: return -1000
                            have = {v['card'] for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield'}
                            return 950 if objects[a['card']]['card'] not in have else 900
                        return 900
                    if k == 'Attack': return 500 if actor == other and focal_hand and (i != 2 or our_creatures) and (i != 18 or any(v['owner'] == other and v['zone'] == 'Battlefield' and v['token'] for v in objects.values())) and i in (0, 2, 3, 4, 5, 6, 8, 9, 11, 12, 14, 16, 18) and (i == 2 or objects[a['card']]['card'] == 'Bear Cub' or i == 18 and objects[a['card']]['token']) else -1000
                    if k == 'Block': return 500 if actor == seat and i == 2 and focal_hand else -1000
                    if k == 'ChooseTarget':
                        if announcing and announcing['source_card'] == name:
                            if qualifies(i, seat, a['target'], obs): return 1000
                            if i == 15: return 900 if announcing['targets'] and objects.get(a['target'].get('Card'), {}).get('owner') == other else -1000
                            return -1000
                        return 200
                    if k == 'PayCosts': return 1200
                    if k == 'TapMana': return 1200 if origin == 'Exile' and focal_hand and actor == seat and not paid else 800 if obs['decision'] == 'pay_costs' else -1000
                    if k in ('Activate', 'BeginActivate', 'AnnounceActivate', 'ActivateMana', 'AnnounceMana'): return -1000
                    if k == 'ChooseLibraryDestination': return 1000 if a['destination'] == 'Bottom' else 900
                    if k == 'ChooseSpellCost': return 900 if a['cost'] == 0 else 800
                    if k == 'ChooseOffspring': return 900 if a['pay'] == (i == 18 and actor == other) else -1000
                    if k == 'ChooseDiscard': return 500
                    return {'Keep': 1500, 'Mulligan': -1000, 'Concede': -10000, 'FinishTargets': 700, 'PutTrigger': 600, 'Pass': 0}.get(k, 100)
                j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j])); action = obs['actions'][j]
                if action['kind'] == 'PayCosts' and announcing and announcing['source_card'] == name:
                    self.assertIsNotNone(locked); self.assertTrue(locked[2]); payment = action['payment']
                    self.assertTrue(all(p >= c for p, c in zip(payment, locked[0])))
                    self.assertEqual(sum(payment), sum(locked[0]) + locked[1]); paid = True
                    self.assertEqual(announcing['cast'], {'payment': [0]*6, 'origin': origin, 'method': 'Normal', 'offspring': None, 'kicker': None, 'bargain': None, 'evidence': None})
                if action['kind'] == 'Pass' and obs['stack'] and obs['stack'][-1]['source_card'] == name and obs['stack'][-1]['kind'] == 'Spell':
                    packet = obs['stack'][-1]; baseline = obs
                after = self.replay(game, obs, j); replays += 1
                if packet and not any(s['id'] == packet['id'] for s in after['stack']) and not (after['resolving'] and after['resolving']['id'] == packet['id']):
                    self.effect(i, packet, baseline, after); completed = True; break
            self.assertTrue(paid and completed, name); self.assertGreater(replays, 0)

if __name__ == '__main__':
    unittest.main()
