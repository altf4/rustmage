"""Paid public-library acceptance for exact searches, Hand discards and Chocobos."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError, Game

NAMES = ['Circuitous Route', 'Map the Frontier', 'Dragonstorm Forecaster', 'Magitek Infantry', 'Honored Knight-Captain', 'Mystical Teachings', 'Glimpse the Core', 'Radioactive Spider', 'Spinewoods Armadillo', 'Call the Mountain Chocobo', 'Chocobo Racetrack', 'Gysahl Greens', 'Harvester of Misery', 'Outcaster Greenblade', 'Pia, Aether Ascetic', 'Simulacrum Shaper', 'Solemn Simulacrum', 'Steel Wrecking Ball', 'Sureshot Sower']
FACTS = {c['name']: c for c in json.loads((Path(__file__).resolve().parents[1] / 'data/standard/cards.json').read_text())['cards']}
HAND = {8, 12, 17, 18}
ACTIVATE = {2, 3, 4, 7}
SEARCH = set(range(10)) | {13, 14, 15, 16}


def matches(name, rule):
    """Independent Oracle-characteristic predicate; never infer Flash from Flashback."""
    c = FACTS[name]
    typ = c['type_line']
    if isinstance(rule, dict):
        key, value = next(iter(rule.items()))
        if key == 'AnyOf': return any(matches(name, x) for x in value)
        if key == 'AllOf': return all(matches(name, x) for x in value)
        if key == 'Name': return name == value
        if key == 'Subtype': return value in typ.split(' — ')[-1].split() or ('Changeling' in c.get('keywords', []) and value not in ('Forest', 'Mountain', 'Desert', 'Gate', 'Equipment'))
        if key == 'Kinds': return any(value & bit and word in typ.split(' — ')[0].split() for bit, word in ((1, 'Creature'), (2, 'Land'), (8, 'Instant'), (4, 'Sorcery'), (16, 'Artifact'), (32, 'Enchantment')))
        if key == 'Keyword': return 'Flash' in c.get('keywords', [])
        raise AssertionError(rule)
    return {'BasicLand': 'Basic' in typ and 'Land' in typ, 'Land': 'Land' in typ, 'Creature': 'Creature' in typ, 'Artifact': 'Artifact' in typ, 'Enchantment': 'Enchantment' in typ}[rule]


def cost(text):
    required = [0] * 6
    generic = 0
    for symbol in text.replace('}', '').split('{')[1:]:
        if symbol.isdigit(): generic += int(symbol)
        else: required['WUBRGC'.index(symbol)] += 1
    return required, generic


class ExactSearchTests(unittest.TestCase):
    def replay(self, game, obs, j):
        saved = game.snapshot()
        with game.clone() as fork: expected = fork.step(fork.observe(obs['actor']), j)
        self.assertEqual(game.step(obs, j), expected)
        game.restore(saved)
        with self.assertRaisesRegex(EngineError, 'StaleRevision'): game.step(obs, j)
        after = game.step(game.observe(obs['actor']), j)
        expected['revision'] = after['revision']
        self.assertEqual(after, expected)
        game.drop_snapshot(saved)
        return after

    def test_nineteen_full_bodies_both_seats(self):
        for i in range(19):
            for seat in ('P0', 'P1'):
                with self.subTest(card=NAMES[i], seat=seat): self.scenario(i, seat)

    def test_zero_searches_optional_declines_and_full_cast_residuals(self):
        for seat in ('P0', 'P1'):
            for i in (0, 1, 8, 14, 15, 16):
                with self.subTest(card=NAMES[i], seat=seat, zero=True): self.scenario(i, seat, zero=True)
            for i in (14, 15, 16):
                with self.subTest(card=NAMES[i], seat=seat, decline=True): self.scenario(i, seat, decline=True)
            for i in HAND:
                with self.subTest(card=NAMES[i], seat=seat, cast=True): self.scenario(i, seat, cast=True)
            with self.subTest(card=NAMES[6], seat=seat, cave=True): self.scenario(6, seat, cave=True)

    def test_flashback_searches_and_fixed_token_bodies(self):
        for seat in ('P0', 'P1'):
            for i in (5, 9, 11):
                with self.subTest(card=NAMES[i], seat=seat): self.scenario(i, seat, flashback=True)

    def test_actual_death_draws_distinguish_mandatory_and_optional(self):
        for seat in ('P0', 'P1'):
            for name, accept in [('Simulacrum Shaper', False), ('Solemn Simulacrum', False), ('Solemn Simulacrum', True)]:
                with self.subTest(card=name, seat=seat, accept=accept):
                    deck = ['Mountain']*24 + ['Forest']*16 + [name]*16 + ['Shock']*16
                    foe = ['Forest']*72
                    other = 'P1' if seat == 'P0' else 'P0'
                    with Game([deck,foe] if seat=='P0' else [foe,deck],seed=31,starting_player=seat,deck_rules='Freeform') as game:
                        source = death = baseline = None
                        for _ in range(16000):
                            obs = game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                            objects={v['id']:v for v in obs['objects']}
                            body=next((v for v in objects.values() if v['owner']==seat and v['card']==name and v['zone']=='Battlefield'),None)
                            if body: source=body['id']
                            grave=source is not None and objects[source]['zone']=='Graveyard'
                            packets=obs['stack']+obs['waiting_triggers']+([obs['announcing']] if obs['announcing'] else [])+([obs['resolving']] if obs['resolving'] else [])
                            packet=next((v for v in packets if grave and v['source_card']==name and v['kind']!='Spell'),None)
                            if packet and death is None:death=packet['id'];baseline=obs['players'][int(seat[1])]['cards_drawn_this_turn']
                            def score(a):
                                k=a['kind'];v=objects.get(a.get('card'))
                                if k in ('Cast','AnnounceCast'):
                                    if obs['actor']!=seat:return -1000
                                    if v['card']==name:return 1000 if source is None else -1000
                                    return 1100 if v['card']=='Shock' and body else -1000
                                if k=='ChooseTarget':return 1500 if a['target']=={'Card':source} else -1000
                                if k=='ChooseOptional':return 1500 if a['accept']==(accept if grave else False) else -1000
                                if k=='PlayLand':return 1200
                                if k=='TapMana':return 1000 if obs['decision']=='pay_costs' else -1000
                                return {'Keep':1700,'PayCosts':1600,'PutTrigger':1500,'FinishTargets':1400,'FinishAttackers':400,'FinishBlockers':400,'Pass':0,'Concede':-10000,'Mulligan':-1000,'Attack':-1000,'Block':-1000,'ActivateMana':-1000,'AnnounceMana':-1000}.get(k,100)
                            j=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]))
                            after=self.replay(game,obs,j)
                            live=after['stack']+after['waiting_triggers']+([after['announcing']] if after['announcing'] else [])+([after['resolving']] if after['resolving'] else [])
                            if death is not None and not any(v['id']==death for v in live):
                                self.assertEqual(after['players'][int(seat[1])]['cards_drawn_this_turn']-baseline,int(name=='Simulacrum Shaper' or accept))
                                self.assertEqual(next(v for v in after['objects'] if v['id']==source)['zone'],'Graveyard')
                                break
                        else:self.fail('death draw not witnessed')

    def test_armadillo_intrinsic_reach_and_actual_ward_three_payment_or_decline(self):
        for seat in ('P0','P1'):
            for pay in (False,True):
                with self.subTest(seat=seat,pay=pay):
                    other='P1' if seat=='P0' else 'P0'
                    deck=['Forest']*40+['Spinewoods Armadillo']*16
                    foe=['Mountain']*40+['Shock']*16
                    with Game([deck,foe] if seat=='P0' else [foe,deck],seed=47,starting_player=seat,deck_rules='Freeform') as game:
                        source=None;shock=None;ward=False;complete=False;sent=False
                        for _ in range(14000):
                            obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']}
                            source=next((v for v in objects.values() if v['owner']==seat and v['card']=='Spinewoods Armadillo' and v['zone']=='Battlefield'),source)
                            if source:self.assertEqual(source['keywords']&2,2)
                            req=obs['resolution_payment']
                            if req:
                                self.assertEqual(req['payer'],other);self.assertEqual(req['cost']['generic'],3)
                                self.assertEqual(game.observe(seat)['resolution_payment'],req);self.assertEqual(game.observe(seat)['actions'],[])
                                shock=req['victim'];ward=True
                            def score(a):
                                k=a['kind'];v=objects.get(a.get('card'))
                                if k in ('Cast','AnnounceCast'):
                                    if v['card']=='Spinewoods Armadillo':return 1100 if source is None else -1000
                                    return 1100 if source is not None and not sent else -1000
                                if k in ('Activate','BeginActivate','AnnounceActivate','AnnounceFlashback'):return -1000
                                if k=='ChooseTarget':return 1600 if source and a['target']=={'Card':source['id']} else -1000
                                if k=='DeclineResolutionMana':return 1800 if not pay else -1000
                                if k=='PayResolutionMana':return 1800
                                if k=='TapMana':return 1300 if obs['decision'] in ('pay_costs','resolution_mana') else -1000
                                if k=='PlayLand':return 1400
                                return {'Keep':2000,'PayCosts':1900,'PutTrigger':1800,'FinishTargets':1700,'FinishAttackers':400,'FinishBlockers':400,'Pass':0,'Concede':-10000,'Mulligan':-1000,'Attack':-1000,'Block':-1000}.get(k,100)
                            j=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]))
                            if obs['actions'][j]['kind']=='PayCosts' and obs['announcing']['source_card']=='Shock':sent=True
                            after=self.replay(game,obs,j)
                            live=after['stack']+after['waiting_triggers']+([after['announcing']] if after['announcing'] else [])+([after['resolving']] if after['resolving'] else [])
                            if ward and not any(v['id']==shock for v in live):
                                body=next(v for v in after['objects'] if v['reference']==source['reference'])
                                self.assertEqual(body['zone'],'Battlefield');self.assertEqual(body['damage'],2 if pay else 0)
                                complete=True;break
                        self.assertTrue(complete)

    def test_chocobo_old_tokens_see_lands_new_tokens_start_at_two_and_cleanup_resets(self):
        for seat in ('P0', 'P1'):
            for name in ('Chocobo Racetrack','Call the Mountain Chocobo','Gysahl Greens'):
                with self.subTest(card=name,seat=seat):
                    deck=['Forest']*24+['Mountain']*24+[name]*16
                    foe=['Forest']*64
                    with Game([deck,foe] if seat=='P0' else [foe,deck],seed=41,starting_player=seat,deck_rules='Freeform') as game:
                        cast=False;pending=None;lands=0;last_turn=None;cleanup=False
                        for _ in range(14000):
                            obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                            objects={v['id']:v for v in obs['objects']}
                            birds=[v for v in objects.values() if v['owner']==seat and v['zone']=='Battlefield' and v['token'] and v['token']['subtypes']=='Bird']
                            source=any(v['owner']==seat and v['card']==name and v['zone']=='Battlefield' for v in objects.values())
                            clear=not(obs['stack'] or obs['waiting_triggers'] or obs['announcing'] or obs['resolving'])
                            if pending and clear:
                                old,turn=pending
                                for ref,power in old:
                                    now=next(v for v in birds if v['reference']==ref)
                                    self.assertEqual((now['power'],now['toughness']),(power+1,2))
                                newborn=[v for v in birds if all(v['reference']!=r for r,_ in old)]
                                self.assertEqual(len(newborn),int(name=='Chocobo Racetrack'))
                                self.assertTrue(all((v['power'],v['toughness'])==(2,2) for v in newborn))
                                pending=None;lands+=1;last_turn=turn
                            if lands>=2 and clear and obs['turn']!=last_turn:
                                self.assertTrue(birds);self.assertTrue(all((v['power'],v['toughness'])==(2,2) for v in birds));cleanup=True;break
                            def score(a):
                                k=a['kind'];v=objects.get(a.get('card'))
                                if k in ('Cast','AnnounceCast'):return 1000 if v['card']==name and not cast and obs['actor']==seat else -1000
                                if k=='PlayLand':return 1200 if lands<2 else -1000
                                if k=='ChooseSearch':return 1500
                                if k=='FinishSearch':return 1400 if obs['search_choice']['selected'] else 0
                                if k=='TapMana':return 1000 if obs['decision']=='pay_costs' else -1000
                                return {'Keep':1700,'PayCosts':1600,'PutTrigger':1500,'FinishAttackers':400,'FinishBlockers':400,'Pass':0,'Concede':-10000,'Mulligan':-1000,'Attack':-1000,'Block':-1000,'ActivateMana':-1000,'AnnounceMana':-1000}.get(k,100)
                            j=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));a=obs['actions'][j]
                            if a['kind']=='PayCosts' and obs['announcing']['source_card']==name:cast=True
                            if a['kind']=='PlayLand' and obs['actor']==seat and (source or birds):pending=([(v['reference'],v['power']) for v in birds],obs['turn'])
                            self.replay(game,obs,j)
                        self.assertTrue(cleanup)

    def scenario(self, i, seat, zero=False, decline=False, cast=False, cave=False, flashback=False):
        name = NAMES[i]
        other = 'P1' if seat == 'P0' else 'P0'
        lands = ['Plains'] * 12 + ['Island'] * 12 + ['Swamp'] * 12 + ['Mountain'] * 12 + ['Forest'] * 12
        fixtures = ['Bear Cub'] * 8 + ['Three Tree Mascot'] * 8 + ['Spectral Sailor'] * 8 + ['Sandstorm Verge'] * 4 + ['Lush Portico'] * 4 + ['Boulderborn Dragon'] * 4 + ['Cathar Commando'] * 4 + ['Giant Growth'] * 4 + ['Think Twice'] * 4
        if i == 5: fixtures += ['Gysahl Greens'] * 4
        if i == 7: fixtures += ['Hero in Training'] * 4 + ['Attercop'] * 4 + ['Masked Meower'] * 4
        if i in (4, 14): fixtures += ["Medic's Kitesail"] * 8 + ['Anthem of Champions'] * 8
        if cave: fixtures += ['Captivating Cave'] * 16 + ['Thrill of Possibility'] * 16
        deck = lands + [name] * 16 + fixtures
        foe = ['Plains'] * 32 + ['Spectral Sailor'] * 16 + ['Boulderborn Dragon'] * 16
        with Game([deck, foe] if seat == 'P0' else [foe, deck], seed=19, starting_player=seat, deck_rules='Freeform') as game:
            phase = 0
            paid = False
            searched = False
            completed = False
            source_id = None
            activated = False
            frames = {}
            cast_packet = None
            baseline = None
            tokens_before = None
            for _ in range(24000):
                obs = game.observe('P0')
                self.assertIsNone(obs['outcome'], (name, seat, cast, cave))
                obs = game.observe(obs['actor'])
                actor = obs['actor']
                objects = {v['id']: v for v in obs['objects']}
                own = [v for v in objects.values() if v['owner'] == seat and v['zone'] == 'Battlefield']
                source = next((v for v in own if v['card'] == name), None)
                if source:
                    source_id = source['id']
                    if i == 3:
                        others = [v for v in own if v['id'] != source['id'] and 'Artifact' in FACTS.get(v['card'], {}).get('type_line', '')]
                        self.assertEqual(source['power'], 1 + bool(others))
                    if i == 13:
                        deserts = sum('Desert' in FACTS.get(v['card'], {}).get('type_line', '') for v in own)
                        self.assertEqual((source['power'], source['toughness']), (1+deserts, 2+deserts))
                    if i == 7: self.assertEqual(source['keywords'] & (2 | 128), 2 | 128)
                foes = [v for v in objects.values() if v['owner'] == other and v['zone'] == 'Battlefield' and 'Creature' in FACTS.get(v['card'], {}).get('type_line', '')]
                artifacts = [v for v in own if v['card'] == 'Three Tree Mascot']
                ann = obs['announcing']
                frame = obs['search_choice']
                ours = frame and obs['resolving']['source_card'] == name
                if ours:
                    self.assertEqual(actor, seat)
                    hidden = game.observe(other)
                    self.assertIsNone(hidden['search_choice'])
                    self.assertEqual(hidden['actions'], [])
                    legal = {a['card'] for a in obs['actions'] if a['kind'] == 'ChooseSearch'}
                    eligible = {v['id'] for v in frame['cards'] if matches(v['card'], frame['filter']) and v['id'] not in frame['selected']}
                    self.assertEqual(legal, eligible if len(frame['selected']) < frame['maximum'] else set())
                    self.assertFalse(any(v['zone'] == 'Library' for v in hidden['objects']))
                    if obs['resolving']['id'] not in frames:
                        frames[obs['resolving']['id']] = (frame['cards'][:], obs['players'][int(seat[1])]['library_count'])
                    wrong = dict(obs, actor=other)
                    with self.assertRaisesRegex(EngineError, 'WrongActor'): game.act(wrong, {'kind': 'FinishSearch'})
                    with self.assertRaises(EngineError): game.act(obs, {'kind': 'ChooseSearch', 'card': 2**32-1})
                    self.assertEqual(game.observe(actor), obs)
                    rejected = next((v for v in frame['cards'] if not matches(v['card'], frame['filter'])), None)
                    if rejected:
                        with self.assertRaises(EngineError): game.act(obs, {'kind': 'ChooseSearch', 'card': rejected['id']})
                        self.assertEqual(game.observe(actor), obs)
                def score(a):
                    k = a['kind']
                    v = objects.get(a.get('card'))
                    if k == 'AnnounceFlashback': return 1600 if flashback and phase == 1 and a['card'] == source_id else -1000
                    if k in ('Cast', 'AnnounceCast'):
                        n = v['card']
                        if actor != seat: return 700 if n in ('Spectral Sailor', 'Boulderborn Dragon') and len(foes) < 3 else -1000
                        if n == name:
                            if i in HAND and not cast: return -1000
                            if source or paid or phase: return -1000
                            ready = (i not in (3, 12) or bool(artifacts)) and (i not in (12, 17) or bool(foes))
                            if cave: ready = any(v['card'] == 'Captivating Cave' and v['zone'] == 'Graveyard' for v in objects.values())
                            return 900 if ready and k == 'AnnounceCast' else -1000
                        if cave and n == 'Thrill of Possibility': return 950 if not any(v['card'] == 'Captivating Cave' and v['zone'] == 'Graveyard' for v in objects.values()) else -1000
                        return 750 if n == 'Three Tree Mascot' and not artifacts and i in (3, 12) else -1000
                    if k in ('Activate', 'AnnounceActivate', 'BeginActivate'):
                        if actor != seat or v['card'] != name or activated: return -1000
                        if i in HAND and not cast: return 1100 if (i == 8 or bool(foes)) and k == 'AnnounceActivate' else -1000
                        return 1100 if i in ACTIVATE and source and k == 'AnnounceActivate' else -1000
                    if k == 'ChooseMode': return 1200 if a['mode'] == int(cave) else -1000
                    if k == 'ChooseTarget' and 'Card' in a['target']:
                        t = objects[a['target']['Card']]
                        if cave: return 1300 if t['card'] == 'Captivating Cave' and t['zone'] == 'Graveyard' else -1000
                        return 1300 if t['owner'] == other else 500
                    if k == 'Discard':
                        if cave: return 1300 if v['card'] == 'Captivating Cave' else -1000
                        return 1200 if v['card'] != name else 600
                    if k == 'ChooseOptional': return 1400 if a['accept'] != decline else -1000
                    if k == 'DeclineEffectPayment': return 1400 if decline else -1000
                    if k == 'ChooseSearch':
                        if zero: return -1000
                        # Deliberately choose same-name distinct copies for the two-card searches.
                        t = next(v for v in frame['cards'] if v['id'] == a['card'])
                        return 1500 if t['card'] == ('Forest' if i < 2 else 'Three Tree Mascot' if i == 7 else 'Boulderborn Dragon' if i == 2 else 'Cathar Commando' if i == 5 else 'Sandstorm Verge' if i in (8, 13) else 'Mountain' if i == 9 else name if i == 3 else 'Anthem of Champions' if i == 14 else "Medic's Kitesail" if i == 4 else 'Forest') else 1400
                    if k == 'FinishSearch': return 1300 if zero or len(frame['selected']) >= frame['maximum'] else 0
                    if k == 'PlayLand':
                        n = v['card'];count = sum(x['card'] == n for x in own)
                        if cave and n == 'Captivating Cave': return -1000
                        if n == 'Sandstorm Verge': return 1150 if i == 13 and not any(x['card'] == n for x in own) else 500
                        return 1000-count if n in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest') else 400
                    if k == 'TapMana': return 1000 if obs['decision'] == 'pay_costs' else -1000
                    return {'Keep':1600, 'PayCosts':1500, 'PutTrigger':1400, 'FinishTargets':1300, 'FinishAttackers':400, 'FinishBlockers':400, 'Pass':0, 'Concede':-10000, 'Mulligan':-1000, 'Attack':-1000, 'Block':-1000, 'ActivateMana':-1000, 'AnnounceMana':-1000}.get(k, 100)
                j = max(range(len(obs['actions'])), key=lambda j: score(obs['actions'][j]))
                action = obs['actions'][j]
                if obs['decision'] == 'targets':
                    with self.assertRaises(EngineError): game.act(obs, {'kind':'ChooseTarget','target':{'Card':2**32-1}})
                    with self.assertRaisesRegex(EngineError,'WrongActor'): game.act(dict(obs,actor=other if actor==seat else seat),action)
                    self.assertEqual(game.observe(actor),obs)
                if ann and ann['source_card'] == name and action['kind'] == 'PayCosts':
                    with self.assertRaises(EngineError): game.act(obs,dict(action,payment=[0]*6))
                    self.assertEqual(game.observe(actor),obs)
                    self.assertIsNotNone(obs['payment_request'])
                    request = obs['payment_request']
                    if ann['kind'] == 'Spell':
                        wanted_cost = ('{5}{B}' if i == 5 else '{5}{R}' if i == 9 else '{6}{G}') if phase else FACTS[name]['mana_cost']
                    else:
                        wanted_cost = {2:'{2}',3:'{2}{W}',4:'{4}{W}{W}',7:'{2}',8:'{1}{G}',12:'{1}{B}',17:'{1}{R}',18:'{3}{G}'}[i]
                    self.assertEqual((request['required'], request['generic']), cost(wanted_cost))
                    self.assertEqual(sum(action['payment']), sum(request['required']) + request['generic'])
                    paid = True
                    source_id = ann['card']
                    if flashback and phase:
                        self.assertEqual(ann['cast']['method'], 'Flashback')
                        self.assertEqual(ann['cast']['origin'], 'Graveyard')
                    if ann['kind'] != 'Spell':
                        activated = True
                        if i == 4:
                            self.assertTrue(any(v['token'] and v['token']['subtypes'] == 'Human Soldier' and v['power'] == 1 and v['toughness'] == 1 for v in own))
                        if i in HAND and not cast:
                            self.assertEqual(objects[source_id]['zone'], 'Hand')
                            self.assertEqual(obs['players'][int(seat[1])]['spells_cast_this_turn'], 0)
                if action['kind'] == 'Pass' and obs['stack'] and obs['stack'][-1]['source_card'] == name:
                    cast_packet = obs['stack'][-1]
                    baseline = obs
                    tokens_before = len([v for v in own if v['token'] and v['token']['subtypes'] == 'Bird'])
                after = self.replay(game, obs, j)
                self.assertEqual(after['schema'], 93)
                if ann and ann['source_card'] == name and action['kind'] == 'PayCosts' and i in HAND and not cast:
                    self.assertEqual(next(v for v in after['objects'] if v['id'] == source_id)['zone'], 'Graveyard')
                    self.assertEqual(after['players'][int(seat[1])]['cards_discarded_this_turn'], obs['players'][int(seat[1])]['cards_discarded_this_turn'] + 1)
                    self.assertIn('ActivatedAbility', after['stack'][-1]['kind'])
                if ours:
                    cards, count = frames[obs['resolving']['id']]
                    if action['kind'] == 'ChooseSearch':
                        self.assertEqual(after['players'][int(seat[1])]['library_count'], count)
                        for v in cards:
                            self.assertEqual(next(x for x in after['search_choice']['cards'] if x['id'] == v['id'])['reference'], v['reference'])
                        self.assertFalse(any(v['id'] in after['search_choice']['selected'] for v in game.observe(other)['objects']))
                        with game.clone() as fork:
                            # A zero selection is independently legal before the first pick.
                            if not frame['selected']: fork.act(fork.observe(actor), {'kind': 'FinishSearch'})
                        self.assertEqual(game.observe(actor), after)
                    elif action['kind'] == 'FinishSearch':
                        self.assertEqual(after['players'][int(seat[1])]['library_count'], count-len(frame['selected']))
                        public = game.observe(other)
                        for selected in frame['selected']:
                            old = next(v for v in cards if v['id'] == selected)
                            new = next(v for v in public['objects'] if v['id'] == selected)
                            self.assertEqual(new['reference']['generation'], old['reference']['generation']+1)
                            self.assertEqual(new['zone'], frame['destination'])
                            if frame['destination'] == 'Battlefield': self.assertEqual(new['tapped'], frame['tapped'])
                            else: self.assertIn(new['reference'], after['known_hand'])
                        if i < 2 and not zero:
                            self.assertEqual(len(frame['selected']), 2)
                            self.assertEqual([next(v['card'] for v in cards if v['id'] == x) for x in frame['selected']], ['Forest', 'Forest'])
                        searched = True
                        if i == 8:
                            self.assertEqual(after['players'][int(seat[1])]['life'], obs['players'][int(seat[1])]['life']+3)
                        if i == 9:
                            birds = [v for v in after['objects'] if v['owner'] == seat and v['token'] and v['token']['subtypes'] == 'Bird']
                            self.assertEqual(len(birds), tokens_before+1)
                            self.assertTrue(any(v['power']==2 and v['toughness']==2 for v in birds))
                        completed = True
                if decline and action['kind'] in ('ChooseOptional', 'DeclineEffectPayment') and obs['resolving'] and obs['resolving']['source_card'] == name:
                    self.assertIsNone(after['search_choice'])
                    self.assertEqual(after['players'][int(seat[1])]['library_count'], obs['players'][int(seat[1])]['library_count'])
                    self.assertEqual(after['known_hand'], obs['known_hand'])
                    completed = True
                if cast_packet and not any(v['id']==cast_packet['id'] for v in after['stack']) and not (after['resolving'] and after['resolving']['id']==cast_packet['id']):
                    p = int(seat[1])
                    if i in (10, 11):
                        birds = [v for v in after['objects'] if v['owner']==seat and v['token'] and v['token']['subtypes']=='Bird']
                        if i == 11: self.assertEqual(len(birds), tokens_before+1)
                        completed = i == 11
                    if i in (12, 17, 18) and not (i in SEARCH):
                        if not cast:
                            target = cast_packet['targets'][0]['object']
                            now = next(v for v in after['objects'] if v['id']==target['card'])
                            old = next(v for v in baseline['objects'] if v['reference']==target)
                            if i == 12 and old['toughness']>2:
                                self.assertEqual((now['power'],now['toughness']), (old['power']-2,old['toughness']-2))
                            else: self.assertEqual(now['zone'], 'Graveyard')
                        else:
                            if i in (12, 17) and cast_packet['kind'] == 'Spell':
                                cast_packet = baseline = None
                                continue
                            if i == 12:
                                for old in baseline['objects']:
                                    if old['zone'] != 'Battlefield' or old['id'] == source_id or 'Creature' not in FACTS.get(old['card'], {}).get('type_line', ''): continue
                                    now = next(v for v in after['objects'] if v['id'] == old['id'])
                                    if old['toughness'] <= 2: self.assertEqual(now['zone'], 'Graveyard')
                                    else: self.assertEqual((now['power'], now['toughness']), (old['power']-2, old['toughness']-2))
                            if i == 17:
                                target = cast_packet['targets'][0]['object']
                                now = next(v for v in after['objects'] if v['id'] == target['card'])
                                self.assertEqual(now['zone'], 'Graveyard')
                            body = next(v for v in after['objects'] if v['id']==source_id)
                            self.assertEqual(body['zone'], 'Battlefield')
                            if i == 18: self.assertTrue(body['keywords'] & 2)
                            if i == 8: self.assertTrue(body['keywords'] & 2)
                        completed = True
                    if cave:
                        target = cast_packet['targets'][0]['object']
                        returned = next(v for v in after['objects'] if v['id']==target['card'])
                        self.assertEqual(returned['zone'], 'Battlefield');self.assertTrue(returned['tapped']);self.assertGreater(returned['reference']['generation'],target['generation'])
                        completed = True
                    cast_packet = baseline = None
                # Racetrack must observe an actual subsequent land entry and create a fixed child.
                if i == 10 and source and any(v['owner']==seat and v['token'] and v['token']['subtypes']=='Bird' for v in after['objects']): completed=True
                if cast and i == 8 and source:
                    self.assertTrue(source['keywords'] & 2)
                    completed = True
                if completed:
                    self.assertTrue(paid)
                    if i in SEARCH and not decline and not cave and not cast: self.assertTrue(searched)
                    if flashback and phase == 0:
                        phase = 1
                        paid = completed = False
                        cast_packet = baseline = None
                        continue
                    if flashback:
                        self.assertEqual(next(v for v in after['objects'] if v['id'] == source_id)['zone'], 'Exile')
                    return
            self.fail(('no complete witness', name, seat, zero, decline, cast, cave))


if __name__ == '__main__': unittest.main()
