#!/usr/bin/env python3
"""Compile pinned card facts into immutable native definitions.

Only explicit implementations and exact complete-text templates are executable. All other identities
remain known but unsupported, never guessed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
LEGACY = [
    ('Plains', 'Land', '{W}', 0), ('Island', 'Land', '{U}', 1),
    ('Swamp', 'Land', '{B}', 2), ('Mountain', 'Land', '{R}', 3), ('Forest', 'Land', '{G}', 4),
    ('Grizzly Bears', 'Creature(2, 2)', '{1}{G}', None),
    ('Goblin Piker', 'Creature(2, 1)', '{1}{R}', None),
    ('Hill Giant', 'Creature(3, 3)', '{3}{R}', None),
    ('Llanowar Elves', 'Creature(1, 1)', '{G}', 4),
    ('Lightning Bolt', 'Damage(3)', '{R}', None), ('Shock', 'Damage(2)', '{R}', None),
    ('Giant Growth', 'Grow', '{G}', None), ('Divination', 'Draw', '{2}{U}', None),
    ('Counterspell', 'Counter', '{U}{U}', None),
]
KEYWORDS = {'flying': 1, 'reach': 2, 'haste': 4, 'vigilance': 8, 'defender': 16, 'flash': 32,
            'lifelink': 64, 'deathtouch': 128, 'trample': 256, 'first strike': 512, 'double strike': 1024}
UNLIMITED = {'Cid, Timeless Artificer', 'Hare Apparent', 'Slime Against Humanity', "Sphinx's Approach", 'Tempest Hawk'}


def fixed_cost(cost):
    if not re.fullmatch(r'(?:\{(?:[0-9]+|[WUBRG])\})+', cost):
        return None
    colored = [0] * 5
    generic = 0
    for symbol in re.findall(r'\{([^}]+)\}', cost):
        if symbol.isdigit():
            generic += int(symbol)
        else:
            colored['WUBRG'.index(symbol)] += 1
    return colored, generic


def creature_template(card):
    # Missing/unknown rules text is not a blank text box. A fresh checksum only
    # establishes data integrity; it cannot establish omitted card semantics.
    if not isinstance(card.get('oracle_text'), str):
        return None
    if card.get('layout') != 'normal' or card.get('type_line', '').split(' — ')[0] not in ('Creature', 'Artifact Creature'):
        return None
    if not card.get('power', '').isdigit() or not card.get('toughness', '').isdigit():
        return None
    if fixed_cost(card.get('mana_cost', '')) is None:
        return None
    # Remove only balanced, unnested reminder text; fail closed on residual text.
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    words = {word.strip().casefold() for word in text.replace('\n', ',').split(',') if word.strip()}
    if not words <= KEYWORDS.keys():
        return None
    return f"Creature({int(card['power'])}, {int(card['toughness'])})", sum(KEYWORDS[word] for word in words)


def spell_template(card):
    if (card.get('layout') != 'normal' or card.get('type_line', '').split(' — ')[0] not in ('Instant', 'Sorcery')
            or fixed_cost(card.get('mana_cost', '')) is None):
        return None
    return {
        "Return target creature to its owner's hand.": 'ReturnToHand',
        'Destroy target creature.': 'Destroy',
    }.get(card.get('oracle_text'))



DRAW_COUNTS = {'a card': 1, 'two cards': 2, 'three cards': 3, 'four cards': 4}


COUNT_WORDS = {'a': 1, 'one': 1, 'two': 2, 'three': 3, 'four': 4, 'five': 5, 'six': 6}
COUNTER_KINDS = {'+1/+1': 'PlusOne', '-1/-1': 'MinusOne', 'stun': 'Stun'}

def counter_instruction(subject, count, kind):
    return f'PutCounters {{ subject: Subject::{subject}, kind: CounterKind::{COUNTER_KINDS[kind]}, amount: {COUNT_WORDS[count]} }}'


TOKEN_DEFS = {}
TOKEN_COUNTS = dict(COUNT_WORDS, an=1, seven=7, eight=8)

def token_program(text):
    """Fixed fully-described creature tokens only; no inferred predefined abilities."""
    if not isinstance(text, str):
        return None
    m = re.fullmatch(r'create (a|an|one|two|three|four|five|six|seven|eight) (tapped )?([0-9]+)/([0-9]+) (.+?) (?:(artifact|enchantment) )?creature tokens?(.*)', text, re.DOTALL)
    if not m:
        return None
    count, tapped, power, toughness, description, extra_type, suffix = m.groups()
    if max(int(power), int(toughness)) > 32767:
        return None
    colors = {'white': 1, 'blue': 2, 'black': 4, 'red': 8, 'green': 16, 'colorless': 0}
    parts = description.split()
    color_words = []
    while parts and (parts[0] in colors or parts[0] == 'and'):
        color_words.append(parts.pop(0))
    color_text = ' '.join(color_words)
    if not re.fullmatch(r'(white|blue|black|red|green|colorless)( and (white|blue|black|red|green))*', color_text):
        return None
    color_names = color_text.split(' and ')
    if len(set(color_names)) != len(color_names) or ('colorless' in color_names and len(color_names) != 1):
        return None
    if not parts or not all(re.fullmatch(r"[A-Z][A-Za-z'-]*", part) for part in parts):
        return None
    subtypes = ' '.join(parts)
    name = subtypes + ' Token'
    if suffix.startswith(' named '):
        match = re.fullmatch(r' named ([A-Z][A-Za-z -]*?)(\.| with .*)', suffix)
        if not match: return None
        name, suffix = match.groups()
    keywords = 0
    if suffix != '.':
        if not suffix.startswith(' with '): return None
        ability = suffix[len(' with '):]
        if ability == '"This token can\'t block."':
            keywords = 2048
        else:
            if not ability.endswith('.'): return None
            words = [word.strip() for word in ability[:-1].replace(' and ', ',').split(',')]
            if not words or not all(word in KEYWORDS for word in words): return None
            keywords = sum(KEYWORDS[word] for word in set(words))
    factory = {'name': name, 'subtypes': subtypes, 'colors': sum(colors[word] for word in color_names),
               'power': int(power), 'toughness': int(toughness), 'extra_type': extra_type,
               'keywords': keywords}
    key = hashlib.sha256(json.dumps(factory, sort_keys=True).encode()).hexdigest()[:24]
    if key in TOKEN_DEFS and TOKEN_DEFS[key] != factory:
        raise ValueError("token factory identity collision")
    TOKEN_DEFS[key] = factory
    return [f'CreateTokens {{ token: __TOKEN_{key}__, amount: {TOKEN_COUNTS[count]}, tapped: {str(bool(tapped)).lower()} }}']


def statement_program(text):
    """Compile only these complete sentences; residual text never gets ignored."""
    if not isinstance(text, str) or not text:
        return None
    text = text[0].lower() + text[1:]
    token = token_program(text)
    if token: return token
    if not text.endswith('.'):
        return None
    if m := re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on this creature\.', text):
        return [counter_instruction('Source', m[1], m[2])]
    draw = r'(a card|two cards|three cards|four cards)'
    if m := re.fullmatch(r'draw ' + draw + r'\.', text):
        return [f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})']
    if m := re.fullmatch(r'you gain ([0-9]+) life\.', text):
        return [f'GainLife(PlayerSet::Controller, {int(m[1])})'] if int(m[1]) <= 65535 else None
    if m := re.fullmatch(r'you lose ([0-9]+) life\.', text):
        return [f'LoseLife(PlayerSet::Controller, {int(m[1])})'] if int(m[1]) <= 65535 else None
    if text == 'you may draw a card. If you do, discard a card.':
        return ['May { skip: 2 }', 'Draw(PlayerSet::Controller, 1)', 'Discard(PlayerSet::Controller, 1)']
    if text == 'each player discards a card.':
        return ['Discard(PlayerSet::Each, 1)']
    if text == 'each opponent discards a card.':
        return ['Discard(PlayerSet::Opponent, 1)']
    if text == 'each player draws a card.':
        return ['Draw(PlayerSet::Each, 1)']
    if m := re.fullmatch(r'you gain ([0-9]+) life and draw ' + draw + r'\.', text):
        return [f'GainLife(PlayerSet::Controller, {int(m[1])})', f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[2]]})'] if int(m[1]) <= 65535 else None
    if m := re.fullmatch(r'each opponent loses ([0-9]+) life and you gain ([0-9]+) life\.', text):
        return [f'LoseLife(PlayerSet::Opponent, {int(m[1])})', f'GainLife(PlayerSet::Controller, {int(m[2])})'] if max(int(m[1]), int(m[2])) <= 65535 else None
    if m := re.fullmatch(r'draw ' + draw + r', then discard (a card|two cards|three cards|four cards)\.', text):
        return [f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})', f'Discard(PlayerSet::Controller, {DRAW_COUNTS[m[2]]})']
    if m := re.fullmatch(r'you may draw ' + draw + r'\.', text):
        return ['May { skip: 1 }', f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})']
    if '. ' in text:
        instructions = []
        for sentence in text.split('. '):
            result = statement_program(sentence if sentence.endswith('.') else sentence + '.')
            if result is None:
                return None
            instructions.extend(result)
        return instructions
    return None



TARGET_RULES = {
    'None': 'TargetRule::None', 'Creature': 'TargetRule::Creature',
    'Any': 'TargetRule::Any', 'Spell': 'TargetRule::Spell',
    'Player': 'TargetRule::Player { opponent_only: false }',
    'Opponent': 'TargetRule::Player { opponent_only: true }',
}
for _name, _zone, _kinds, _relation, _other, _nonland in [
    ('OtherCreature', 'Battlefield', 1, -1, True, False),
    ('CreatureYou', 'Battlefield', 1, 0, False, False),
    ('OtherCreatureYou', 'Battlefield', 1, 0, True, False),
    ('CreatureOpponent', 'Battlefield', 1, 1, False, False),
    ('Permanent', 'Battlefield', 0, -1, False, False),
    ('Nonland', 'Battlefield', 0, -1, False, True),
    ('OtherNonlandYou', 'Battlefield', 0, 0, True, True),
    ('Land', 'Battlefield', 2, -1, False, False),
    ('Artifact', 'Battlefield', 16, -1, False, False),
    ('ArtifactCreature', 'Battlefield', 17, -1, False, False),
    ('Enchantment', 'Battlefield', 32, -1, False, False),
    ('ArtifactEnchantment', 'Battlefield', 48, -1, False, False),
    ('CreatureGraveYou', 'Graveyard', 1, 0, False, False),
    ('CreatureGrave', 'Graveyard', 1, -1, False, False),
]:
    TARGET_RULES[_name] = (f'TargetRule::Object {{ zone: TargetZone::{_zone}, kinds: {_kinds}, '
                          f'relation: {_relation}, other: {str(_other).lower()}, nonland: {str(_nonland).lower()} }}')


def rust_targets(spec):
    if spec == 'None':
        return 'TargetSpec::NONE'
    if isinstance(spec, str):
        return f'TargetSpec::one({TARGET_RULES[spec]})'
    rule, minimum, maximum = spec
    return f'TargetSpec {{ rule: {TARGET_RULES[rule]}, min: {minimum}, max: {maximum} }}'


def targeted_program(text):
    """One homogeneous target group, requiring the entire effect to compile."""
    if not isinstance(text, str) or not text.endswith('.'):
        return None
    text = text[0].lower() + text[1:]
    # Optional effects still require targets at announcement (CR603.5).
    if text.startswith('you may '):
        result = targeted_program(text[len('you may '):])
        if result:
            spec, program = result
            return spec, [f'May {{ skip: {len(program)} }}', *program]
    if m := re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on (target creature|target creature you control|another target creature you control|up to one target creature|up to one target creature you control)\.', text):
        rules = {'target creature': 'Creature', 'target creature you control': 'CreatureYou',
                 'another target creature you control': 'OtherCreatureYou',
                 'up to one target creature': ('Creature', 0, 1),
                 'up to one target creature you control': ('CreatureYou', 0, 1)}
        return rules[m[3]], [counter_instruction('Targets', m[1], m[2])]
    if m := re.fullmatch(r'tap (target creature|up to one target creature|up to two target creatures) and put (a|one|two|three) stun counters? on (it|each of them)\.', text):
        plural = m[1] == 'up to two target creatures'
        if plural != (m[3] == 'each of them'): return None
        spec = ('Creature', 0, 2) if plural else ('Creature', 0, 1) if m[1].startswith('up to') else 'Creature'
        return spec, ['Tap', counter_instruction('Targets', m[2], 'stun')]
    rules = {
        "return target creature to its owner's hand.": ('Creature', ['ReturnToHand']),
        "return target nonland permanent to its owner's hand.": ('Nonland', ['ReturnToHand']),
        "return up to one other target nonland permanent you control to its owner's hand.": (('OtherNonlandYou', 0, 1), ['ReturnToHand']),
        'untap target artifact or creature.': ('ArtifactCreature', ['Untap']),
        'untap target nonland permanent.': ('Nonland', ['Untap']),
        'untap one or two target creatures. They each get +2/+2 until end of turn.': (('Creature', 1, 2), ['Untap', 'Grow(2)']),
        "return target creature card from your graveyard to the battlefield.": ('CreatureGraveYou', ['ReturnToBattlefield']),
        "return target creature card from your graveyard to your hand.": ('CreatureGraveYou', ['ReturnToHand']),
        "return target creature card from a graveyard to its owner's hand.": ('CreatureGrave', ['ReturnToHand']),
        'destroy target creature.': ('Creature', ['Destroy']),
        'destroy target artifact.': ('Artifact', ['Destroy']),
        'destroy target enchantment.': ('Enchantment', ['Destroy']),
        'destroy target artifact or enchantment.': ('ArtifactEnchantment', ['Destroy']),
        'counter target spell.': ('Spell', ['Counter']),
    }
    if text in rules:
        return rules[text]
    numbers = {'one': 1, 'two': 2, 'three': 3}
    if m := re.fullmatch(r'(tap|untap) (up to (one|two|three) |)(other |)target (creatures?|permanents?|lands?)( you control| an opponent controls| your opponents control|)\.', text):
        action, optional, count, other, kind, relation = m.groups()
        rule = {'creature': 'Creature', 'permanent': 'Permanent', 'land': 'Land'}[kind.rstrip('s')]
        if other and kind.rstrip('s') != 'creature':
            return None
        if relation and kind.rstrip('s') != 'creature':
            return None
        if kind.rstrip('s') == 'creature':
            if relation == ' you control': rule = 'OtherCreatureYou' if other else 'CreatureYou'
            elif relation: rule = 'CreatureOpponent' if not other else None
            elif other: rule = 'OtherCreature'
        if rule is None: return None
        minimum, maximum = (0, numbers[count]) if optional else (1, 1)
        return (rule, minimum, maximum), [action.capitalize()]
    draw = r'(a card|two cards|three cards|four cards)'
    if m := re.fullmatch(r'target player draws ' + draw + r'( and loses ([0-9]+) life|)\.', text):
        program = [f'Draw(PlayerSet::Targeted, {DRAW_COUNTS[m[1]]})']
        if m[3]:
            if int(m[3]) > 65535: return None
            program.append(f'LoseLife(PlayerSet::Targeted, {int(m[3])})')
        return 'Player', program
    if m := re.fullmatch(r'target (player|opponent) discards ' + draw + r'\.', text):
        return ('Player' if m[1] == 'player' else 'Opponent'), [f'Discard(PlayerSet::Targeted, {DRAW_COUNTS[m[2]]})']
    if m := re.fullmatch(r'target (player|opponent) (gains|loses) ([0-9]+) life\.', text):
        if int(m[3]) > 65535: return None
        return ('Player' if m[1] == 'player' else 'Opponent'), [f'{"GainLife" if m[2] == "gains" else "LoseLife"}(PlayerSet::Targeted, {int(m[3])})']
    if m := re.fullmatch(r'it deals ([0-9]+) damage to (any target|target creature|target opponent|target player)\.', text):
        if int(m[1]) > 32767: return None
        return {'any target': 'Any', 'target creature': 'Creature', 'target opponent': 'Opponent', 'target player': 'Player'}[m[2]], [f'Damage({int(m[1])})']
    if m := re.fullmatch(r'target creature gets ([+-][0-9]+)/([+-][0-9]+) until end of turn\.', text):
        if m[1] != m[2] or abs(int(m[1])) > 32767: return None
        return 'Creature', [f'Grow({int(m[1])})']
    # A targeted clause followed by wholly non-targeted instructions is supported.
    if '. ' in text:
        first, rest = text.split('. ', 1)
        result = targeted_program(first + '.')
        tail = statement_program(rest)
        if result and tail:
            spec, program = result
            return spec, program + tail
    return None

def trigger_creature_template(card):
    if not isinstance(card.get('oracle_text'), str):
        return None
    # Reuse the same complete body/type/cost checks as the keyword-only template.
    if creature_template(dict(card, oracle_text='')) is None:
        return None
    keywords = 0
    abilities = []
    prefixes = {
        'When this creature enters, ': 'SelfEnters',
        'When this creature dies, ': 'SelfDies',
        'Whenever this creature attacks, ': 'SelfAttacks',
        'Whenever this creature enters or attacks, ': 'SelfEntersOrAttacks',
        'Whenever another artifact you control enters, ': 'OtherArtifactEntersYou',
        'Whenever another creature you control enters, ': 'OtherCreatureEntersYou',
        'Whenever this creature or another creature you control enters, ': 'CreatureEntersYou',
        'Landfall — Whenever a land you control enters, ': 'LandEntersYou',
    }
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    for line in text.split('\n'):
        words = {word.strip().casefold() for word in line.split(',') if word.strip()}
        if words and words <= KEYWORDS.keys():
            keywords |= sum(KEYWORDS[word] for word in words)
            continue
        for prefix, condition in prefixes.items():
            if line.startswith(prefix):
                effect_text = line[len(prefix):]
                targeted = targeted_program(effect_text)
                if targeted:
                    targets, program = targeted
                else:
                    targets, program = 'None', statement_program(effect_text)
                if program is None:
                    return None
                abilities.append((condition, program, targets))
                break
        else:
            return None
    if not abilities:
        return None
    return f"Creature({int(card['power'])}, {int(card['toughness'])})", keywords, abilities


def spell_program_template(card):
    old = spell_template(card)
    if old:
        return 'Creature', [old]
    if (card.get('layout') != 'normal' or card.get('type_line', '').split(' — ')[0] not in ('Instant', 'Sorcery')
            or fixed_cost(card.get('mana_cost', '')) is None):
        return None
    text = card.get('oracle_text')
    if isinstance(text, str) and text.startswith(card.get('name', '') + ' deals '):
        text = 'it deals ' + text[len(card['name']) + len(' deals '):]
    targeted = targeted_program(text)
    if targeted: return targeted
    program = statement_program(text)
    return ('None', program) if program else None


def activation_cost(text):
    cost = dict(colored=[0] * 5, generic=0, tap=False, sacrifice=False, life=0, remove=None)
    seen = set()
    for component in text.split(', '):
        mana = fixed_cost(component)
        if mana:
            if 'mana' in seen: return None
            seen.add('mana')
            cost['colored'], cost['generic'] = mana
        elif component == '{T}':
            if cost['tap']: return None
            cost['tap'] = True
        elif component in ('Sacrifice this creature', 'Sacrifice this artifact', 'Sacrifice this enchantment', 'Sacrifice this permanent'):
            if cost['sacrifice']: return None
            cost['sacrifice'] = True
        elif m := re.fullmatch(r'Pay ([0-9]+) life', component):
            if cost['life'] or int(m[1]) > 65535: return None
            cost['life'] = int(m[1])
        elif m := re.fullmatch(r'Remove (a|one|two|three|four|five|six) (?:(\+1/\+1|-1/-1|stun) )?counters? from this (?:creature|artifact|enchantment|permanent)', component):
            if cost['remove']: return None
            cost['remove'] = (COUNTER_KINDS[m[2]] if m[2] else None, COUNT_WORDS[m[1]])
        else:
            return None
    return cost


def permanent_program_template(card):
    """Whole supported permanent text, including all activation restrictions."""
    if (card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str)
            or fixed_cost(card.get('mana_cost', '')) is None):
        return None
    typ = card.get('type_line', '').split(' — ')[0]
    if typ not in ('Creature', 'Artifact Creature', 'Artifact', 'Enchantment'):
        return None
    creature = 'Creature' in typ
    if creature and (not card.get('power', '').isdigit() or not card.get('toughness', '').isdigit()):
        return None
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    activations, body = [], []
    enters = [0, 0, 0]
    for line in text.split('\n'):
        # These aliases denote the source only, not any arbitrary mentioned name.
        line = line.replace('When this artifact enters, ', 'When this creature enters, ').replace('When this enchantment enters, ', 'When this creature enters, ')
        if m := re.fullmatch(r'This creature enters with (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on it\.', line):
            if not creature: return None
            enters[list(COUNTER_KINDS).index(m[2])] += COUNT_WORDS[m[1]]
            continue
        if ': ' not in line:
            body.append(line)
            continue
        cost_text, effect_text = line.split(': ', 1)
        cost = activation_cost(cost_text)
        if cost is None: return None
        sorcery = effect_text.endswith(' Activate only as a sorcery.')
        once = effect_text.endswith(' Activate only once each turn.')
        if sorcery: effect_text = effect_text[:-len(' Activate only as a sorcery.')]
        if once: effect_text = effect_text[:-len(' Activate only once each turn.')]
        if effect_text.startswith('This creature deals '):
            effect_text = 'it deals ' + effect_text[len('This creature deals '):]
        targeted = targeted_program(effect_text)
        if targeted: targets, program = targeted
        else: targets, program = 'None', statement_program(effect_text)
        if program is None: return None
        activations.append((cost, targets, program, sorcery, once))
    if len(activations) > 64: return None
    body_text = '\n'.join(body).strip()
    proxy = dict(card, type_line='Creature', power=card.get('power', '0'),
                 toughness=card.get('toughness', '1'), oracle_text=body_text)
    template = creature_template(proxy)
    trigger = trigger_creature_template(proxy)
    if template: effect, keywords = template; triggers = []
    elif trigger: effect, keywords, triggers = trigger
    else: return None
    if not creature: effect = 'Permanent'
    if not activations and not any(enters) and creature: return None
    return effect, keywords, triggers, activations, enters


def rust_activations(abilities):
    rows = []
    for cost, targets, program, sorcery, once in abilities:
        remove = 'None' if cost['remove'] is None else f"Some(({'None' if cost['remove'][0] is None else 'Some(CounterKind::' + cost['remove'][0] + ')'}, {cost['remove'][1]}))"
        rows.append(f"ActivatedAbility {{ cost: ActivationCost {{ colored: {cost['colored']}, generic: {cost['generic']}, tap: {str(cost['tap']).lower()}, sacrifice: {str(cost['sacrifice']).lower()}, life: {cost['life']}, remove: {remove} }}, targets: {rust_targets(targets)}, program: {rust_program(program)}, sorcery: {str(sorcery).lower()}, once_per_turn: {str(once).lower()} }}")
    return '&[' + ', '.join(rows) + ']'


def rust_program(program):
    return '&[' + ', '.join('Instruction::' + instruction for instruction in program) + ']'

def compile_catalogue(snapshot, target):
    TOKEN_DEFS.clear()
    canonical = json.dumps(snapshot['cards'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    if hashlib.sha256(canonical).hexdigest() != snapshot['cards_sha256'] or len(snapshot['cards']) != snapshot['card_count']:
        raise ValueError('card snapshot checksum/count mismatch')
    cards = {card['name']: card for card in snapshot['cards']}
    if len(cards) != len(snapshot['cards']):
        raise ValueError('duplicate card name')
    if set(cards) & set(target['banned_card_names']):
        raise ValueError('legal snapshot and banned-list target disagree')
    unreviewed_exceptions = {name for name, card in cards.items() if 'A deck can have' in card.get('oracle_text', '')} - UNLIMITED
    if unreviewed_exceptions:
        raise ValueError(f'unreviewed deck construction exception: {sorted(unreviewed_exceptions)}')
    legacy = {name: (effect, cost, color) for name, effect, cost, color in LEGACY}
    names = list(legacy) + sorted((set(cards) | set(target['banned_card_names'])) - set(legacy))
    if len(names) > 65536:
        raise ValueError('catalogue exceeds u16 identity space')
    definitions = []
    supported = []
    coverage = []
    for index, name in enumerate(names):
        card = cards.get(name, {})
        cost = card.get('mana_cost', '')
        keywords = sum(KEYWORDS[word] for word in {keyword.casefold() for keyword in card.get('keywords', [])} & KEYWORDS.keys())
        effect = 'Unsupported'
        color = None
        template = creature_template(card)
        trigger_template = trigger_creature_template(card)
        spell = spell_program_template(card)
        permanent = permanent_program_template(card)
        activations, enters = [], [0, 0, 0]
        abilities = []
        if name in legacy:
            effect, fallback_cost, color = legacy[name]
            # Lands have no mana cost. Their legacy cost token encodes colour only.
            cost = '' if effect == 'Land' else card.get('mana_cost', fallback_cost)
            kind = 2 if effect == 'Land' else 1 if effect.startswith('Creature') else 4 if effect in ('Draw',) else 8
        else:
            types = card.get('type_line', '').split(' — ')[0].split()
            kind = (1 if 'Creature' in types else 0) | (2 if 'Land' in types else 0) | (4 if 'Sorcery' in types else 0) | (8 if 'Instant' in types else 0) | (16 if 'Artifact' in types else 0) | (32 if 'Enchantment' in types else 0)
            if template is not None:
                effect, keywords = template
            elif trigger_template is not None:
                effect, keywords, abilities = trigger_template
            elif spell is not None:
                effect = 'Spell'
            elif permanent is not None:
                effect, keywords, abilities, activations, enters = permanent
        programs = {
            'Damage(3)': ('Any', ['Damage(3)']), 'Damage(2)': ('Any', ['Damage(2)']),
            'Grow': ('Creature', ['Grow(3)']), 'Draw': ('None', ['Draw(PlayerSet::Controller, 2)']),
            'Counter': ('Spell', ['Counter']),
        }
        if effect in programs:
            rule, program = programs[effect]
            effect = f'Spell({rust_targets(rule)}, {rust_program(program)})'
        elif effect == 'Spell':
            rule, program = spell
            effect = f'Spell({rust_targets(rule)}, {rust_program(program)})'
        triggers_rust = '&[' + ', '.join(f'TriggeredAbility {{ condition: TriggerCondition::{condition}, targets: {rust_targets(targets)}, program: {rust_program(program)} }}' for condition, program, targets in abilities) + ']'

        if effect != 'Unsupported':
            supported.append(index)
        basic = name in {'Plains', 'Island', 'Swamp', 'Mountain', 'Forest'}
        colored, generic = fixed_cost(cost) or ([0] * 5, 0)
        deck_limit = 'None' if basic or name in UNLIMITED else 'Some(4)'
        quote = lambda value: json.dumps(value, ensure_ascii=False)
        oracle_id = f"Some({quote(card['oracle_id'])})" if card else 'None'
        type_line = card.get('type_line', 'Basic Land' if basic else 'Creature' if kind & 1 else 'Instant' if kind & 8 else 'Sorcery' if kind & 4 else '')
        oracle_keywords = '&[' + ', '.join(quote(word) for word in card.get('keywords', [])) + ']'
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {quote(name)}, oracle_id: {oracle_id}, mana_cost: {quote(cost)}, type_line: {quote(type_line)}, oracle_keywords: {oracle_keywords}, standard_legal: {str(name in cards).lower()}, implemented: {str(effect != 'Unsupported').lower()}, keywords: Keywords({keywords}), deck_limit: {deck_limit} }}, effect: Effect::{effect}, triggers: {triggers_rust}, activated: {rust_activations(activations)}, enters: Counters {{ plus_one: {enters[0]}, minus_one: {enters[1]}, stun: {enters[2]} }}, kind: {kind}, basic: {str(basic).lower()}, mana_color: {'None' if color is None else f'Some({color})'}, colored: {colored}, generic: {generic} }},")
        coverage.append({'name': name, 'standard_legal': name in cards, 'implemented': effect != 'Unsupported',
                         'implementation': 'explicit' if name in legacy else 'creature_template' if template else 'trigger_template' if trigger_template else 'spell_template' if spell else 'permanent_template' if permanent else None})

    # Only factories referenced by wholly accepted cards enter the executable catalogue.
    used_tokens = sorted(set(re.findall(r'__TOKEN_([0-9a-f]+)__', '\n'.join(definitions))))
    token_base = len(names)
    token_infos = []
    for offset, key in enumerate(used_tokens):
        factory = TOKEN_DEFS[key]
        index = token_base + offset
        name = '@token/' + key
        names.append(name)
        types = ((factory['extra_type'].title() + ' ') if factory['extra_type'] else '') + 'Creature'
        type_line = types + ' — ' + factory['subtypes']
        kind = 1 | (16 if factory['extra_type'] == 'artifact' else 32 if factory['extra_type'] == 'enchantment' else 0)
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {json.dumps(name)}, oracle_id: None, mana_cost: \"\", type_line: {json.dumps(type_line, ensure_ascii=False)}, oracle_keywords: &[], standard_legal: false, implemented: true, keywords: Keywords({factory['keywords']}), deck_limit: None }}, effect: Effect::Creature({factory['power']}, {factory['toughness']}), triggers: &[], activated: &[], enters: Counters {{ plus_one: 0, minus_one: 0, stun: 0 }}, kind: {kind}, basic: false, mana_color: None, colored: [0; 5], generic: 0 }},")
        token_infos.append(f"    TokenInfo {{ name: {json.dumps(factory['name'])}, subtypes: {json.dumps(factory['subtypes'])}, colors: {factory['colors']} }},")
        marker = '__TOKEN_' + key + '__'
        definitions = [definition.replace(marker, f'Card({index})') for definition in definitions]
    if len(names) > 65536:
        raise ValueError('catalogue exceeds u16 identity space')
    # The fingerprint covers data and compiler logic; card IDs are local to it.
    target_bytes = json.dumps(target, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    version = hashlib.sha256(canonical + target_bytes + Path(__file__).read_bytes()).hexdigest()
    lines = ['// Generated by scripts/generate_catalogue.py. Do not edit.',
             f'pub const CATALOGUE_VERSION: &str = "{version}";',
             '#[rustfmt::skip]', f'pub(super) static DEFINITIONS: [Definition; {len(names)}] = [', *definitions, '];',
             '#[rustfmt::skip]', f'pub(super) static NAME_INDEX: [(&str, u16); {len(names)}] = [']
    lines += [f'    ({json.dumps(name, ensure_ascii=False)}, {index}),' for name, index in sorted((name, index) for index, name in enumerate(names))]
    lines += ['];', f'pub(super) const TOKEN_BASE: usize = {token_base};',
              '#[rustfmt::skip]', f'pub(super) static TOKENS: [TokenInfo; {len(token_infos)}] = [', *token_infos, '];',
              '#[allow(non_upper_case_globals)]', 'impl Card {', '#[rustfmt::skip]',
              f'    pub const ALL: [Card; {len(supported)}] = [{", ".join(f"Card({i})" for i in supported)}];']
    # Preserve existing Rust constants and JSON tokens. New callers use English names.
    for index, (name, _, _, _) in enumerate(LEGACY):
        lines.append(f'    pub const {name.replace(" ", "")}: Card = Card({index});')
    lines += ['}', '']
    return '\n'.join(lines), {'schema': 1, 'catalogue_version': version, 'standard_legal_cards': len(cards),
        'implemented_standard_cards': sum(row['implemented'] and row['standard_legal'] for row in coverage),
        'implemented_total': len(supported), 'token_definitions': len(used_tokens), 'cards': coverage}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    snapshot = json.loads((ROOT / 'data/standard/cards.json').read_text())
    target = json.loads((ROOT / 'data/targets/standard-2026-10-15.json').read_text())
    rust, coverage = compile_catalogue(snapshot, target)
    outputs = {ROOT / 'src/generated/catalogue.rs': rust,
               ROOT / 'data/standard/coverage.json': json.dumps(coverage, ensure_ascii=False, indent=2) + '\n'}
    for path, content in outputs.items():
        if args.check:
            if not path.exists() or path.read_text() != content:
                raise SystemExit(f'outdated generated file: {path.relative_to(ROOT)}')
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
    print(json.dumps({key: value for key, value in coverage.items() if key != 'cards'}))
