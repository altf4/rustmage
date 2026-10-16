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
CREATURE_TYPE_NAMES = tuple(json.loads((ROOT / 'data/creature-types.json').read_text())['types'])
EMBLEM_DEFS = {}
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
            'lifelink': 64, 'deathtouch': 128, 'trample': 256, 'first strike': 512, 'double strike': 1024, 'hexproof': 4096, 'indestructible': 8192, 'menace': 32768, 'skulk': 1048576}
INTRINSIC_KEYWORDS = dict(KEYWORDS, changeling=524288)
POWER_BLOCKING_TEXT_FLAGS = {
    "this creature can't be blocked by creatures with power 2 or less.": 2097152,
    "this creature can't be blocked by creatures with power 3 or greater.": 4194304,
    "this creature can't be blocked by creatures with greater power.": 8388608,
}
TEXT_FLAGS = {**POWER_BLOCKING_TEXT_FLAGS, "this creature can't block.": 2048, "this creature can't be blocked.": 16384,
              "this creature can't be blocked by more than one creature.": 65536,
              "this creature can't be blocked except by three or more creatures.": 131072,
              "this creature can't be blocked by tokens.": 262144}
UNLIMITED = {'Cid, Timeless Artificer', 'Hare Apparent', 'Slime Against Humanity', "Sphinx's Approach", 'Tempest Hawk'}


def fixed_cost(cost):
    if not re.fullmatch(r'(?:\{(?:[0-9]+|[WUBRGC])\})+', cost):
        return None
    colored = [0] * 6
    generic = 0
    for symbol in re.findall(r'\{([^}]+)\}', cost):
        if symbol.isdigit():
            generic += int(symbol)
        else:
            colored['WUBRGC'.index(symbol)] += 1
    return colored, generic



def spell_cost_options(cost):
    """Unique exact equivalents; two-brid generic choices retain their full cost."""
    if not re.fullmatch(r'(?:\{(?:[0-9]+|[WUBRGC]|[WUBRG]/[WUBRG]|2/[WUBRG])\})+',cost): return None
    allowed={'W/U','W/B','U/B','U/R','B/R','B/G','R/G','R/W','G/W','G/U'}
    variants={(0,0,0,0,0,0,0)}
    for symbol in re.findall(r'\{([^}]+)\}',cost):
        if symbol.isdigit(): choices=[(6,int(symbol))]
        elif symbol.startswith('2/'): choices=[(6,2),('WUBRGC'.index(symbol[2]),1)]
        else:
            if '/' in symbol:
                if symbol not in allowed: return None
                colors=symbol.split('/')
            else: colors=[symbol]
            choices=[('WUBRGC'.index(color),1) for color in colors]
        variants={tuple(n+(amount if index==component else 0) for index,n in enumerate(row))
                  for row in variants for component,amount in choices}
        if len(variants)>65536 or any(max(row)>4294967295 for row in variants): return None
    # Colored components sort first, so the canonical choice uses the generic
    # side of every two-brid symbol. Its total remains the printed mana value.
    return [(list(row[:6]),row[6]) for row in sorted(variants)]

def reduction_filter(noun):
    if noun=="Lesson": return 'CastFilter::Subtype("Lesson")'
    words={'': 'Any','creature':'Creature','noncreature':'Noncreature','artifact':'Artifact',
           'enchantment':'Enchantment','instant and sorcery':'InstantSorcery'}
    if noun.casefold() in words:return 'CastFilter::'+words[noun.casefold()]
    subtypes={word.casefold():word for word in CREATURE_TYPE_NAMES}
    if ' spells and ' in noun:
        nouns=noun.split(' spells and ')
        if len(nouns)!=2 or any(n.casefold() not in subtypes for n in nouns):return None
        return 'CastFilter::AnyOf(&['+', '.join('CastFilter::Subtype('+json.dumps(subtypes[n.casefold()])+')' for n in nouns)+'])'
    if noun.casefold() in subtypes:return 'CastFilter::Subtype('+json.dumps(subtypes[noun.casefold()])+')'
    return None

def reduction_quantity(text):
    if text.casefold() in ('outlaws you control','assassin, mercenary, pirate, rogue, and/or warlock you control'):
        filters=[stat_count_filter(noun+' you control') for noun in ['Assassin','Mercenary','Pirate','Rogue','Warlock']]
        return 'StatValue::CountUnion(&['+', '.join(filters)+'])'
    if text.startswith('color among '):text='colors among '+text[len('color among '):]
    if text.endswith(' you own in exile and in your graveyard'):
        noun=text[:-len(' you own in exile and in your graveyard')]
        values=[stat_value(noun+' you own in exile'),stat_value(noun+' in your graveyard')]
        return 'StatValue::Sum(&['+', '.join(values)+'])' if all(values) else None
    return stat_value(text)

def derived_reduction(text,card):
    text=text.removeprefix('the ')
    if m:=re.fullmatch(r'(greatest power among|total power of) (.+)',text):
        filt=creature_filter(m[2])
        if filt:return f'SpellReductionAmount::CreaturePower {{ filter: {filt}, greatest: {str(m[1].startswith("greatest")).lower()} }}'
    if card.get('name') and text==card['name']+"'s power":return 'SpellReductionAmount::SourcePower'
    return None

def extract_cost_reductions(card):
    text=card.get('oracle_text')
    if not isinstance(text,str) or card.get('layout')!='normal':return card,[]
    body,rules=[],[]
    for original in text.split('\n'):
        line=original.strip();turn='Any';guard='StepGuard::Always';source_only=False;filt=None;amount=None;quantity=None;derived=None
        for prefix,timing in [('During turns other than yours, ','Other'),('During your turn, ','Own')]:
            if line.startswith(prefix):turn=timing;line=line[len(prefix):];break
        if m:=re.fullmatch(r'Affinity for ([A-Za-z -]+)(?: \(This spell costs \{1\} less to cast for each (.+) you control\.\))?',line):
            noun=m[1]
            candidate=reduction_quantity(noun.casefold()+' you control')
            if m[2] is None or candidate==reduction_quantity(m[2].casefold()+' you control'):
                quantity=candidate;amount=1;source_only=True;filt='CastFilter::Any'
        elif m:=re.fullmatch(r'(?:Vivid — )?This spell costs \{([0-9]+)\} less to cast for each (.+)\.',line):
            quantity=reduction_quantity(m[2]);amount=int(m[1]);source_only=True;filt='CastFilter::Any'
            if m[2]=='creature that attacked this turn':derived=f'SpellReductionAmount::Attackers {{ factor: {amount} }}'
        elif m:=re.fullmatch(r'This spell costs \{X\} less to cast, where X is (?:the number of )?(.+)\.',line):
            quantity=reduction_quantity(m[1].removeprefix('the '));amount=1;source_only=True;filt='CastFilter::Any'
            if quantity is None:derived=derived_reduction(m[1],card)
        elif m:=re.fullmatch(r'([A-Za-z -]+) spells you cast with mana value ([0-9]+) or greater cost \{X\} less to cast, where X is (.+)\.',line):
            base_filter=reduction_filter(m[1]);minimum=int(m[2]);derived=derived_reduction(m[3],card);amount=1
            if base_filter and minimum<=18446744073709551615:
                filt=f'CastFilter::ManaValueMinimum {{ filter: &{base_filter}, minimum: {minimum} }}'
        elif m:=re.fullmatch(r'(?:([A-Za-z -]+) )?[Ss]pells you cast have affinity for ([A-Za-z -]+)\.(?: \(They cost \{1\} less to cast for each ([A-Za-z -]+) you control\.\))?',line):
            candidate=reduction_quantity(m[2].casefold()+' you control')
            if m[3] is None or candidate==reduction_quantity(m[3].casefold()+' you control'):
                quantity=candidate;amount=1;filt=reduction_filter(m[1] or '')
        elif m:=re.fullmatch(r'(?:([A-Za-z -]+) )?[Ss]pells you cast cost \{([0-9]+)\} less to cast\.',line):
            filt=reduction_filter(m[1] or '');amount=int(m[2])
        elif m:=re.fullmatch(r'The second spell you cast each turn costs \{([0-9]+)\} less to cast\.',line):
            filt='CastFilter::Any';amount=int(m[1]);guard='StepGuard::Threshold { value: StepValue::SpellsCast, minimum: Some(1), maximum: Some(1) }'
        elif m:=re.fullmatch(r'This spell costs \{([0-9]+)\} less to cast(?: if (.+)| during your turn)?\.',line):
            source_only=True;filt='CastFilter::Any';amount=int(m[1])
            if m[2]:
                guard=('StepGuard::Threshold { value: StepValue::SpellsCast, minimum: Some(1), maximum: None }'
                       if m[2]=="you've cast another spell this turn" else step_guard(m[2]))
                # CR 601.2a moves the proposed spell out of hand before costs
                # are determined. Affordability currently queries the original
                # hand, so retain this clause until proposed-state checks exist.
                if guard and 'StepValue::Hand' in guard:guard=None
            elif line.endswith(' during your turn.'):turn='Own'
        scaled=('Affinity for ' in line or 'affinity for ' in line or ' less to cast for each ' in line or 'less to cast, where X is ' in line)
        if filt is None or guard is None or amount is None or amount>65535 or (scaled and quantity is None and derived is None):
            body.append(original);continue
        # Cost predicates/quantities must not inspect the pre-proposal hand.
        # stat_value has only battlefield/graveyard/exile count vocabulary.
        amount=(derived or (f'SpellReductionAmount::Stat {{ value: {quantity}, factor: {amount} }}' if quantity
                else f'SpellReductionAmount::Fixed({amount})'))
        rules.append(f'SpellReduction {{ source_only: {str(source_only).lower()}, filter: {filt}, amount: {amount}, turn: CostTurn::{turn}, guard: {guard} }}')
    return dict(card,oracle_text='\n'.join(body)),rules

def permanent_type(card):
    typ = card.get('type_line', '').split(' — ')[0]
    return typ.removeprefix('Legendary ').removeprefix('Kindred ')


def creature_template(card):
    # Missing/unknown rules text is not a blank text box. A fresh checksum only
    # establishes data integrity; it cannot establish omitted card semantics.
    if not isinstance(card.get('oracle_text'), str):
        return None
    if card.get('layout') != 'normal' or permanent_type(card) not in ('Creature', 'Artifact Creature', 'Enchantment Creature'):
        return None
    if not card.get('power', '').isdigit() or not card.get('toughness', '').isdigit():
        return None
    if spell_cost_options(card.get('mana_cost', '')) is None:
        return None
    # Remove only balanced, unnested reminder text; fail closed on residual text.
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    keywords = 0
    body = []
    for line in text.split('\n'):
        if line.casefold() in TEXT_FLAGS: keywords |= TEXT_FLAGS[line.casefold()]
        else: body.append(line)
    words = {word.strip().casefold() for word in ','.join(body).split(',') if word.strip()}
    if not words <= INTRINSIC_KEYWORDS.keys(): return None
    return f"Creature({int(card['power'])}, {int(card['toughness'])})", keywords | sum(INTRINSIC_KEYWORDS[word] for word in words)



def spell_template(card):
    if (card.get('layout') != 'normal' or card.get('type_line', '').split(' — ')[0] not in ('Instant', 'Sorcery')
            or spell_cost_options(card.get('mana_cost', '')) is None):
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

def amount_value(text):
    if text == "the number of cards you've discarded this turn": return 'AmountValue::CardsDiscardedThisTurn'
    if text == 'the amount of life you gained this turn': return 'AmountValue::LifeGainedThisTurn'
    if ' plus the number of ' in text:
        parts=text.split(' plus the number of ')
        values=[stat_value(part.removeprefix('the number of ')) for part in parts]
        return 'AmountValue::Stat(StatValue::Sum(&['+', '.join(values)+']))' if all(values) else None
    if text.startswith('the number of '):text=text[len('the number of '):]
    value=stat_value(text)
    return f'AmountValue::Stat({value})' if value else None


def scaled_damage_program(text):
    targets={'any target':'Any','target creature':'Creature',
             'target creature an opponent controls':'CreatureOpponent',
             'target creature or planeswalker':'CreaturePlaneswalker',
             'target opponent':'Opponent','target player':'Player'}
    source=r'this (?:creature|artifact|enchantment|permanent|spell) deals '
    amount,target=None,None
    if m:=re.fullmatch(source+r'damage equal to (.+) to (.+)\.',text):
        amount,target=m.groups()
    elif m:=re.fullmatch(source+r'damage to (.+) equal to (.+)\.',text):
        target,amount=m.groups()
    elif m:=re.fullmatch(source+r'X damage to (.+), where X is (.+)\.',text):
        target,amount=m.groups()
    value=amount_value(amount) if amount else None
    return (targets[target],[f'DamageScaled({value})']) if target in targets and value else None


def scaled_life_program(text):
    if m:=re.fullmatch(r'you gain life equal to (.+)\.',text):
        value=amount_value(m[1])
    elif m:=re.fullmatch(r'you gain X life, where X is (.+)\.',text):
        value=amount_value(m[1])
    elif m:=re.fullmatch(r'you gain ([0-9]+) life for each (.+)\.',text):
        value=amount_value(m[2]);factor=int(m[1])
        if not 1<=factor<=65535:return None
        if value and factor!=1:value=f'AmountValue::Multiple {{ value: &{value}, factor: {factor} }}'
    else:return None
    return [f'GainLifeScaled(PlayerSet::Controller, {value})'] if value else None


def scaled_token_program(text):
    body,value=None,None
    if m:=re.fullmatch(r'create X (.+), where X is (.+)\.',text):
        body='a '+m[1];value=amount_value(m[2])
    elif m:=re.fullmatch(r'for each (.+), create (.+)\.',text):
        value=amount_value(m[1]);body=m[2]
    elif m:=re.fullmatch(r'create (.+) for each (.+)\.',text):
        body=m[1];value=amount_value(m[2])
    if value is None or body is None or not re.match(r'(?:a|an|one) ',body): return None
    program=token_program('create '+body+'.') or predefined_program('create '+body+'.')
    if not program or len(program)!=1 or not program[0].startswith('CreateTokens {'):return None
    return [re.sub(r'amount: [0-9]+', 'amount: '+value, program[0],count=1).replace('CreateTokens {','CreateTokensScaled {',1)]


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
        if ability in ('"This token can\'t block."', '"This token can\'t be blocked."'):
            keywords = 2048 if ability == '"This token can\'t block."' else 16384
        else:
            if not ability.endswith('.'): return None
            words = [word.strip() for word in ability[:-1].replace(' and ', ',').split(',')]
            if not words or not all(word in INTRINSIC_KEYWORDS for word in words): return None
            keywords = sum(INTRINSIC_KEYWORDS[word] for word in set(words))
    factory = {'name': name, 'subtypes': subtypes, 'colors': sum(colors[word] for word in color_names),
               'power': int(power), 'toughness': int(toughness), 'extra_type': extra_type,
               'keywords': keywords}
    key = hashlib.sha256(json.dumps(factory, sort_keys=True).encode()).hexdigest()[:24]
    if key in TOKEN_DEFS and TOKEN_DEFS[key] != factory:
        raise ValueError("token factory identity collision")
    TOKEN_DEFS[key] = factory
    return [f'CreateTokens {{ token: __TOKEN_{key}__, amount: {TOKEN_COUNTS[count]}, tapped: {str(bool(tapped)).lower()} }}']


def mana_program(text):
    if not isinstance(text, str): return None
    if m := re.fullmatch(r'add ((?:\{[WUBRGC]\})+)\.', text):
        mana = [0] * 6
        for symbol in re.findall(r'\{([WUBRGC])\}', m[1]): mana['WUBRGC'.index(symbol)] += 1
        return [f'AddMana(ManaProduction::Fixed({mana}))']
    if m := re.fullmatch(r'add (one|two|three) mana of any (color|one color|type)\.', text):
        mask = 63 if m[2] == 'type' else 31
        return [f'AddMana(ManaProduction::Choice {{ mask: {mask}, amount: {COUNT_WORDS[m[1]]} }})']
    if re.fullmatch(r'add \{[WUBRGC]\}(?:, \{[WUBRGC]\})*(?:,? or \{[WUBRGC]\})\.', text):
        mask = sum(1 << 'WUBRGC'.index(symbol) for symbol in set(re.findall(r'\{([WUBRGC])\}', text)))
        return [f'AddMana(ManaProduction::Choice {{ mask: {mask}, amount: 1 }})']
    return None

PREDEFINED = {
    'Treasure': '{T}, Sacrifice this artifact: Add one mana of any color.',
    'Gold': 'Sacrifice this artifact: Add one mana of any color.',
    'Food': '{2}, {T}, Sacrifice this artifact: You gain 3 life.',
    'Clue': '{2}, Sacrifice this artifact: Draw a card.',
    'Mutagen': '{1}, {T}, Sacrifice this artifact: Put a +1/+1 counter on target creature. Activate only as a sorcery.',
    'Heartwood': '{T}: Add {R} or {G}.',
    'Lander': '{2}, {T}, Sacrifice this artifact: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.',
}

def predefined_program(text):
    m = re.fullmatch(r'create (a|an|one|two|three|four|five|six|seven|eight) (tapped )?(Treasure|Gold|Food|Clue|Mutagen|Heartwood|Lander) tokens?\.', text)
    if not m: return None
    count, tapped, kind = m.groups()
    factory = {'name': kind + ' Token', 'subtypes': kind, 'colors': 24 if kind == 'Heartwood' else 0,
               'power': None, 'toughness': None, 'extra_type': 'artifact', 'keywords': 0, 'predefined': kind}
    key = hashlib.sha256(json.dumps(factory, sort_keys=True).encode()).hexdigest()[:24]
    TOKEN_DEFS[key] = factory
    return [f'CreateTokens {{ token: __TOKEN_{key}__, amount: {TOKEN_COUNTS[count]}, tapped: {str(bool(tapped)).lower()} }}']


SUBTYPE_PLURALS = {'elves': 'Elf', 'goblins': 'Goblin', 'dinosaurs': 'Dinosaur', 'dragons': 'Dragon',
                   'allies': 'Ally', 'turtles': 'Turtle', 'soldiers': 'Soldier', 'zombies': 'Zombie',
                   'vampires': 'Vampire', 'merfolk': 'Merfolk', 'humans': 'Human', 'cats': 'Cat',
                   'dogs': 'Dog', 'rabbits': 'Rabbit', 'birds': 'Bird', 'bats': 'Bat', 'mice': 'Mouse',
                   'kithkin': 'Kithkin', 'knights': 'Knight', 'warriors': 'Warrior', 'spirits': 'Spirit',
                   'giants': 'Giant', 'heroes': 'Hero', 'villains': 'Villain', 'faeries': 'Faerie',
                   'spiders': 'Spider', 'otters': 'Otter', 'squirrels': 'Squirrel', 'mounts': 'Mount',
                   'insects': 'Insect', 'rats': 'Rat', 'snakes': 'Snake', 'wizards': 'Wizard', 'clerics':'Cleric', 'angels':'Angel',
                   'slivers':'Sliver', 'frogs':'Frog', 'lizards':'Lizard'}


def creature_filter(text):
    text = text.casefold()
    if text.startswith('each '): text = text[5:]
    if text.startswith('all '): text = text[4:]
    other = text.startswith('other ')
    if other: text = text[6:]
    attacking = text.startswith('attacking ')
    if attacking: text = text[10:]
    keyword = 0
    if ' with ' in text:
        text, word = text.split(' with ', 1)
        if word not in KEYWORDS: return None
        keyword = KEYWORDS[word]
    relation = -1
    if text.endswith(' you control'): relation, text = 0, text[:-12]
    elif text.endswith(' your opponents control'): relation, text = 1, text[:-23]
    singulars={name.casefold():plural for plural,name in SUBTYPE_PLURALS.items()}
    singulars.update({'creature':'creatures','creature token':'creature tokens','nontoken creature':'nontoken creatures','artifact creature':'artifact creatures'})
    text=singulars.get(text,text)
    subtype, tokens, artifact = 'None', 'None', False
    if text == 'creatures': pass
    elif text == 'creature tokens': tokens = 'Some(true)'
    elif text == 'nontoken creatures': tokens = 'Some(false)'
    elif text == 'artifact creatures': artifact = True
    elif text == 'nontoken artifact creatures': artifact,tokens=True,'Some(false)'
    elif text in SUBTYPE_PLURALS: subtype = 'Some(' + json.dumps(SUBTYPE_PLURALS[text]) + ')'
    else: return None
    return (f'CreatureFilter {{ relation: {relation}, other: {str(other).lower()}, subtype: {subtype}, '
            f'keyword: Keywords({keyword}), attacking: {str(attacking).lower()}, tokens: {tokens}, artifact: {str(artifact).lower()} }}')


def grow_instruction(subject, power, toughness):
    return f'Grow {{ subject: Subject::{subject}, power: {power}, toughness: {toughness} }}'


# Count expressions never inspect derived power/toughness, avoiding layer recursion.
def stat_count_filter(text):
    text=text.casefold()
    other=text.startswith('other ')
    if other:text=text[6:]
    zone,cards='Battlefield',False
    if text.endswith(' you control'): noun=text[:-12]
    elif text.endswith(' in your graveyard'): noun=text[:-18];zone,cards='Graveyard',True
    elif text.endswith(' you own in exile'): noun=text[:-len(' you own in exile')];zone,cards='Exile',True
    else:return None
    kinds={'creature':1,'creatures':1,'land':2,'lands':2,'artifact':16,'artifacts':16,
           'enchantment':32,'enchantments':32,'permanent':1|2|16|32|64|256,'permanents':1|2|16|32|64|256,
           'instant and sorcery':4|8,'instant and/or sorcery':4|8,'creature and planeswalker':1|64,'artifact and/or creature':16|1}
    kind,exclude,basic,subtype=0,0,False,None
    if cards:
        if noun.endswith(' cards'):noun=noun[:-6]
        elif noun.endswith(' card'):noun=noun[:-5]
        elif noun!='cards':return None
    if zone=='Battlefield' and noun.startswith('nontoken '):
        noun=noun[len('nontoken '):];cards=True
    if noun=='cards' and zone=='Battlefield':return None
    if noun in ('cards','permanents'):pass
    elif noun=='nonland permanents':exclude=2
    elif noun=='noncreature, nonland':exclude=1|2
    elif noun in kinds:kind=kinds[noun]
    else:
        if noun.startswith('basic '):basic=True;noun=noun[6:]
        singular={word.casefold():word for word in [*CREATURE_TYPE_NAMES,'Plains','Island','Swamp','Mountain','Forest','Equipment','Desert','Gate','Cave','Town','Treasure','Lesson']}
        plurals={word.casefold()+'s':word for word in ['Island','Swamp','Mountain','Forest','Desert','Gate','Cave','Town','Treasure']}
        subtype=SUBTYPE_PLURALS.get(noun,plurals.get(noun,singular.get(noun)))
        if subtype is None:return None
        if basic and subtype not in ('Plains','Island','Swamp','Mountain','Forest'):return None
    return f'CountFilter {{ zone: CountZone::{zone}, relation: 0, kind: {kind}, exclude: {exclude}, subtype: {"None" if subtype is None else "Some("+json.dumps(subtype)+")"}, basic: {str(basic).lower()}, other: {str(other).lower()}, cards_only: {str(cards).lower()} }}'


def stat_value(text):
    if text=='tokens you control':
        return 'StatValue::Tokens('+stat_count_filter('permanents you control')+')'
    if text=='your life total':return 'StatValue::ControllerLife'
    if text=='card types among cards in all graveyards':return 'StatValue::GraveyardCardTypes'
    if text.startswith('differently named '):
        filt=stat_count_filter(text[len('differently named '):])
        return f'StatValue::DistinctNames({filt})' if filt else None
    for prefix,variant in [('colors among ','Colors'),('card types among ','CardTypes')]:
        if text.startswith(prefix):
            filt=stat_count_filter(text[len(prefix):])
            return f'StatValue::{variant}({filt})' if filt else None
    if text.startswith('greatest mana value among '):
        filt=stat_count_filter(text[len('greatest mana value among '):])
        return f'StatValue::GreatestManaValue({filt})' if filt else None
    if ' and each ' in text:
        values=[stat_value(part) for part in text.split(' and each ')]
        return 'StatValue::Sum(&['+', '.join(values)+'])' if all(values) else None
    filt=stat_count_filter(text)
    return f'StatValue::Count({filt})' if filt else None


def extract_uncounterable(card):
    """Only a standalone self-spell prohibition; retain the whole residual body."""
    if card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str):
        return card, False
    body = []; found = False
    for original in card['oracle_text'].split('\n'):
        line = re.sub(r'\([^()]*\)', '', original).strip()
        if line == "This spell can't be countered.":
            if found: return card, False
            found = True
        else: body.append(original)
    return dict(card, oracle_text='\n'.join(body)), found


def extract_creature_entry_tapped(card):
    """One exact self-entry replacement, with its entire residual body retained."""
    if (card.get('layout')!='normal' or 'Creature' not in card.get('type_line','').split(' — ')[0].split()
            or not isinstance(card.get('oracle_text'),str)):
        return card,'Never'
    body=[];entry='Never'
    for original in card['oracle_text'].split('\n'):
        line=re.sub(r'\([^()]*\)','',original).strip()
        if line=='This creature enters tapped.':rule='Always';replacement=None
        elif line=="This creature enters tapped if it's not your turn.":rule='OpponentsTurn';replacement=None
        elif m:=re.fullmatch(r'This creature enters tapped with (a|one|two|three|four|five|six) stun counters? on it\.',line):
            rule='Always';replacement=line.replace('enters tapped with','enters with',1)
        else:body.append(original);continue
        # Multiple replacement clauses require explicit composition support.
        if entry!='Never':return card,'Never'
        entry=rule
        if replacement:body.append(replacement)
    return dict(card,oracle_text='\n'.join(body)),entry


def extract_entry_counters(card):
    """Exact pre-entry replacements; only emitted after the residual body compiles."""
    if (card.get('layout') != 'normal' or 'Creature' not in card.get('type_line', '').split(' — ')[0].split()
            or not isinstance(card.get('oracle_text'), str)):
        return card, []
    body, rules = [], []
    for line in card['oracle_text'].split('\n'):
        if line == 'Raid — This creature enters with a +1/+1 counter on it if you attacked this turn.':
            rules.append('EntryCounterRule { guard: StepGuard::Attacked, amount: EntryCounterAmount::Fixed(1) }')
        elif line == 'This creature enters with a +1/+1 counter on it if an opponent lost life this turn.':
            rules.append('EntryCounterRule { guard: StepGuard::OpponentLostLife, amount: EntryCounterAmount::Fixed(1) }')
        elif line == 'This creature enters with a +1/+1 counter on it if a creature died this turn.':
            rules.append('EntryCounterRule { guard: StepGuard::CreatureDied, amount: EntryCounterAmount::Fixed(1) }')
        elif line == "This creature enters with two +1/+1 counters on it if you've cast two or more spells this turn.":
            rules.append('EntryCounterRule { guard: StepGuard::Threshold { value: StepValue::SpellsCast, minimum: Some(2), maximum: None }, amount: EntryCounterAmount::Fixed(2) }')
        elif line == 'This creature enters with a +1/+1 counter on it for each other creature and/or artifact you control.':
            filt = 'CountFilter { zone: CountZone::Battlefield, relation: 0, kind: 17, exclude: 0, subtype: None, basic: false, other: true, cards_only: false }'
            rules.append('EntryCounterRule { guard: StepGuard::Always, amount: EntryCounterAmount::Stat(StatValue::Count(' + filt + ')) }')
        else:
            body.append(line)
    return dict(card, oracle_text='\n'.join(body)), rules


def stat_threshold_condition(text):
    """Boolean source-stat predicates over earlier-layer counts or owned history."""
    if text=="it's attacking":return 'StatValue::SourceAttacking'
    if text=="it's modified":return 'StatValue::SourceModified'
    if text=="you haven't cast a spell this turn":return 'StatValue::Not(&StatValue::SpellsCastThisTurn)'
    if text=="you've drawn two or more cards this turn":value,minimum='StatValue::CardsDrawnThisTurn',2
    elif text=="there's a Lesson card in your graveyard":value,minimum=stat_value('Lesson cards in your graveyard'),1
    elif text=='you control a token':value,minimum=stat_value('tokens you control'),1
    elif m:=re.fullmatch(r'you have ([0-9]+) or more life',text):value,minimum='StatValue::ControllerLife',int(m[1])
    elif text=="you've cast two or more spells this turn":value,minimum='StatValue::SpellsCastThisTurn',2
    elif m:=re.fullmatch(r'you control (?:a|an|another) (.+)',text):
        value=stat_value(('other ' if text.startswith('you control another ') else '')+m[1]+' you control');minimum=1
    elif m:=re.fullmatch(r'(?:you control|there are) (two|three|four|five|six|seven|eight|[0-9]+) or more (.+)',text):
        minimum={**COUNT_WORDS,'seven':7,'eight':8}.get(m[1],int(m[1]) if m[1].isdigit() else 0)
        value=stat_value(m[2]+(' you control' if text.startswith('you control ') else ''))
    else:return None
    if value is None or minimum>65535:return None
    return f'StatValue::AtLeast {{ value: &{value}, minimum: {minimum} }}'


def source_rule_line(line, names):
    """Normalize only exact source subjects, including a leading condition."""
    line=strip_rule_label(line)
    for alias in names:
        for verb in ('gets ', 'has ', "can't be blocked"):
            line=re.sub(r'(^|, )'+re.escape(alias)+r' '+re.escape(verb),lambda m:m[1]+('This creature ' if not m[1] else 'this creature ')+verb,line)
    return line


def source_grant_mask(words):
    restrictions={"can't be blocked":16384,"can't be blocked by more than one creature":65536}
    return restrictions.get(words,keyword_list(words.casefold()))


def extract_stat_rules(card):
    """Extract exact source/attached scaling and characteristic clauses only.
    The residual card must still compile in full before any rule is emitted.
    """
    if card.get('layout')!='normal' or not isinstance(card.get('oracle_text'),str):return card,None,[]
    body,scales=[],[]
    base=None
    name=card.get('name','')
    names=[name] if name else []
    if name and card.get('type_line','').startswith('Legendary '):names.append(name.split(',')[0])
    types=card.get('type_line','').split(' — ')[0].split()
    attached=card.get('type_line','').split(' — ')[-1] in ('Aura','Equipment')
    for original in re.sub(r'\([^()]*\)', '',card['oracle_text']).strip().split('\n'):
        line=original.strip()
        # Ability words have no rules meaning; only strip for a recognized CDA.
        cda=re.sub(r'^[A-Za-z][A-Za-z\x27 -]+ — ', '',line)
        for alias in names:
            if cda.startswith(alias+"'s "):cda="This creature's "+cda[len(alias+"'s "):]
            if line.startswith(alias+' gets '):line='This creature gets '+line[len(alias+' gets '):]
        m=re.fullmatch(r"This creature's (power and toughness are each|power is) equal to (.+)\.",cda)
        if m and 'Creature' in types:
            if base is not None:return card,None,[]
            axes,value=m.groups();offset=0
            if value.endswith(' and its toughness is equal to that number plus 1'):
                if axes!='power is':return card,None,[]
                axes='both_offset';offset=1;value=value[:-len(' and its toughness is equal to that number plus 1')]
            multiplier=1
            if value.startswith('twice the number of '):multiplier=2;value=value[len('twice the number of '):]
            elif value.startswith('the number of '):value=value[len('the number of '):]
            elif value.startswith('the greatest mana value among '):value=value[len('the '):]
            else:body.append(original);continue
            parsed=stat_value(value)
            if parsed is None:body.append(original);continue
            expected_t='1+*' if offset else '*' if axes!='power is' else card.get('toughness')
            if card.get('power')!='*' or card.get('toughness')!=expected_t:return card,None,[]
            expr=lambda add:f'Some(StatExpr {{ value: {parsed}, multiplier: {multiplier}, offset: {add} }})'
            base=f'DynamicStats {{ power: {expr(0)}, toughness: {expr(offset) if axes!="power is" else "None"} }}'
            continue
        conditional=source_rule_line(line,names)
        if m:=re.fullmatch(r'This creature gets ([+-][0-9]+)/([+-][0-9]+) as long as you gained life this turn\.',conditional):
            if 'Creature' not in types or max(abs(int(m[1])),abs(int(m[2])))>32767:body.append(original);continue
            scales.append(f'ScalingModifier {{ subject: Subject::Source, value: StatValue::LifeGainOccurredThisTurn, stats: Stats {{ power: {int(m[1])}, toughness: {int(m[2])} }} }}')
            continue
        prefix=re.fullmatch(r'As long as (.+), this creature gets (?:an additional )?([+-][0-9]+)/([+-][0-9]+)(?: and (?:has )?(.+))?\.',conditional)
        suffix=re.fullmatch(r'This creature gets (?:an additional )?([+-][0-9]+)/([+-][0-9]+)(?: and (?:has )?(.+?))? as long as (.+)\.',conditional)
        if prefix or suffix:
            condition,power,toughness,keywords=prefix.groups() if prefix else (suffix[4],suffix[1],suffix[2],suffix[3])
            value=stat_threshold_condition(condition)
            if ('Creature' in types and value is not None and max(abs(int(power)),abs(int(toughness)))<=32767
                    and (keywords is None or (source_grant_mask(keywords) is not None and not source_grant_mask(keywords) & KEYWORDS['flash']))):
                scales.append(f'ScalingModifier {{ subject: Subject::Source, value: {value}, stats: Stats {{ power: {int(power)}, toughness: {int(toughness)} }} }}')
                if keywords:
                    verb='' if keywords.startswith("can't ") else 'has '
                    body.append(f'This creature {verb}{keywords} as long as {condition}.')
                continue
        m=re.fullmatch(r'(This creature|Enchanted creature|Equipped creature) gets ([+-][0-9]+)/([+-][0-9]+) for each (.+?)(?: and has (.+))?\.',line)
        if m:
            subject,power,toughness,value,keywords=m.groups()
            if subject=='This creature' and 'Creature' not in types:body.append(original);continue
            if subject!='This creature' and not attached:body.append(original);continue
            if max(abs(int(power)),abs(int(toughness)))>32767:body.append(original);continue
            parsed=stat_value(value)
            if parsed is None:body.append(original);continue
            if keywords:
                if subject=='This creature' or keyword_list(keywords) is None:body.append(original);continue
                body.append(subject+' has '+keywords+'.')
            scales.append(f'ScalingModifier {{ subject: Subject::{"Source" if subject=="This creature" else "Attached"}, value: {parsed}, stats: Stats {{ power: {int(power)}, toughness: {int(toughness)} }} }}')
        else:body.append(original)
    if base is None and not scales:return card,None,[]
    proxy=dict(card,oracle_text='\n'.join(body).strip())
    if base is not None:
        proxy['power']='0'
        if '*' in card.get('toughness',''):proxy['toughness']='0'
    return proxy,base,scales


def static_permanent_template(card):
    if not isinstance(card.get('oracle_text'), str): return None
    body, modifiers, grants = [], [], []
    name=card.get('name','')
    names=[name] if name else []
    if name and card.get('type_line','').startswith('Legendary '): names.append(name.split(',')[0])
    def grant(subject,words,own_turn,requires_life_gain=False,condition=None):
        source_only=subject.casefold()=='this creature'
        if source_only and 'Creature' not in card.get('type_line','').split(' — ')[0].split(): return None
        filt=creature_filter('creatures you control' if source_only else subject)
        keywords=source_grant_mask(words) if source_only else keyword_list(words.casefold())
        # Positive grants commute only when recipient selection does not depend
        # on another layer-6 ability. Dependency/removal machinery stays gated.
        if filt is None or keywords is None or 'keyword: Keywords(0)' not in filt: return None
        if condition is not None and (not source_only or keywords & KEYWORDS['flash']):return None
        condition='None' if condition is None else f'Some({condition})'
        return f'KeywordGrant {{ condition: {condition}, requires_life_gain: {str(requires_life_gain).lower()}, filter: {filt}, source_only: {str(source_only).lower()}, own_turn: {str(own_turn).lower()}, keywords: Keywords({keywords}) }}'
    for original in re.sub(r'\([^()]*\)', '', card['oracle_text']).strip().split('\n'):
        line=original.strip()
        line=source_rule_line(line,names)
        requires_life_gain=line.endswith(' as long as you gained life this turn.')
        if requires_life_gain:line=line[:-len(' as long as you gained life this turn.')]+'.'
        own_turn=line.startswith('During your turn, ')
        if own_turn: line=line[len('During your turn, '):]
        for alias in names:
            if line.startswith(alias+' has '): line='this creature has '+line[len(alias+' has '):]
        prefix=re.fullmatch(r"As long as (.+), this creature (has .+|can't be blocked(?: by more than one creature)?)\.",line)
        suffix=re.fullmatch(r"[Tt]his creature (has .+?|can't be blocked(?: by more than one creature)?) as long as (.+)\.",line)
        if prefix or suffix:
            condition,words=prefix.groups() if prefix else (suffix[2],suffix[1])
            words=words.removeprefix('has ')
            value=stat_threshold_condition(condition)
            if value is None:return None
            extra=grant('this creature',words,own_turn,requires_life_gain,value)
            if extra is None:return None
            grants.append(extra);continue
        m = re.fullmatch(r'(.+) get ([+-][0-9]+)/([+-][0-9]+)(?: and have (.+))?\.', line)
        if m:
            if own_turn: return None
            filt = creature_filter(m[1])
            if filt is None or max(abs(int(m[2])), abs(int(m[3]))) > 32767: return None
            if m[4]:
                extra=grant(m[1],m[4],False,requires_life_gain)
                if extra is None: return None
                grants.append(extra)
            modifiers.append(f'StaticModifier {{ requires_life_gain: {str(requires_life_gain).lower()}, filter: {filt}, stats: Stats {{ power: {int(m[2])}, toughness: {int(m[3])} }} }}')
        elif m:=re.fullmatch(r'(.+) (?:have|has) (.+)\.',line):
            extra=grant(m[1],m[2],own_turn,requires_life_gain)
            if extra is None: return None
            grants.append(extra)
        else: body.append(original)
    if not modifiers and not grants: return None
    proxy = dict(card, oracle_text='\n'.join(body).strip())
    result = permanent_program_template(proxy)
    if result is None:
        template = creature_template(proxy)
        trigger = trigger_creature_template(proxy)
        if template: effect, keywords = template; result = effect, keywords, [], [], [0,0,0]
        elif trigger: effect, keywords, triggers = trigger; result = effect, keywords, triggers, [], [0,0,0]
    return (*result, modifiers, grants) if result else None



def keyword_list(text):
    words = [part.strip() for part in text.replace(', and ', ',').replace(' and ', ',').split(',')]
    if not words or not all(word in KEYWORDS for word in words): return None
    return sum(KEYWORDS[word] for word in set(words))


def keyword_effect(text, subject):
    """Entire fixed-duration modifier clause; no ability removal/qualified gains."""
    grow = None
    if m := re.fullmatch(r'gets? ([+-][0-9]+)/([+-][0-9]+) and gains? (.+) until end of turn\.', text):
        if max(abs(int(m[1])), abs(int(m[2]))) > 32767: return None
        grow, words = (int(m[1]), int(m[2])), m[3]
    elif m := re.fullmatch(r'gains? (.+) until end of turn\.', text): words = m[1]
    else: return None
    keywords = keyword_list(words)
    if keywords is None: return None
    if subject in ('Targets', 'Source', 'Attached'):
        return ([grow_instruction(subject, *grow)] if grow else []) + [f'GrantKeywords {{ subject: Subject::{subject}, keywords: Keywords({keywords}) }}']
    return ([f'GrowCreatures {{ filter: {subject}, power: {grow[0]}, toughness: {grow[1]} }}'] if grow else []) + [f'GrantCreatures {{ filter: {subject}, keywords: Keywords({keywords}) }}']


def normalize_duration(text):
    prefix = 'until end of turn, '
    if text.startswith(prefix) and text.count('.') == 1:
        return text[len(prefix):-1] + ' until end of turn.'
    return text


RULE_LABELS = ('Infusion — ', 'Delirium — ', 'Threshold — ', 'Descend 4 — ', 'Descend 8 — ', 'Ferocious — ', 'Alliance — ')

def strip_rule_label(text):
    for label in RULE_LABELS:
        if text.startswith(label):return text[len(label):]
    return text

def split_sentence(text):
    quoted=False
    for i,c in enumerate(text):
        if c=='"':quoted=not quoted
        if c=='.' and not quoted and text[i:i+2]=='. ':
            return text[:i+1],text[i+2:]
    return text,''

def guarded_clause(text):
    if not text.startswith('if '):return None
    for m in re.finditer(', ',text):
        if guard:=step_guard(text[3:m.start()]):return guard,text[m.end():]
    return None

def guarded_program(text,parse):
    clause=guarded_clause(text)
    if clause is None:return None
    guard,body=clause
    # Preserve the exact existing optional-payment construct as one nested block.
    atomic=optional_payment_program(body[0].lower()+body[1:],False)
    if atomic is not None and parse==statement_program:
        if any('AddMana(' in p for p in atomic):return None
        return [f'If {{ guard: {guard}, skip: {len(atomic)} }}',*atomic]
    first,tail=split_sentence(body)
    program=parse(first)
    if not program or any('AddMana(' in p for p in program):return None
    if tail.startswith('Otherwise, '):
        other,tail=split_sentence(tail[len('Otherwise, '):])
        alternative=parse(other)
        if not alternative or any('AddMana(' in p for p in alternative):return None
        program=[f'If {{ guard: {guard}, skip: {len(program)+1} }}',*program,f'Skip({len(alternative)})',*alternative]
    else:program=[f'If {{ guard: {guard}, skip: {len(program)} }}',*program]
    if tail:
        rest=parse(tail)
        if rest is None:return None
        program+=rest
    return program

def preserves_creature_focus(program):
    # Only these instructions prove that no competing object antecedent is
    # introduced and the original target stays in its zone. Scan both branches
    # conservatively: an ambiguous path must not bind a later pronoun either.
    return all(p.startswith(('If {', 'Skip(', 'Grow { subject: Subject::Targets,',
                             'GrantKeywords { subject: Subject::Targets,',
                             'PutCounters { subject: Subject::Targets,'))
               or p == 'Untap' for p in program)

def valid_creature_focus(program):
    for index, instruction in enumerate(program):
        if instruction.startswith(('Grow { subject: Subject::Targets,',
                                   'GrantKeywords { subject: Subject::Targets,')):
            if not preserves_creature_focus(program[:index]):return False
    return True

def focused_creature_program(text):
    program=unchecked_focused_creature_program(text)
    return program if program and valid_creature_focus(program) else None

def unchecked_focused_creature_program(text):
    text=strip_rule_label(text)
    if not text:return None
    text=text[0].lower()+text[1:]
    if result:=guarded_program(text,focused_creature_program):return result
    if m:=re.fullmatch(r'(?:it|that creature) (?:also )?(.+)',text):
        if grow:=re.fullmatch(r'gets ([+-][0-9]+)/([+-][0-9]+) until end of turn\.',m[1]):
            p,t=map(int,grow.groups())
            return [grow_instruction('Targets',p,t)] if max(abs(p),abs(t))<=32767 else None
        return keyword_effect(m[1],'Targets')
    return statement_program(text)


def search_program(text):
    quality = r'(basic land|land|creature|artifact|enchantment)'
    match = re.fullmatch(r'search your library for (a|up to two|up to three|up to four) ' + quality +
        r' cards?, (reveal (?:it|them), )?put (?:it|them) (into your hand|onto the battlefield)( tapped)?, then shuffle\.', text)
    if not match: return None
    count, kind, reveal, destination, tapped = match.groups()
    plural = count != 'a'
    if (' cards,' in text) != plural or ('put them ' in text) != plural: return None
    if reveal and (('reveal them,' in reveal) != plural): return None
    count = {'a': 1, 'up to two': 2, 'up to three': 3, 'up to four': 4}[count]
    if destination == 'into your hand' and (tapped or not reveal): return None
    filter_ = {'basic land': 'BasicLand', 'land': 'Land', 'creature': 'Creature', 'artifact': 'Artifact', 'enchantment': 'Enchantment'}[kind]
    destination = 'Hand' if destination == 'into your hand' else 'Battlefield'
    return [f'Search(SearchSpec {{ filter: SearchFilter::{filter_}, maximum: {count}, destination: SearchDestination::{destination}, tapped: {str(bool(tapped)).lower()}, reveal: {str(bool(reveal)).lower()} }})']


def sacrifice_spec(text):
    """Exact controlled-permanent filters; no targeting/protection predicates."""
    text = text.casefold()
    source_only = text.startswith('this ')
    if source_only:
        subject=text[5:]
        if subject not in ('creature','artifact','enchantment','permanent'): return None
        amount, other = 1, False
    else:
        m=re.fullmatch(r'(a|an|another|one|two|three|four|five|six|thirteen) (.+)',text)
        if not m: return None
        amount=dict(COUNT_WORDS,a=1,an=1,another=1,thirteen=13)[m[1]]
        other,subject=m[1]=='another',m[2]
    if subject.endswith(' you control'): subject=subject[:-12]
    plurals={word.casefold()+'s':word for word in ('creature','artifact','enchantment','land','permanent','planeswalker','token','food','clue','treasure','lander')}
    subject=' '.join(plurals.get(word,word) for word in subject.split())
    kinds={'creature':1,'artifact':16,'enchantment':32,'land':2,'permanent':0,
           'creature or planeswalker':65,'artifact or creature':17,'creature or artifact':17,
           'creature or enchantment':33,'enchantment or creature':33,
           'artifact or land':18,'land or artifact':18,
           'artifact or enchantment':48,'enchantment or artifact':48}
    # Only entire homogeneous qualifiers are supported; e.g. 'artifact or creature
    # token' cannot use a union mask with a shared token restriction.
    if subject.endswith('s') and subject[:-1] in kinds: subject=subject[:-1]
    if ' or ' in subject and (' with ' in subject or subject.startswith('nontoken ')):
        return None
    keyword=0
    if ' with ' in subject:
        subject,word=subject.split(' with ',1)
        if word not in KEYWORDS: return None
        keyword=KEYWORDS[word]
    nonland,tokens,subtype=False,'None','None'
    if subject=='nonland permanent': subject,nonland='permanent',True
    if subject.startswith('nontoken '): subject,tokens=subject[9:],'Some(false)'
    elif subject in ('creature token','artifact token','token'):
        subject,tokens={'creature token':'creature','artifact token':'artifact','token':'permanent'}[subject],'Some(true)'
    if subject in ('food','clue','treasure','lander'):
        subtype,subject='Some('+json.dumps(subject.capitalize())+')','artifact'
    if subject not in kinds: return None
    filt=(f'SacrificeFilter {{ keywords: Keywords({keyword}), source_only: {str(source_only).lower()}, kinds: {kinds[subject]}, nonland: {str(nonland).lower()}, '
          f'other: {str(other).lower()}, tokens: {tokens}, subtype: {subtype} }}')
    return amount,filt


def extract_additional_cost(card):
    """One mandatory object cost; the entire residual text must still compile."""
    text = card.get('oracle_text', '')
    lines = text.split('\n')
    costs = [line for line in lines if line.startswith('As an additional cost to cast this spell, ')]
    if len(costs) != 1: return card, None
    line = costs[0]
    if line == 'As an additional cost to cast this spell, discard a card.':
        kind, spec = 'Discard', sacrifice_spec('a permanent')
    else:
        m = re.fullmatch(r'As an additional cost to cast this spell, sacrifice (.+)\.', line)
        if not m: return card, None
        kind, spec = 'Sacrifice', sacrifice_spec(m[1])
    if spec is None or spec[0] != 1 or 'source_only: true' in spec[1]: return card, None
    return dict(card, oracle_text='\n'.join(item for item in lines if item != line)), f'AdditionalCost {{ kind: EffectChoiceKind::{kind}, filter: {spec[1]} }}'


def object_choice(kind,players,spec,skip=None):
    amount,filter_=spec
    skip='None' if skip is None else f'Some({skip})'
    return f'ChooseObjects {{ kind: EffectChoiceKind::{kind}, players: PlayerSet::{players}, filter: {filter_}, amount: {amount}, optional_skip: {skip} }}'


def sacrifice_program(text):
    m=re.fullmatch(r'(you|each player|each opponent|target player|target opponent) sacrifices? (.+?)(?: of (?:their|your) choice)?\.',text)
    if not m or (spec:=sacrifice_spec(m[2])) is None: return None
    targets={'target player':'Player','target opponent':'Opponent'}.get(m[1],'None')
    players={'you':'Controller','each player':'Each','each opponent':'Opponent'}.get(m[1],'Targeted')
    return targets,[object_choice('Sacrifice',players,spec)]


def optional_payment_program(text,targeted):
    # A single exact conditional sentence. Extra sentences may have different
    # conditional scope; never swallow them into the conditional block by guess.
    m=re.fullmatch(r'you may (sacrifice [^.]+|discard (?:a card|two cards|three cards))\. If you do, ([^.]+)\.',text)
    if not m: return None
    cost,body=m[1],m[2]+'.'
    if cost.startswith('sacrifice '):
        kind,spec='Sacrifice',sacrifice_spec(cost[len('sacrifice '):])
        if spec is None: return None
    else:
        kind='Discard'
        spec=(DRAW_COUNTS[cost[len('discard '):]],sacrifice_spec('a permanent')[1])
    if targeted:
        result=targeted_program(body)
        if result is None: return None
        targets,program=result
        return targets,[object_choice(kind,'Controller',spec,len(program)),*program]
    program=statement_program(body)
    if program is None: return None
    return [object_choice(kind,'Controller',spec,len(program)),*program]



def mill_quantity(text):
    words={'a card':1,'one card':1,'two cards':2,'three cards':3,'four cards':4,
           'five cards':5,'six cards':6,'seven cards':7,'eight cards':8,'nine cards':9,'ten cards':10}
    if text in words: return words[text]
    if m:=re.fullmatch(r'([0-9]+) cards?',text):
        return int(m[1]) if int(m[1])<=65535 else None
    return None


def mill_instruction(players, amount):
    return f'Mill(PlayerSet::{players}, MillAmount::Fixed({amount}))'


def mill_program(text):
    # Bind recovery only within an explicit mill-and-recover expression. A bare
    # 'them' or 'milled cards' elsewhere does not acquire a guessed antecedent.
    if m:=re.fullmatch(r'(?:you )?mill (.+?)(?:, then you may return a (permanent|land) card from among them to your hand|\. (?:Then )?You may put a (permanent|land) card from among (?:them|the milled cards|the cards milled this way) into your hand)\.(?: (.+))?',text):
        amount=mill_quantity(m[1])
        if amount is None: return None
        tail=statement_program(m[4]) if m[4] else []
        if tail is None: return None
        filter_=(m[2] or m[3]).capitalize()
        return [mill_instruction('Controller',amount),f'RecoverMilled(RecoveryFilter::{filter_})',*tail]
    if m:=re.fullmatch(r'(?:you )?mill (.+?)(?:, then (.+)| and (you gain [0-9]+ life))\.',text):
        amount=mill_quantity(m[1]);tail=statement_program((m[2] or m[3])+'.')
        if amount is not None and tail: return [mill_instruction('Controller',amount),*tail]
    if m:=re.fullmatch(r'(?:you )?mill (.+)\.',text):
        amount=mill_quantity(m[1])
        if amount is not None: return [mill_instruction('Controller',amount)]
    if m:=re.fullmatch(r'you may mill (.+)\.',text):
        amount=mill_quantity(m[1])
        if amount is not None: return [f'MayMill({amount})',mill_instruction('Controller',amount)]
    if m:=re.fullmatch(r'(each player|each opponent) mills (.+)\.',text):
        amount=mill_quantity(m[2])
        if amount is not None: return [mill_instruction('Each' if m[1]=='each player' else 'Opponent',amount)]
    return None


def counter_group_program(text):
    """Non-targeted simultaneous recipients, using only existing creature facts."""
    m=re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on each (.+)\.',text)
    if not m: return None
    if (filt:=creature_filter(m[3])) is None: return None
    return [f'PutCreatureCounters {{ filter: {filt}, kind: CounterKind::{COUNTER_KINDS[m[2]]}, amount: {COUNT_WORDS[m[1]]} }}']


def battlefield_filter(kinds=0, relation=-1, nonland=False, tapped=None, without=0, subtype=None, max_mana=4294967295):
    return (f'BattlefieldFilter {{ kinds: {kinds}, relation: {relation}, nonland: {str(nonland).lower()}, '
            f'tapped: {"None" if tapped is None else "Some("+str(tapped).lower()+")"}, '
            f'without: Keywords({without}), subtype: {"None" if subtype is None else "Some("+json.dumps(subtype)+")"}, max_mana: {max_mana} }}')


def battlefield_filters(subject):
    return {
        'creatures': [battlefield_filter(kinds=1)],
        'tapped creatures': [battlefield_filter(kinds=1,tapped=True)],
        'untapped creatures': [battlefield_filter(kinds=1,tapped=False)],
        'creatures and enchantments': [battlefield_filter(kinds=33)],
        'creatures and Spacecraft': [battlefield_filter(kinds=1),battlefield_filter(subtype='Spacecraft')],
        'nonland permanents with mana value 1 or less': [battlefield_filter(nonland=True,max_mana=1)],
        'artifacts with mana value 3 or less': [battlefield_filter(kinds=16,max_mana=3)],
    }.get(subject)


def battlefield_move_program(text):
    result=None
    if text=='destroy all creatures. You gain 1 life for each creature destroyed this way.':
        text='destroy all creatures.';result='GainLifeMoved'
    elif text=='destroy all creatures and enchantments. Draw a card for each permanent destroyed this way.':
        text='destroy all creatures and enchantments.';result='DrawMoved'
    if m:=re.fullmatch(r'(destroy|exile) all (.+)\.',text):
        if filters:=battlefield_filters(m[2]):
            program=['MoveBattlefield { filters: &['+', '.join(filters)+'], exile: '+str(m[1]=='exile').lower()+' }']
            return program+([result] if result else [])
    return None


def battlefield_damage_program(text):
    m=re.fullmatch(r'this (?:creature|artifact|enchantment|permanent|spell) deals ([0-9]+) damage to (.+)\.',text)
    if not m or not 0<=int(m[1])<=32767: return None
    amount=int(m[1]);subject=m[2]
    if subject=='each player': return [f'DamagePlayers(PlayerSet::Each, {amount})']
    filters={
        'each creature': [battlefield_filter(kinds=1)],
        'each creature without flying': [battlefield_filter(kinds=1,without=1)],
        'each creature your opponents control': [battlefield_filter(kinds=1,relation=1)],
        'each creature and planeswalker': [battlefield_filter(kinds=65)],
        'each creature and each planeswalker': [battlefield_filter(kinds=65)],
        'each creature and planeswalker your opponents control': [battlefield_filter(kinds=65,relation=1)],
        'each opponent and each creature they control': [battlefield_filter(kinds=1,relation=1)],
    }.get(subject)
    if filters is None:return None
    players='Some(PlayerSet::Opponent)' if subject=='each opponent and each creature they control' else 'None'
    return ['DamageBattlefield { filters: &['+', '.join(filters)+f'], players: {players}, amount: {amount} }}']


def statement_program(text):
    """Compile only these complete sentences; residual text never gets ignored."""
    if not isinstance(text, str) or not text:
        return None
    text = strip_rule_label(text)
    if not text:return None
    text = normalize_duration(text[0].lower() + text[1:])
    if text=='draw two cards. Then discard a card unless you attacked this turn.':
        # On raid, execute the forward skip; otherwise bypass it and discard.
        return ['Draw(PlayerSet::Controller, 2)', 'If { guard: StepGuard::Attacked, skip: 1 }',
                'Skip(1)', 'Discard(PlayerSet::Controller, 1)']
    if m:=re.fullmatch(r'([^.]+) if you gained life this turn\.',text):
        program=statement_program(m[1]+'.')
        return [f'If {{ guard: StepGuard::GainedLife, skip: {len(program)} }}',*program] if program and not any('AddMana(' in p for p in program) else None
    first,tail=split_sentence(text)
    tail=strip_rule_label(tail)
    if tail and (clause:=guarded_clause(tail[0].lower()+tail[1:])):
        guard,alternative=clause
        # Only a single replacement sentence establishes this exact scope.
        # Earlier/later sentences may have independent conditions or antecedents.
        if alternative.endswith(' instead.') and not split_sentence(alternative)[1]:
            normal=statement_program(first);other=statement_program(alternative.removesuffix(' instead.')+'.')
            if normal and other and not any('AddMana(' in p for p in normal+other):
                return [f'If {{ guard: {guard}, skip: {len(other)+1} }}',*other,f'Skip({len(normal)})',*normal]
    if program:=guarded_program(text,statement_program):return program
    if m:=re.fullmatch(r'(?:you )?draw (?:cards equal to (.+)|X cards, where X is (.+))\.',text):
        value=amount_value(m[1] or m[2])
        return [f'DrawScaled(PlayerSet::Controller, {value})'] if value else None
    if m:=re.fullmatch(r'each opponent loses X life and you gain X life, where X is (.+)\.',text):
        value=amount_value(m[1])
        return [f'DrainLifeScaled({value})'] if value else None
    if text=="draw a card for each card you've discarded this turn.":return ['DrawScaled(PlayerSet::Controller, AmountValue::CardsDiscardedThisTurn)']
    for phrase,mask in POWER_BLOCKING_TEXT_FLAGS.items():
        if text==phrase[:-1]+' this turn.':return [f'GrantKeywords {{ subject: Subject::Source, keywords: Keywords({mask}) }}']
    if text=='discard a card.':return ['Discard(PlayerSet::Controller, 1)']
    if text=='discard your hand.':return ['DiscardHand(PlayerSet::Controller)']
    if program:=scaled_life_program(text):return program
    if text == 'you get an emblem with "Whenever you gain life, target opponent loses that much life."':
        body='Whenever you gain life, target opponent loses that much life.'
        key=hashlib.sha256(body.encode()).hexdigest()[:24]
        EMBLEM_DEFS[key]=body
        return [f'CreateEmblem(__EMBLEM_{key}__)']
    if result:=battlefield_move_program(text):return result
    if result:=battlefield_damage_program(text):return result
    if m:=re.fullmatch(r'this creature connives(?: ([0-9]+))?\.',text):
        amount=int(m[1]) if m[1] else 1
        return [f'Connive {{ subject: Subject::Source, amount: {amount} }}'] if amount<=65535 else None
    if counters := counter_group_program(text): return counters
    if mill := mill_program(text): return mill
    if choice := optional_payment_program(text, False): return choice
    if sacrifice := sacrifice_program(text):
        if sacrifice[0] == 'None': return sacrifice[1]
    token = scaled_token_program(text) or token_program(text) or predefined_program(text)
    if token: return token
    if m:=re.fullmatch(r'(?:you )?draw (a card|two cards|three cards|four cards) and (create .+)\.',text):
        if program:=token_program(m[2]+'.') or predefined_program(m[2]+'.'):
            return [f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})',*program]
    if text in ('tap enchanted creature.','tap equipped creature.'): return ['TapAttached']
    if m := re.fullmatch(r'(?:enchanted|equipped) creature gets ([+-][0-9]+)/([+-][0-9]+) until end of turn\.',text):
        if max(abs(int(m[1])),abs(int(m[2])))>32767: return None
        return [grow_instruction('Attached',int(m[1]),int(m[2]))]
    if m := re.fullmatch(r'(?:enchanted|equipped) creature (gains? .+ until end of turn\.)',text):
        return keyword_effect(m[1],'Attached')
    if m:=re.fullmatch(r'sacrifice (this creature|this artifact|this enchantment|this permanent)\.',text):
        return [object_choice('Sacrifice','Controller',sacrifice_spec(m[1]))]
    if text == 'each player discards their hand, then draws seven cards.':
        return ['DiscardHand(PlayerSet::Each)','Draw(PlayerSet::Each, 7)']
    if m:=re.fullmatch(r'tap (.+)\.',text):
        if filt:=creature_filter(m[1]): return [f'TapCreatures({filt})']
    if m:=re.fullmatch(r'this creature gets ([+-][0-9]+)/([+-][0-9]+) until end of turn and can\'t be blocked this turn\.',text):
        if max(abs(int(m[1])),abs(int(m[2])))>32767: return None
        return [grow_instruction('Source',int(m[1]),int(m[2])),'GrantKeywords { subject: Subject::Source, keywords: Keywords(16384) }']
    if m:=re.fullmatch(r'this creature (gains .+ until end of turn)\. Tap it\.',text):
        if program:=keyword_effect(m[1]+'.','Source'): return [*program,'TapSource']
    if text == 'put a +1/+1 counter on this creature and add {R}.':
        return [counter_instruction('Source','a','+1/+1'), *mana_program('add {R}.')]
    if text == 'untap this creature.': return ['UntapSource']
    if text == 'tap this creature.': return ['TapSource']
    if m := re.fullmatch(r'untap (.+)\.', text):
        if filt := creature_filter(m[1]): return [f'UntapCreatures({filt})']
    if m := re.fullmatch(r'(?:you )?draw (a card|two cards|three cards|four cards) and (?:you )?lose ([0-9]+) life\.', text):
        if int(m[2])>65535: return None
        return [f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})',f'LoseLife(PlayerSet::Controller, {int(m[2])})']
    if m := re.fullmatch(r'you lose ([0-9]+) life and (create .+)\.', text):
        if int(m[1])>65535: return None
        if program := token_program(m[2]+'.') or predefined_program(m[2]+'.'): return [f'LoseLife(PlayerSet::Controller, {int(m[1])})', *program]
    if m := re.fullmatch(r'this (?:creature|artifact|enchantment|permanent|spell) deals ([0-9]+) damage to each opponent(?: and you gain ([0-9]+) life)?\.', text):
        if int(m[1]) > 32767 or (m[2] and int(m[2]) > 65535): return None
        return [f'DamagePlayers(PlayerSet::Opponent, {int(m[1])})'] + ([f'GainLife(PlayerSet::Controller, {int(m[2])})'] if m[2] else [])
    if text == "exile each opponent's graveyard.": return ['ExileGraveyard(PlayerSet::Opponent)']
    if text == "exile your graveyard.": return ['ExileGraveyard(PlayerSet::Controller)']
    if text == 'investigate.': return predefined_program('create a Clue token.')
    if mana := mana_program(text): return mana
    if text.startswith('you may search your library '):
        program=search_program(text[len('you may '):])
        return [f'May {{ skip: {len(program)} }}',*program] if program else None
    if search := search_program(text): return search
    if not text.endswith('.'):
        return None
    if m := re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on this creature\.', text):
        return [counter_instruction('Source', m[1], m[2])]
    if m := re.fullmatch(r'(scry|surveil) ([0-9]+)\.', text):
        return [f'{m[1].capitalize()}({int(m[2])})'] if int(m[2]) <= 65535 else None
    if m := re.fullmatch(r'this creature gets ([+-][0-9]+)/([+-][0-9]+) until end of turn\.', text):
        if max(abs(int(m[1])), abs(int(m[2]))) > 32767: return None
        return [grow_instruction('Source', int(m[1]), int(m[2]))]
    if m := re.fullmatch(r'(.+) gets? ([+-][0-9]+)/([+-][0-9]+) until end of turn\.', text):
        filt = creature_filter(m[1])
        if filt and max(abs(int(m[2])), abs(int(m[3]))) <= 32767:
            return [f'GrowCreatures {{ filter: {filt}, power: {int(m[2])}, toughness: {int(m[3])} }}']
    if text.startswith('this creature '):
        if program := keyword_effect(text[len('this creature '):], 'Source'): return program
    if m := re.fullmatch(r'(.+?) (gets? .+|gains? .+) until end of turn\.', text):
        filt = creature_filter(m[1])
        if filt and (program := keyword_effect(m[2] + ' until end of turn.', filt)): return program
    draw = r'(a card|two cards|three cards|four cards)'
    if m := re.fullmatch(r'(scry|surveil) ([0-9]+), then draw ' + draw + r'\.', text):
        if int(m[2]) > 65535: return None
        return [f'{m[1].capitalize()}({int(m[2])})', f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[3]]})']
    if m := re.fullmatch(r'draw (a card|two cards|three cards|four cards) and put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on this creature\.', text):
        return [f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})',counter_instruction('Source',m[2],m[3])]
    if m := re.fullmatch(r'draw ' + draw + r'\.', text):
        return [f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})']
    if m := re.fullmatch(r'you gain ([0-9]+) life\.', text):
        return [f'GainLife(PlayerSet::Controller, {int(m[1])})'] if int(m[1]) <= 65535 else None
    if m := re.fullmatch(r'each opponent loses ([0-9]+) life\.', text):
        return [f'LoseLife(PlayerSet::Opponent, {int(m[1])})'] if int(m[1]) <= 65535 else None
    if m := re.fullmatch(r'this creature deals ([0-9]+) damage to you(?: and you draw (a card|two cards|three cards|four cards))?\.', text):
        return [f'DamagePlayers(PlayerSet::Controller, {int(m[1])})']+([f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[2]]})'] if m[2] else []) if int(m[1])<=32767 else None
    if m:=re.fullmatch(r'each player loses ([0-9]+) life\.',text):
        return [f'LoseLife(PlayerSet::Each, {int(m[1])})'] if int(m[1])<=65535 else None
    if m:=re.fullmatch(r'discard '+draw+r', then draw '+draw+r'\.',text):
        return [f'Discard(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})',f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[2]]})']
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
            if sentence.startswith('Then '): sentence=sentence[5:]
            result = statement_program(sentence if sentence.endswith('.') else sentence + '.')
            if result is None:
                return None
            instructions.extend(result)
        return instructions
    return None



TARGET_RULES = {
    'None': 'TargetRule::None', 'Creature': 'TargetRule::Creature',
    'Any': 'TargetRule::Any', 'Spell': 'TargetRule::Spell',
    'CreatureVehicle': 'TargetRule::CreatureOrVehicle { relation: -1 }',
    'CreatureVehicleOpponent': 'TargetRule::CreatureOrVehicle { relation: 1 }',
    'Player': 'TargetRule::Player { opponent_only: false }',
    'Opponent': 'TargetRule::Player { opponent_only: true }',
}
for _name,_kinds,_excluded in [('SpellCreature',1,0),('SpellNoncreature',0,1),('SpellInstantSorcery',12,0),('SpellArtifact',16,0),('SpellEnchantment',32,0),('SpellArtifactEnchantment',48,0),('SpellArtifactCreature',17,0)]:
    TARGET_RULES[_name]=f'TargetRule::SpellKinds {{ kinds: {_kinds}, excluded: {_excluded} }}'
for _name,_colors,_minimum,_maximum in [('SpellRedGreen',24,0,'u64::MAX'),('SpellWhiteBlack',5,0,'u64::MAX'),('SpellMana2',0,2,2),('SpellManaMin4',0,4,'u64::MAX')]:
    TARGET_RULES[_name]=f'TargetRule::SpellCharacteristics {{ colors: {_colors}, min_mana: {_minimum}, max_mana: {_maximum} }}'


for _name, _zone, _kinds, _relation, _other, _nonland in [
    ('OtherCreature', 'Battlefield', 1, -1, True, False),
    ('CreatureYou', 'Battlefield', 1, 0, False, False),
    ('OtherCreatureYou', 'Battlefield', 1, 0, True, False),
    ('CreatureOpponent', 'Battlefield', 1, 1, False, False),
    ('Permanent', 'Battlefield', 0, -1, False, False),
    ('Nonland', 'Battlefield', 0, -1, False, True),
    ('OtherNonlandYou', 'Battlefield', 0, 0, True, True),
    ('OtherPermanentYou', 'Battlefield', 0, 0, True, False),
    ('NonlandOpponent', 'Battlefield', 0, 1, False, True),
    ('CreatureEnchantment', 'Battlefield', 33, -1, False, False),
    ('CardGrave', 'Graveyard', 0, -1, False, False),
    ('Land', 'Battlefield', 2, -1, False, False),
    ('Artifact', 'Battlefield', 16, -1, False, False),
    ('ArtifactCreature', 'Battlefield', 17, -1, False, False),
    ('ArtifactCreatureYou', 'Battlefield', 17, 0, False, False),
    ('Enchantment', 'Battlefield', 32, -1, False, False),
    ('ArtifactEnchantment', 'Battlefield', 48, -1, False, False),
    ('CreatureGraveYou', 'Graveyard', 1, 0, False, False),
    ('OtherCreatureGraveYou', 'Graveyard', 1, 0, True, False),
    ('InstantSorceryGraveYou', 'Graveyard', 12, 0, False, False),
    ('CreatureGrave', 'Graveyard', 1, -1, False, False),
]:
    TARGET_RULES[_name] = (f'TargetRule::Object {{ zone: TargetZone::{_zone}, kinds: {_kinds}, '
                          f'relation: {_relation}, other: {str(_other).lower()}, nonland: {str(_nonland).lower()}, tapped: false, min_mana: 0, max_mana: u32::MAX }}')


for _name, _kinds, _relation, _nonland, _tapped, _minimum, _maximum in [
    ('CreatureOpponentTapped', 1, 1, False, True, 0, 4294967295),
    ('CreatureOpponentMax3', 1, 1, False, False, 0, 3),
    ('NonlandOpponentMax2', 0, 1, True, False, 0, 2),
    ('CreatureMin3', 1, -1, False, False, 3, 4294967295),
]:
    TARGET_RULES[_name] = (f'TargetRule::Object {{ zone: TargetZone::Battlefield, kinds: {_kinds}, '
                          f'relation: {_relation}, other: false, nonland: {str(_nonland).lower()}, '
                          f'tapped: {str(_tapped).lower()}, min_mana: {_minimum}, max_mana: {_maximum} }}')

EXILE_SUBJECTS = {
    'target creature': 'Creature', 'target artifact': 'Artifact',
    'target enchantment': 'Enchantment', 'target artifact or enchantment': 'ArtifactEnchantment',
    'target creature or enchantment': 'CreatureEnchantment',
    'target nonland permanent': 'Nonland',
    'target creature an opponent controls': 'CreatureOpponent',
    'target nonland permanent an opponent controls': 'NonlandOpponent',
    'target tapped creature an opponent controls': 'CreatureOpponentTapped',
    'target creature an opponent controls with mana value 3 or less': 'CreatureOpponentMax3',
    'target nonland permanent an opponent controls with mana value 2 or less': 'NonlandOpponentMax2',
    'target creature with mana value 3 or greater': 'CreatureMin3',
    'target card from a graveyard': 'CardGrave',
}

# The same immutable rule is used at announcement and resolution. Unions retain
# branch-local qualifiers: an artifact need not also be a flying creature.
def filtered_target(base, predicate):
    return f'TargetRule::Filtered {{ base: &{TARGET_RULES[base]}, predicate: &TargetPredicate::{predicate} }}'


for _name, _kinds, _relation, _tapped, _minimum, _maximum in [
    ('CreatureTapped', 1, -1, True, 0, 4294967295),
    ('CreatureMax3', 1, -1, False, 0, 3),
    ('PermanentMin3', 0, -1, False, 3, 4294967295),
    ('CreaturePlaneswalker', 65, -1, False, 0, 4294967295),
    ('Planeswalker', 64, -1, False, 0, 4294967295),
    ('CreaturePlaneswalkerMax3', 65, -1, False, 0, 3),
    ('CreaturePlaneswalkerMin3', 65, -1, False, 3, 4294967295),
    ('CardGraveOpponent', 0, 1, False, 0, 4294967295),
    ('PermanentOpponent', 0, 1, False, 0, 4294967295),
]:
    _zone = 'Graveyard' if _name == 'CardGraveOpponent' else 'Battlefield'
    TARGET_RULES[_name] = (f'TargetRule::Object {{ zone: TargetZone::{_zone}, kinds: {_kinds}, '
                          f'relation: {_relation}, other: false, nonland: false, '
                          f'tapped: {str(_tapped).lower()}, min_mana: {_minimum}, max_mana: {_maximum} }}')
TARGET_RULES['VampireYou'] = ('TargetRule::Filtered { base: &TargetRule::Object { zone: TargetZone::Battlefield, kinds: 0, relation: 0, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: 4294967295 }, predicate: &TargetPredicate::Subtype("Vampire") }')

for _name, _base, _predicate in [
    ('CreatureAttacking', 'Creature', 'Attacking'),
    ('CreatureAttackingBlocking', 'Creature', 'AttackingOrBlocking'),
    ('CreatureDamagedOpponent', 'CreatureOpponent', 'Damaged'),
    ('CreatureDealtDamageOpponent', 'CreatureOpponent', 'DealtDamage'),
    ('CreaturePowerMin3', 'Creature', 'PowerAtLeast(3)'),
    ('CreaturePowerMin4', 'Creature', 'PowerAtLeast(4)'),
    ('CreatureToughnessMin4', 'Creature', 'ToughnessAtLeast(4)'),
    ('CreatureEitherMin4', 'Creature', 'PowerOrToughnessAtLeast(4)'),
    ('CreatureFlying', 'Creature', 'Keyword(Keywords(1))'),
    ('CreatureGreenWhite', 'Creature', 'Colors(17)'),
    ('CreaturePlaneswalkerBlackRed', 'CreaturePlaneswalker', 'Colors(12)'),
    ('CreaturePlaneswalkerGreenBlue', 'CreaturePlaneswalker', 'Colors(18)'),
    ('CreaturePlaneswalkerBlueRed', 'CreaturePlaneswalker', 'Colors(10)'),
    ('TokenOpponent', 'PermanentOpponent', 'Token'),
    ('Spacecraft', 'Permanent', 'Subtype("Spacecraft")'),
    ('NonbasicLand', 'Land', 'Nonbasic'),
    ('CreatureNonOutlaw', 'Creature', 'NonOutlaw'),
]:
    TARGET_RULES[_name] = filtered_target(_base, _predicate)
for _name, _members in [
    ('ArtifactEnchantmentCreatureFlying', ['ArtifactEnchantment', 'CreatureFlying']),
    ('ArtifactEnchantmentCreaturePowerMin4', ['ArtifactEnchantment', 'CreaturePowerMin4']),
    ('ArtifactCreatureTapped', ['Artifact', 'CreatureTapped']),
    ('CreatureSpacecraft', ['Creature', 'Spacecraft']),
    ('PlayerPlaneswalker', ['Player', 'Planeswalker']),
    ('CreatureVehicleNonbasicLand', ['CreatureVehicle', 'NonbasicLand']),
]:
    TARGET_RULES[_name] = 'TargetRule::Union(&[' + ', '.join(TARGET_RULES[member] for member in _members) + '])'

FIXED_DAMAGE_SUBJECTS = {
    'any target':'Any',
    'target creature':'Creature',
    'target creature or planeswalker':'CreaturePlaneswalker',
    'target opponent':'Opponent',
    'target player':'Player',
    'target player or planeswalker':'PlayerPlaneswalker',
    'target tapped creature':'CreatureTapped',
    'target attacking or blocking creature':'CreatureAttackingBlocking',
}

REMOVAL_SUBJECTS = dict(EXILE_SUBJECTS, **{
    'target permanent': 'Permanent',
    'target creature or Vehicle': 'CreatureVehicle',
    'target creature or Spacecraft': 'CreatureSpacecraft',
    'target artifact or creature': 'ArtifactCreature',
    'target creature or planeswalker': 'CreaturePlaneswalker',
    'target attacking creature': 'CreatureAttacking',
    'target attacking or blocking creature': 'CreatureAttackingBlocking',
    'target tapped creature': 'CreatureTapped',
    'target creature with mana value 3 or less': 'CreatureMax3',
    'target permanent with mana value 3 or greater': 'PermanentMin3',
    'target creature or planeswalker with mana value 3 or less': 'CreaturePlaneswalkerMax3',
    'target creature or planeswalker with mana value 3 or greater': 'CreaturePlaneswalkerMin3',
    'target creature with power 3 or greater': 'CreaturePowerMin3',
    'target creature with power 4 or greater': 'CreaturePowerMin4',
    'target creature with toughness 4 or greater': 'CreatureToughnessMin4',
    'target creature with power or toughness 4 or greater': 'CreatureEitherMin4',
    'target creature with flying': 'CreatureFlying',
    'target artifact, enchantment, or creature with flying': 'ArtifactEnchantmentCreatureFlying',
    'target artifact, enchantment, or creature with power 4 or greater': 'ArtifactEnchantmentCreaturePowerMin4',
    'target artifact or tapped creature': 'ArtifactCreatureTapped',
    'target creature, Vehicle, or nonbasic land': 'CreatureVehicleNonbasicLand',
    'target non-outlaw creature': 'CreatureNonOutlaw',
    'target green or white creature': 'CreatureGreenWhite',
    "target creature or planeswalker that's black or red": 'CreaturePlaneswalkerBlackRed',
    "target creature or planeswalker that's green or blue": 'CreaturePlaneswalkerGreenBlue',
    "target creature or planeswalker that's blue or red": 'CreaturePlaneswalkerBlueRed',
    'target token an opponent controls': 'TokenOpponent',
    "target card from an opponent's graveyard": 'CardGraveOpponent',
    'target creature an opponent controls that was dealt damage this turn': 'CreatureDamagedOpponent',
    'target creature an opponent controls that dealt damage this turn': 'CreatureDealtDamageOpponent',
})


def rust_target_group(spec):
    if spec == 'None':
        return 'TargetSpec::NONE'
    if isinstance(spec, str):
        return f'TargetSpec::one({TARGET_RULES[spec]})'
    rule, minimum, maximum = spec
    return f'TargetSpec {{ rule: {TARGET_RULES[rule]}, min: {minimum}, max: {maximum} }}'


def rust_targets(spec):
    if isinstance(spec, list):
        return 'TargetPlan::Groups(&[' + ', '.join(rust_target_group(group) for group in spec) + '])'
    return 'TargetPlan::Single(' + rust_target_group(spec) + ')'


def power_damage_program(text):
    opponent = r"target creature (?:an opponent controls|you don't control)"
    if m := re.fullmatch(r'target creature you control (fights|deals damage equal to its power to) ' + opponent + r'\.', text):
        action = 'Fight' if m[1] == 'fights' else 'PowerDamage'
        return ['CreatureYou', 'CreatureOpponent'], [f'{action} {{ source: 0, recipient: 1 }}']
    # A buff refers to the first occurrence; its pronoun introduces no new target.
    if m := re.fullmatch(r'target creature you control gets ([+-][0-9]+)/([+-][0-9]+) until end of turn\. (?:Then it|It) (fights|deals damage equal to its power to) ' + opponent + r'\.', text):
        if max(abs(int(m[1])), abs(int(m[2]))) > 32767: return None
        action = 'Fight' if m[3] == 'fights' else 'PowerDamage'
        return ['CreatureYou', 'CreatureOpponent'], [grow_instruction('Targets', int(m[1]), int(m[2])), f'{action} {{ source: 0, recipient: 1 }}']
    if m := re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on target creature you control(?:\. (?:Then it|It)|, then it) (fights|deals damage equal to its power to) ' + opponent + r'\.', text):
        action = 'Fight' if m[3] == 'fights' else 'PowerDamage'
        return ['CreatureYou', 'CreatureOpponent'], [counter_instruction('Targets', m[1], m[2]), f'{action} {{ source: 0, recipient: 1 }}']
    return None



COUNTER_TARGETS = {
    'target spell':'Spell','target creature spell':'SpellCreature',
    'target noncreature spell':'SpellNoncreature','target instant or sorcery spell':'SpellInstantSorcery',
    'target artifact spell':'SpellArtifact','target enchantment spell':'SpellEnchantment',
    'target artifact or enchantment spell':'SpellArtifactEnchantment',
    'target artifact or creature spell':'SpellArtifactCreature',
    'target red or green spell':'SpellRedGreen','target white or black spell':'SpellWhiteBlack',
    'target spell with mana value 2':'SpellMana2','target spell with mana value 4 or greater':'SpellManaMin4',
}


def mana_cost_spec(text):
    mana=fixed_cost(text)
    if mana is None or max([mana[1],*mana[0]])>4294967295: return None
    return f'ManaCost {{ colored: {mana[0]}, generic: {mana[1]} }}'


def counter_unless(cost,subject='Targets',exile=False,skip=0):
    return (f'CounterUnlessPay {{ cost: {cost}, subject: CounterSubject::{subject}, '
            f'exile: {str(exile).lower()}, unpaid_skip: {skip} }}')


def counter_program(text):
    for phrase,target in COUNTER_TARGETS.items():
        prefix='counter '+phrase
        if text==prefix+'.': return target,['Counter']
        if not text.startswith(prefix+' unless its controller pays '): continue
        rest=text[len(prefix+' unless its controller pays '):]
        match=re.fullmatch(r'((?:\{(?:[0-9]+|[WUBRGC])\})+)\.(.*)',rest)
        if match is None or (cost:=mana_cost_spec(match[1])) is None: return None
        tail=match[2]
        if not tail: return target,[counter_unless(cost)]
        if tail.startswith(' If they do, '):
            body=tail[len(' If they do, '):]
            # Later sentences have independent scope; admit only one paid sentence.
            if body.count('.') != 1: return None
            if body.startswith('you create '): body=body[4:]
            program=statement_program(body)
            return (target,[counter_unless(cost,skip=len(program)),*program]) if program else None
        if tail==" If that spell is countered this way, exile it instead of putting it into its owner's graveyard.":
            return target,[counter_unless(cost,exile=True)]
        return None
    return None


def ward_word(word):
    if not word.startswith('ward '): return None
    return mana_cost_spec(word[len('ward '):].upper())


def nonmana_ward_program(line):
    # Whole standalone clauses only; mixed costs and residual text stay gated.
    text=line.casefold()
    if text=='ward—get five poison counters.':
        return ['MayOpponent { skip: 2 }','AddPoison(PlayerSet::Opponent, 5)','Skip(1)','CounterRelated']
    if text=='ward—discard a card.':
        cost=object_choice('Discard','Opponent',sacrifice_spec('a permanent'),skip=1)
    elif text=='ward—sacrifice a creature.':
        cost=object_choice('Sacrifice','Opponent',sacrifice_spec('a creature'),skip=1)
    elif m:=re.fullmatch(r'ward—pay ([0-9]+) life\.',text):
        amount=int(m[1])
        if amount > 65535:return None
        return [f'MayPayOpponentLife {{ amount: {amount}, skip: 2 }}',
                f'LoseLife(PlayerSet::Opponent, {amount})','Skip(1)','CounterRelated']
    else:return None
    return [cost,'Skip(1)','CounterRelated']


def targeted_program(text):
    """Complete target occurrences with independent predicates and effect focus."""
    if not isinstance(text, str) or not text.endswith('.'):
        return None
    text = strip_rule_label(text)
    text = normalize_duration(text[0].lower() + text[1:])
    # Exact subject/cardinality mapping shared by announcement and resolution.
    object_subjects={
        'target creature':'Creature','target creature you control':'CreatureYou',
        'target creature an opponent controls':'CreatureOpponent',
        'target creature or Vehicle':'CreatureVehicle',
        'target creature or Vehicle an opponent controls':'CreatureVehicleOpponent',
        'target nonland permanent':'Nonland','other target creature':'OtherCreature',
        'other target permanent you control':'OtherPermanentYou',
        'other target nonland permanent you control':'OtherNonlandYou',
        'target tapped creature':'CreatureTapped','target artifact or creature':'ArtifactCreature',
        'target nonland permanent an opponent controls':'NonlandOpponent','target land':'Land',
    }
    if m:=re.fullmatch(r"return (up to one |)(.+) to its owner's hand\.",text):
        if rule:=object_subjects.get(m[2]):
            return ((rule,0,1) if m[1] else rule),['ReturnToHand']
    if m:=re.fullmatch(r'(tap|untap) (up to one |)(.+)\.',text):
        if rule:=object_subjects.get(m[3]):
            spec=(rule,0,1) if m[2] else (rule,1,1) if rule=='Creature' else rule
            return spec,[m[1].capitalize()]
    if m:=re.fullmatch(r'tap target creature an opponent controls and put (a|one|two|three) stun counters? on it\.',text):
        return 'CreatureOpponent',['Tap',counter_instruction('Targets',m[1],'stun')]
    if m:=re.fullmatch(r'([^.]+) if you gained life this turn\.',text):
        result=targeted_program(m[1]+'.')
        return (result[0],[f'If {{ guard: StepGuard::GainedLife, skip: {len(result[1])} }}',*result[1]]) if result else None
    if m:=re.fullmatch(r'(target creature|target creature you control|another target creature you control) gets ([+-])X/([+-])X until end of turn, where X is (.+)\.',text):
        value=amount_value(m[4])
        target={'target creature':'Creature','target creature you control':'CreatureYou','another target creature you control':'OtherCreatureYou'}[m[1]]
        power,toughness=(1 if sign=='+' else -1 for sign in [m[2],m[3]])
        return (target,[f'GrowScaled {{ subject: Subject::Targets, amount: {value}, power: {power}, toughness: {toughness} }}']) if value else None
    if program:=scaled_damage_program(text):return program
    if m:=re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on each of (up to two|up to three|one or two|two) target creatures( you control)?\.',text):
        spec=('CreatureYou' if m[4] else 'Creature',0 if m[3].startswith('up to') else 1 if m[3]=='one or two' else 2,3 if m[3]=='up to three' else 2)
        return spec,[counter_instruction('Targets',m[1],m[2])]
    if text=='double the number of each kind of counter on up to two target creatures and/or artifacts you control.':
        return ('ArtifactCreatureYou',0,2),['DoubleCounters']
    if counter:=counter_program(text): return counter
    if text=='exile up to one target card from a graveyard.': return ('CardGrave',0,1),['Exile']
    if text=='exile up to two target cards from graveyards.': return ('CardGrave',0,2),['Exile']
    if text=='target player draws a card, then up to one target creature you control connives.':
        return ['Player',('CreatureYou',0,1)],['Draw(PlayerSet::Targeted, 1)','SelectGroup(1)','Connive { subject: Subject::Targets, amount: 1 }']
    if m:=re.fullmatch(r'(up to one )?target creature( you control| an opponent controls)? connives(?: ([0-9]+))?\.',text):
        amount=int(m[3]) if m[3] else 1
        if amount>65535: return None
        rule={None:'Creature',' you control':'CreatureYou',' an opponent controls':'CreatureOpponent'}[m[2]]
        return ((rule,0,1) if m[1] else rule),[f'Connive {{ subject: Subject::Targets, amount: {amount} }}']
    if text=='two target players each draw a card.': return ('Player',2,2),['Draw(PlayerSet::Targeted, 1)']
    if m:=re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on target creature\. It gains (.+) until end of turn\.',text):
        keywords=keyword_list(m[3])
        if keywords is None: return None
        return 'Creature',[counter_instruction('Targets',m[1],m[2]),f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({keywords}) }}']
    if m:=re.fullmatch(r'(target creature or Vehicle)( an opponent controls)? gets ([+-][0-9]+)/([+-][0-9]+) until end of turn\.',text):
        if max(abs(int(m[3])),abs(int(m[4])))>32767: return None
        return ('CreatureVehicleOpponent' if m[2] else 'CreatureVehicle'),[grow_instruction('Targets',int(m[3]),int(m[4]))]
    if m:=re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on up to one target creature or Vehicle\. It gains (.+) until end of turn\.',text):
        keywords=keyword_list(m[3])
        if keywords is None: return None
        return ('CreatureVehicle',0,1),[counter_instruction('Targets',m[1],m[2]),f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({keywords}) }}']
    if m := re.fullmatch(r'attach this Equipment to target creature you control\.(?: That creature gains (.+) until end of turn\.)?', text):
        program=['AttachSource']
        if m[1]:
            extra=keyword_list(m[1])
            if extra is None: return None
            program.append(f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({extra}) }}')
        return 'CreatureYou',program
    if m:=re.fullmatch(r'target (player|opponent) mills (.+)\.',text):
        amount=mill_quantity(m[2])
        program=([mill_instruction('Targeted',amount)] if amount is not None else
                 ['Mill(PlayerSet::Targeted, MillAmount::ControllerLife)'] if m[2]=='cards equal to your life total' else None)
        if program: return ('Player' if m[1]=='player' else 'Opponent'),program
    if choice := optional_payment_program(text, True): return choice
    if sacrifice := sacrifice_program(text):
        if sacrifice[0] != 'None': return sacrifice
    if result := power_damage_program(text): return result
    if m := re.fullmatch(r'(target creature|target creature you control|another target creature you control|target creature an opponent controls) can\'t block this turn\.', text):
        rule = {'target creature': 'Creature', 'target creature you control': 'CreatureYou',
                'another target creature you control': 'OtherCreatureYou',
                'target creature an opponent controls': 'CreatureOpponent'}[m[1]]
        return rule, ['GrantKeywords { subject: Subject::Targets, keywords: Keywords(2048) }']
    if m := re.fullmatch(r'(another target creature you control|target creature you control|target creature) (.+)', text):
        rule = {'target creature': 'Creature', 'target creature you control': 'CreatureYou', 'another target creature you control': 'OtherCreatureYou', 'target creature an opponent controls':'CreatureOpponent'}[m[1]]
        if program := keyword_effect(m[2], 'Targets'): return rule, program
    if m := re.fullmatch(r'(another target creature you control|target creature you control|target creature) (.+)', text):
        rule = {'target creature': 'Creature', 'target creature you control': 'CreatureYou', 'another target creature you control': 'OtherCreatureYou', 'target creature an opponent controls':'CreatureOpponent'}[m[1]]
        if m[2] == "can't be blocked this turn.":
            return rule, ['GrantKeywords { subject: Subject::Targets, keywords: Keywords(16384) }']
        if m[2].endswith(" and can't be blocked this turn."):
            clause = m[2][:-len(" and can't be blocked this turn.")] + '.'
            if program := keyword_effect(clause, 'Targets'):
                return rule, program + ['GrantKeywords { subject: Subject::Targets, keywords: Keywords(16384) }']
    if text == "exile target player's graveyard.": return 'Player', ['ExileGraveyard(PlayerSet::Targeted)']
    if m := re.fullmatch(r'(destroy|exile) (.+?)\.', text):
        rule = REMOVAL_SUBJECTS.get(m[2])
        if rule and (m[1] == 'exile' or 'graveyard' not in m[2]):
            return rule, [m[1].capitalize()]
    if m := re.fullmatch(r'exile (.+?)( until this (?:creature|artifact|enchantment) leaves the battlefield|)\.', text):
        subject, linked = m.groups()
        if subject in EXILE_SUBJECTS:
            return EXILE_SUBJECTS[subject], ['ExileUntilSourceLeaves' if linked else 'Exile']
    if m := re.fullmatch(r'you may have (target creature|target creature you control|another target creature you control) get ([+-][0-9]+)/([+-][0-9]+) until end of turn\.', text):
        result = targeted_program(f'{m[1]} gets {m[2]}/{m[3]} until end of turn.')
        return (result[0], [f'May {{ skip: {len(result[1])} }}', *result[1]]) if result else None
    # Optional effects still require targets at announcement (CR603.5).
    if text.startswith('you may '):
        result = targeted_program(text[len('you may '):])
        if result:
            spec, program = result
            return spec, [f'May {{ skip: {len(program)} }}', *program]
    if m := re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on target Vampire you control\.', text):
        return 'VampireYou', [counter_instruction('Targets', m[1], m[2])]
    if m := re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on (target creature|target creature you control|another target creature you control|target creature an opponent controls|up to one target creature|up to one target creature you control)\.', text):
        rules = {'target creature': 'Creature', 'target creature you control': 'CreatureYou',
                 'another target creature you control': 'OtherCreatureYou',
                 'target creature an opponent controls': 'CreatureOpponent',
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
        'untap one or two target creatures. They each get +2/+2 until end of turn.': (('Creature', 1, 2), ['Untap', grow_instruction('Targets', 2, 2)]),
        "return target creature card from your graveyard to the battlefield.": ('CreatureGraveYou', ['ReturnToBattlefield']),
        "return another target creature card from your graveyard to your hand.": ('OtherCreatureGraveYou', ['ReturnToHand']),
        "return target creature card from your graveyard to your hand.": ('CreatureGraveYou', ['ReturnToHand']),
        "return target instant or sorcery card from your graveyard to your hand.": ('InstantSorceryGraveYou', ['ReturnToHand']),
        "return another target permanent you control to its owner's hand.": ('OtherPermanentYou', ['ReturnToHand']),
        "return another target creature you control to its owner's hand.": ('OtherCreatureYou', ['ReturnToHand']),
        "return target creature card from a graveyard to its owner's hand.": ('CreatureGrave', ['ReturnToHand']),
        'destroy target creature.': ('Creature', ['Destroy']),
        'destroy target artifact.': ('Artifact', ['Destroy']),
        'destroy target enchantment.': ('Enchantment', ['Destroy']),
        'destroy target artifact or enchantment.': ('ArtifactEnchantment', ['Destroy']),
        'destroy up to one target artifact or enchantment.': (('ArtifactEnchantment',0,1),['Destroy']),
        "return up to one other target creature you control to its owner's hand.": (('OtherCreatureYou',0,1),['ReturnToHand']),
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
    if text=='target opponent gets two poison counters.':
        return 'Opponent',['AddPoison(PlayerSet::Targeted, 2)']
    if m := re.fullmatch(r'target (player|opponent) (gains|loses) ([0-9]+) life\.', text):
        if int(m[3]) > 65535: return None
        return ('Player' if m[1] == 'player' else 'Opponent'), [f'{"GainLife" if m[2] == "gains" else "LoseLife"}(PlayerSet::Targeted, {int(m[3])})']
    if m:=re.fullmatch(r'target opponent loses ([0-9]+) life and you gain ([0-9]+) life\.',text):
        loss,gain=map(int,m.groups())
        if max(loss,gain)>65535:return None
        return 'Opponent',[f'LoseLife(PlayerSet::Targeted, {loss})',f'GainLife(PlayerSet::Controller, {gain})']
    if m := re.fullmatch(r'this (?:creature|artifact|enchantment|permanent|spell) deals ([0-9]+) damage to ('+'|'.join(map(re.escape,FIXED_DAMAGE_SUBJECTS))+r')(?: and you gain ([0-9]+) life)?\.', text):
        rule=FIXED_DAMAGE_SUBJECTS[m[2]]
        if int(m[1])>32767 or (m[3] and int(m[3])>65535):return None
        return rule,[f'Damage({int(m[1])})']+([f'GainLife(PlayerSet::Controller, {int(m[3])})'] if m[3] else [])
    if m := re.fullmatch(r'(target creature|target creature you control|another target creature you control|target creature an opponent controls) gets ([+-][0-9]+)/([+-][0-9]+) until end of turn\.', text):
        if max(abs(int(m[2])), abs(int(m[3]))) > 32767: return None
        rule = {'target creature': 'Creature', 'target creature you control': 'CreatureYou', 'another target creature you control': 'OtherCreatureYou', 'target creature an opponent controls':'CreatureOpponent'}[m[1]]
        return rule, [grow_instruction('Targets', int(m[2]), int(m[3]))]
    # A targeted clause followed by wholly non-targeted instructions is supported.
    if '. ' in text:
        first, rest = text.split('. ', 1)
        result = targeted_program(first + '.')
        tail = statement_program(rest)
        if result:
            spec=result[0]
            rule=spec[0] if isinstance(spec,tuple) else spec
            maximum=spec[2] if isinstance(spec,tuple) else 1
            if maximum==1 and rule in ('Creature','CreatureYou','OtherCreatureYou','OtherCreature','CreatureOpponent'):
                if replacement:=guarded_clause((strip_rule_label(rest)[0].lower()+strip_rule_label(rest)[1:])):
                    guard,body=replacement
                    if body.endswith(' until end of turn instead.'):
                        alternative=focused_creature_program(body.removesuffix(' instead.')+'.')
                        if alternative and all(p.startswith(('Grow {','GrantKeywords {')) for p in result[1]+alternative):
                            return spec,[f'If {{ guard: {guard}, skip: {len(alternative)+1} }}',*alternative,f'Skip({len(result[1])})',*result[1]]
                tail=focused_creature_program(rest)
                if tail and not valid_creature_focus(result[1]+tail):return None
        if result and rest == 'Untap it.':
            if isinstance(result[0], list): return None
            rule = result[0][0] if isinstance(result[0], tuple) else result[0]
            maximum = result[0][2] if isinstance(result[0], tuple) else 1
            if maximum == 1 and rule not in ('None', 'Player', 'Opponent', 'Spell', 'CreatureGrave', 'CreatureGraveYou'): tail = ['Untap']
        if result and tail:
            spec, program = result
            return spec, program + tail
        if result and (second := targeted_program(rest)):
            if 'another target ' in rest.casefold(): return None
            left = result[0] if isinstance(result[0], list) else [result[0]]
            right = second[0] if isinstance(second[0], list) else [second[0]]
            offset = len(left)
            if offset + len(right) > 64: return None
            rebased = []
            for instruction in second[1]:
                instruction = re.sub(r'(source: |recipient: )([0-9]+)', lambda m: m[1] + str(int(m[2]) + offset), instruction)
                instruction = re.sub(r'SelectGroup\(([0-9]+)\)', lambda m: f'SelectGroup({int(m[1]) + offset})', instruction)
                rebased.append(instruction)
            return left + right, result[1] + [f'SelectGroup({offset})'] + rebased
    return None

def cast_condition(relation='You', filter_='Any', ordinal=None):
    ordinal = 'None' if ordinal is None else f'Some({ordinal})'
    return f'SpellCast {{ relation: CastRelation::{relation}, filter: CastFilter::{filter_}, ordinal: {ordinal} }}'


def step_guard(text):
    if text == 'you gained life this turn': return 'StepGuard::GainedLife'
    if text == 'you attacked this turn': return 'StepGuard::Attacked'
    if text == 'a creature died this turn': return 'StepGuard::CreatureDied'
    if text == 'an opponent lost life this turn': return 'StepGuard::OpponentLostLife'
    if text == 'this creature is tapped': return 'StepGuard::SourceTapped'
    def threshold(value, minimum=None, maximum=None):
        lo = 'None' if minimum is None else f'Some({minimum})'
        hi = 'None' if maximum is None else f'Some({maximum})'
        return f'StepGuard::Threshold {{ value: StepValue::{value}, minimum: {lo}, maximum: {hi} }}'
    def expression(value, minimum):
        return f'StepGuard::Expression {{ value: {value}, minimum: Some({minimum}), maximum: None }}'
    if m:=re.fullmatch(r'you control (?:a|an|another) (.+)',text):
        noun=m[1]
        if ' or ' not in noun and (value:=stat_value(('other ' if text.startswith('you control another ') else '')+noun+' you control')):
            return expression(value,1)
    if m:=re.fullmatch(r'(?:you control|there are) (two|three|four|five|six|seven|eight|[0-9]+) or more (.+)',text):
        n={**COUNT_WORDS,'seven':7,'eight':8}.get(m[1],int(m[1]) if m[1].isdigit() else 0)
        noun=m[2]+(' you control' if text.startswith('you control ') else '')
        if n<=65535 and (value:=stat_value(noun)):return expression(value,n)
    if text == "you didn't cast a spell this turn": return threshold('SpellsCast', maximum=0)
    if m := re.fullmatch(r'you control a creature with power ([0-9]+) or greater', text):
        return threshold('GreatestCreaturePower', minimum=int(m[1])) if int(m[1]) <= 32767 else None
    if m := re.fullmatch(r'you control another creature with power ([0-9]+) or greater', text):
        return threshold('GreatestOtherCreaturePower',minimum=int(m[1])) if int(m[1])<=32767 else None
    if m := re.fullmatch(r'creatures you control have total (power|toughness) ([0-9]+) or greater', text):
        return threshold('TotalCreaturePower' if m[1]=='power' else 'TotalCreatureToughness', minimum=int(m[2])) if int(m[2]) <= 32767 else None
    if m := re.fullmatch(r'you control (two|three|four|five|six|[0-9]+) or more (tapped creatures|creatures|lands)', text):
        n = COUNT_WORDS.get(m[1], int(m[1]) if m[1].isdigit() else 0)
        return threshold({'tapped creatures':'TappedCreatures','creatures':'Creatures','lands':'Lands'}[m[2]],minimum=n) if n <= 65535 else None
    if m := re.fullmatch(r'you have fewer than ([0-9]+) cards in hand',text):
        n=int(m[1]); return threshold('Hand',maximum=n-1) if 0<n<=65535 else None
    if m := re.fullmatch(r'you have ([0-9]+) or (more|less) life',text):
        n=int(m[1]); return threshold('Life',minimum=n) if m[2]=='more' and n<=65535 else threshold('Life',maximum=n) if n<=65535 else None
    return None


def step_condition(step, relation='You', guard='StepGuard::Always'):
    return f'AtStep {{ step: TriggerStep::{step}, relation: CastRelation::{relation}, guard: {guard} }}'


STEP_PREFIXES = {}
for _subject, _step in [('upkeep','Upkeep'),('draw step','Draw'),('end step','End')]:
    for _when, _relation in [('your '+_subject,'You'),('each '+_subject,'Any'),("each opponent's "+_subject,'Opponent')]:
        STEP_PREFIXES['At the beginning of '+_when+', '] = step_condition(_step,_relation)
for _text, _step, _relation in [
    ('At the beginning of combat on your turn, ','BeginCombat','You'),
    ('At the beginning of each combat, ','BeginCombat','Any'),
    ("At the beginning of combat on each opponent's turn, ",'BeginCombat','Opponent'),
    ('At end of combat, ','EndCombat','Any'),
    ('At the end of combat on your turn, ','EndCombat','You'),
    ('At the beginning of your first main phase, ','Main1','You'),
    ('At the beginning of your precombat main phase, ','Main1','You'),
    ('At the beginning of your second main phase, ','Main2','You'),
    ('At the beginning of your postcombat main phase, ','Main2','You'),
    ('At the beginning of each of your main phases, ','MainPhases','You'),
    ('At the beginning of the end step, ','End','Any'),
]: STEP_PREFIXES[_text] = step_condition(_step,_relation)


def life_condition(first=False,own_turn=False):
    return f'LifeGained {{ first_only: {str(first).lower()}, own_turn: {str(own_turn).lower()} }}'


def death_condition(relation='You',other=False,nontoken=False,subtype=None):
    subtype='None' if subtype is None else 'Some('+json.dumps(subtype)+')'
    return (f'CreatureDies {{ relation: CastRelation::{relation}, other: {str(other).lower()}, '
            f'nontoken: {str(nontoken).lower()}, subtype: {subtype} }}')


VITAL_PREFIXES = {
    'Whenever you gain life, ': life_condition(),
    'Whenever you gain life for the first time each turn, ': life_condition(first=True),
    'Whenever you gain life for the first time during each of your turns, ': life_condition(first=True,own_turn=True),
}
for _quantifier,_other in [('a',False),('another',True)]:
    for _token in ['', 'nontoken ']:
        for _ownership,_relation in [(' you control','You'),(' an opponent controls','Opponent'),('','Any')]:
            VITAL_PREFIXES[f'Whenever {_quantifier} {_token}creature{_ownership} dies, '] = death_condition(_relation,_other,bool(_token))
# Exact, single-subtype events. Other qualifiers, unions and batching remain gated.
for _subtype in ('Goblin','Rat','Insect','Elf','Zombie','Vampire','Human','Ally','Hero','Villain'):
    for _quantifier,_other in [('a',False),('another',True)]:
        VITAL_PREFIXES[f'Whenever {_quantifier} {_subtype} you control dies, '] = death_condition(other=_other,subtype=_subtype)

# Exact public battlefield departures, independently of whether they were
# sacrificed, destroyed, or put into a graveyard by a state-based action.
for _phrase, _kinds, _relation, _exclude_source_kinds in [
    ('an enchantment you control',32,'You',0),
    ('an artifact you control',16,'You',0),
    ('an artifact',16,'Any',0),
    ('another artifact you control',16,'You',16),
    ('another artifact or creature',17,'Any',17),
    ('another creature you control or a land you control',3,'You',1),
]:
    VITAL_PREFIXES[f'Whenever {_phrase} is put into a graveyard from the battlefield, '] = (
        f'PermanentToGraveyard {{ kinds: {_kinds}, relation: CastRelation::{_relation}, exclude_source_kinds: {_exclude_source_kinds} }}')


# Single sacrifice events only. Event quantities and sacrificed-object pronouns
# are not bound to an unrelated ability-cost context.
for _phrase in ('an artifact','a Food','a Clue','another creature or artifact','another permanent','a permanent','a token','a permanent during your turn'):
    _spec=sacrifice_spec(_phrase.replace(' during your turn',''))
    VITAL_PREFIXES['Whenever you sacrifice '+_phrase+', ']=(
        'Sacrificed { filter: '+_spec[1]+', own_turn: '+str(' during your turn' in _phrase).lower()+' }')

VITAL_PREFIXES['Whenever another creature you control with deathtouch dies, ']=(
    'CreatureDiesFiltered { filter: '+creature_filter('other creatures you control with deathtouch')+' }')



def player_damage_prefix(line):
    match=re.match(r'Whenever (.+?) deals? (combat )?damage to (a player|an opponent), ',line)
    if not match: return None
    descriptor=match[1]
    one=descriptor.startswith('one or more ')
    if one: descriptor=descriptor[len('one or more '):]
    if descriptor=='this creature': subject='DamageSubject::Source'
    elif descriptor in ('equipped creature','enchanted creature'): subject='DamageSubject::Attached'
    else:
        qualification='Any'
        if descriptor.startswith('a '):
            if one: return None
            descriptor=descriptor[2:]
            if descriptor.startswith('modified creature '):
                descriptor=descriptor.replace('modified creature ','modified creatures ',1)
            elif descriptor.startswith('nontoken artifact creature '): descriptor=descriptor.replace('artifact creature ','artifact creatures ',1)
            elif descriptor.startswith('creature '): descriptor=descriptor.replace('creature ','creatures ',1)
            else: return None
        elif not one: return None
        if descriptor.startswith('modified creatures '):
            qualification='Modified';descriptor=descriptor[len('modified '):]
        for suffix,kind in [(' with a counter on it','Counter'),(' with counters on them','Counter'),
                            (' with +1/+1 counters on them','PlusOne')]:
            if descriptor.endswith(suffix):
                if qualification!='Any': return None
                qualification=kind;descriptor=descriptor[:-len(suffix)];break
        if descriptor=='Heroes you control':
            filter_=creature_filter('creatures you control').replace('subtype: None','subtype: Some("Hero")')
        else: filter_=creature_filter(descriptor)
        if filter_ is None: return None
        subject=f'DamageSubject::Creatures {{ filter: {filter_}, qualification: DamageQualification::{qualification} }}'
    condition=(f'PlayerDamaged {{ subject: {subject}, combat_only: {str(bool(match[2])).lower()}, '
               f'opponent_only: {str(match[3]=="an opponent").lower()}, one_or_more: {str(one).lower()} }}')
    return match[0],condition


def damage_event_program(text):
    # A damage-sized quantity binds only inside a player-damage trigger. Ordinary
    # spell/step/entry parsing cannot infer an event from 'that many'.
    normalized=text[0].lower()+text[1:] if text else ''
    if normalized=='that player gets two poison counters.':
        return 'None',['AddPoison(PlayerSet::DamagedPlayer, 2)']
    if m:=re.fullmatch(r'(?:you )?draw that many cards(, then discard a card)?\.',normalized):
        return 'None',['DrawDamageAmount']+(['Discard(PlayerSet::Controller, 1)'] if m[1] else [])
    targeted=targeted_program(text)
    if targeted: return targeted
    program=statement_program(text)
    return ('None',program) if program else None


def discard_condition(relation='You',one_or_more=False):
    return f'Discarded {{ relation: CastRelation::{relation}, one_or_more: {str(one_or_more).lower()} }}'


def discard_event_program(text):
    normalized=text[0].lower()+text[1:] if text else ''
    if re.fullmatch(r'put that many \+1/\+1 counters on this creature\.',normalized):
        return 'None',['PutDiscardCounters(CounterKind::PlusOne)']
    if re.fullmatch(r'this creature deals that much damage to each opponent\.',normalized):
        return 'None',['DamageDiscardAmount(PlayerSet::Opponent)']
    targeted=targeted_program(text)
    if targeted: return targeted
    program=statement_program(text)
    return ('None',program) if program else None


def draw_condition(relation='You',ordinal=None):
    ordinal='None' if ordinal is None else f'Some({ordinal})'
    return f'CardDrawn {{ relation: CastRelation::{relation}, ordinal: {ordinal} }}'


def draw_event_program(text):
    normalized=text[0].lower()+text[1:] if text else ''
    if m:=re.fullmatch(r'that player loses ([0-9]+) life\.',normalized):
        return ('None',[f'LoseLife(PlayerSet::DrawingPlayer, {int(m[1])})']) if int(m[1])<=65535 else None
    if m:=re.fullmatch(r'this creature deals ([0-9]+) damage to them\.',normalized):
        return ('None',[f'DamagePlayers(PlayerSet::DrawingPlayer, {int(m[1])})']) if int(m[1])<=32767 else None
    if normalized=='you create a Treasure token.': normalized=normalized[4:]
    targeted=targeted_program(normalized)
    if targeted: return targeted
    program=statement_program(normalized)
    return ('None',program) if program else None


def trigger_mode_lines(card, text):
    """Compile every alternative before replacing a whole exact trigger header."""
    lines=text.split('\n');body=[];modal={};i=0
    while i<len(lines):
        line=lines[i].strip();i+=1
        if not line.endswith(', choose one —'):
            body.append(line);continue
        header=line[:-len('choose one —')];bullets=[]
        while i<len(lines) and lines[i].strip().startswith('• '):
            bullets.append(lines[i].strip());i+=1
        modes=modal_spell_template(dict(card,type_line='Instant',oracle_text='Choose one —\n'+'\n'.join(bullets)))
        # The existing exact trigger parser authenticates event/guard/source scope.
        parsed=trigger_creature_template(dict(card,oracle_text=header+'draw a card.'))
        if modes is None or parsed is None or len(parsed[2])!=1:
            return None
        condition,_,_=parsed[2][0]
        data='&['+', '.join(f'SpellMode {{ text: {json.dumps(label,ensure_ascii=False)}, targets: {rust_targets(rule)}, program: {rust_program(program)} }}' for label,rule,program in modes)+']'
        key=f'__rustmage_modal_trigger_{len(modal)}__'
        modal[key]=(condition,['Modal('+data+')'],'None');body.append(key)
    return body,modal


def trigger_creature_template(card):
    if not isinstance(card.get('oracle_text'), str):
        return None
    # Reuse the same complete body/type/cost checks as the keyword-only template.
    if creature_template(dict(card, oracle_text='')) is None:
        return None
    keywords = 0
    abilities = []
    prefixes = {
        'Whenever you draw a card, ': draw_condition(),
        'Whenever you draw your first card each turn, ': draw_condition(ordinal=1),
        'Whenever you draw your second card each turn, ': draw_condition(ordinal=2),
        'Whenever an opponent draws a card, ': draw_condition(relation='Opponent'),
        'Whenever an opponent draws their first card each turn, ': draw_condition(relation='Opponent',ordinal=1),
        'Whenever an opponent draws their second card each turn, ': draw_condition(relation='Opponent',ordinal=2),
        'Whenever a player draws a card, ': draw_condition(relation='Any'),
        'Whenever a player draws their second card each turn, ': draw_condition(relation='Any',ordinal=2),
        'When you discard this card, ': 'SelfDiscarded',
        'When you cycle this card, ': 'SelfCycled',
        'Whenever you discard a card, ': discard_condition(),
        'Whenever you discard one or more cards, ': discard_condition(one_or_more=True),
        'Whenever an opponent discards a card, ': discard_condition(relation='Opponent'),
        'Whenever an opponent discards one or more cards, ': discard_condition(relation='Opponent',one_or_more=True),
        'Whenever a player discards a card, ': discard_condition(relation='Any'),
        'Whenever a player discards one or more cards, ': discard_condition(relation='Any',one_or_more=True),
        'Whenever you cast a spell, ': cast_condition(),
        'Whenever you cast a creature spell, ': cast_condition(filter_='Creature'),
        'Whenever you cast a noncreature spell, ': cast_condition(filter_='Noncreature'),
        'Whenever you cast an instant or sorcery spell, ': cast_condition(filter_='InstantSorcery'),
        'Whenever you cast an artifact spell, ': cast_condition(filter_='Artifact'),
        'Whenever you cast an enchantment spell, ': cast_condition(filter_='Enchantment'),
        'Whenever an opponent casts a spell, ': cast_condition(relation='Opponent'),
        'Whenever a player casts a spell, ': cast_condition(relation='Any'),
        'Whenever you cast your second spell each turn, ': cast_condition(ordinal=2),
        'Flurry — Whenever you cast your second spell each turn, ': cast_condition(ordinal=2),
        'When this creature enters, ': 'SelfEnters',
        'When this creature dies, ': 'SelfDies',
        'When this creature enters or dies, ': 'SelfEntersOrDies',
        **{f'When this {noun} is put into a graveyard from the battlefield, ': 'SelfDies'
           for noun in ('creature','artifact','enchantment','Aura','Equipment')},
        **{f'When this {noun} enters or is put into a graveyard from the battlefield, ': 'SelfEntersOrDies'
           for noun in ('creature','artifact','enchantment','Aura','Equipment')},
        'Whenever this creature attacks, ': 'SelfAttacks',
        'Whenever this creature enters or attacks, ': 'SelfEntersOrAttacks',
        'Whenever another artifact you control enters, ': 'OtherArtifactEntersYou',
        'Whenever an artifact you control enters, ': 'ArtifactEntersYou',
        'Whenever an enchantment you control enters, ': 'EnchantmentEntersYou',
        'Whenever another creature you control enters, ': 'OtherCreatureEntersYou',
        'Whenever another Villain you control enters, ': 'OtherSubtypeEntersYou("Villain")',
        'Whenever this creature or another creature you control enters, ': 'CreatureEntersYou',
        'Landfall — Whenever a land you control enters, ': 'LandEntersYou',
        **{
            f'Whenever this creature becomes the target of {kind}{who}, ':
                f'SelfTargeted {{ relation: CastRelation::{relation}, spells_only: {str(spells_only).lower()} }}'
            for kind,spells_only in [('a spell or ability',False),('a spell',True)]
            for who,relation in [(' you control','You'),(' an opponent controls','Opponent'),('','Any')]
        },
        **STEP_PREFIXES,
        **VITAL_PREFIXES,
    }

    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    grouped=trigger_mode_lines(card,text)
    if grouped is None:return None
    lines,modal=grouped
    for line in lines:
        if line in modal:
            abilities.append(modal[line]);continue
        line = line.strip()
        line = strip_rule_label(line)
        for label in ('Do You Like Squirrels? — ', 'I LOVE Squirrels! — ', 'Vivid — ', 'Valiant — '):
            if line.startswith(label):line=line[len(label):]
        limited = line.endswith(' This ability triggers only once each turn.')
        if limited: line = line[:-len(' This ability triggers only once each turn.')]
        first_target = 'Whenever this creature becomes the target of a spell or ability you control for the first time each turn, '
        if line.startswith(first_target):
            # With immutable abilities and no control changes, every qualifying
            # first target is observed; per-object trigger use records this event.
            if limited:return None
            limited=True
            line='Whenever this creature becomes the target of a spell or ability you control, '+line[len(first_target):]
        name = card.get('name', '')
        names = [name] if name else []
        if name and card.get('type_line', '').startswith('Legendary '):
            names.append(name.split(',')[0])
            if ' the ' in name:names.append(name.split(' the ')[0])
        for alias in names:
            if line.startswith('Whenever ' + alias + ' becomes the target of '):
                line='Whenever this creature becomes the target of '+line[len('Whenever ' + alias + ' becomes the target of '):]
            if line.startswith('Whenever ' + alias + ' attacks, '):
                line = 'Whenever this creature attacks, ' + line[len('Whenever ' + alias + ' attacks, '):]
            # An authenticated plural source name still denotes this object.
            line=line.replace('When '+alias+' enter, ','When this creature enters, ')
            if line.startswith('When ' + alias + ' enters, '):
                line = 'When this creature enters, ' + line[len('When ' + alias + ' enters, '):]
            if line.startswith('When ' + alias + ' dies, '):
                line = 'When this creature dies, ' + line[len('When ' + alias + ' dies, '):]
        if line.startswith('Raid — When '):line=line[len('Raid — '):]
        for label in ('Raid — ', 'Formidable — ', 'Survival — ', 'Morbid — '):
            if line.startswith(label + 'At the beginning of '): line = line[len(label):]
        for alias in names:
            line=re.sub(r'^Whenever '+re.escape(alias)+r' deals? ((?:combat )?damage to )',r'Whenever this creature deals \1',line)
            line=line.replace('Whenever '+alias+' enters or attacks, ','Whenever this creature enters or attacks, ')
        for alias in names:
            if line.startswith(alias+' '):
                candidate='This creature'+line[len(alias):]
                if candidate.casefold() in POWER_BLOCKING_TEXT_FLAGS:line=candidate
        if line.casefold() in TEXT_FLAGS:
            if limited: return None
            keywords |= TEXT_FLAGS[line.casefold()]; continue
        if ward:=nonmana_ward_program(line):
            if limited:return None
            abilities.append(('Ward',ward,'None'))
            continue
        if m:=re.fullmatch(r'Firebending ([0-9]+)',line):
            if limited or int(m[1])>65535:return None
            abilities.append(('SelfAttacks',[f'AddCombatRed({int(m[1])})'],'None'));continue
        if line=="Firebending X, where X is the number of creatures you control.":
            if limited:return None
            abilities.append(('SelfAttacks',['AddCombatRedScaled('+amount_value('the number of creatures you control')+')'],'None'));continue
        words = [word.strip().casefold() for word in line.split(',') if word.strip()]
        if words and all(word in INTRINSIC_KEYWORDS or word == 'prowess' or ward_word(word) is not None for word in words):
            if limited: return None
            keywords |= sum(INTRINSIC_KEYWORDS[word] for word in set(words) if word in INTRINSIC_KEYWORDS)
            for word in words:
                if word=='prowess': abilities.append((cast_condition(filter_='Noncreature'),[grow_instruction('Source',1,1)],'None'))
                elif (cost:=ward_word(word)) is not None: abilities.append(('Ward',[counter_unless(cost,'Related')],'None'))
            continue
        for alias in names:
            prefix='Whenever '+alias+' or another '
            if line.startswith(prefix):line='Whenever this creature or another '+line[len(prefix):]
        damage_prefix=player_damage_prefix(line)
        line_prefixes=[damage_prefix] if damage_prefix else []
        if m:=re.match(r'Whenever (a|an|another) ([A-Z][a-z]+) you control enters, ',line):
            kind='OtherSubtypeEntersYou' if m[1]=='another' else 'SubtypeEntersYou'
            line_prefixes.append((m[0],kind+'('+json.dumps(m[2])+')'))
        if m:=re.match(r'Whenever (this creature or another |another )(nontoken )?(creature|artifact|[A-Z][a-z]+) you control enters, ',line):
            include=m[1].startswith('this creature')
            if m[2] or (include and m[3]!='creature'):
                noun=m[3]
                if noun in ('creature','artifact'):kinds,subtype=({'creature':1,'artifact':16}[noun],'None')
                elif noun in CREATURE_TYPE_NAMES:kinds,subtype=(0,'Some('+json.dumps(noun)+')')
                else:return None
                condition=f'PermanentEntersYou {{ kinds: {kinds}, nontoken: {str(bool(m[2])).lower()}, subtype: {subtype}, include_source: {str(include).lower()} }}'
                line_prefixes.insert(0,(m[0],condition))
        # These first-trigger limits collapse simultaneous event multiplicity.
        # Do not generalize to unrestricted batched events without batch context.
        if limited:
            for prefix,condition in {
                'Whenever one or more artifacts you control enter, ': 'ArtifactEntersYou',
                'Whenever one or more enchantments you control enter, ': 'EnchantmentEntersYou',
                'Whenever one or more nontoken creatures you control die, ': death_condition(nontoken=True),
                'Whenever one or more players discard one or more cards, ': discard_condition(relation='Any',one_or_more=True),
            }.items():
                line_prefixes.append((prefix,condition))
        for prefix, condition in [*line_prefixes,*prefixes.items()]:
            if line.startswith(prefix):
                effect_text = line[len(prefix):]
                intervening=None
                if effect_text.startswith('if '):
                    clause=guarded_clause(effect_text)
                    if clause is None:return None
                    guard,effect_text=clause
                    if condition.startswith('AtStep {'):
                        condition=condition.replace('StepGuard::Always',guard)
                    elif (condition in ('SelfEnters','SelfAttacks','SelfEntersOrAttacks','OtherCreatureEntersYou','CreatureEntersYou','ArtifactEntersYou','OtherArtifactEntersYou','EnchantmentEntersYou','LandEntersYou')
                            or condition.startswith(('SpellCast {','LifeGained {','CardDrawn {','PlayerDamaged {','OtherSubtypeEntersYou(','SubtypeEntersYou(','PermanentEntersYou {'))):
                        intervening=guard
                    else:return None  # Lookback-event conditions need prior-event state.
                # Only self-event subjects prove an initial 'it' denotes this source.
                # Other-creature events and later targeted clauses have other antecedents.
                if condition in ('SelfEnters', 'SelfDies', 'SelfEntersOrDies', 'SelfAttacks', 'SelfEntersOrAttacks', 'SelfDiscarded', 'SelfCycled'):
                    effect_text = re.sub(r'^(?:[Ii]t|[Hh]e|[Ss]he) deals ', 'this creature deals ', effect_text)
                    if m:=re.fullmatch(r'(?:it|he|she) connives(?: ([0-9]+))?\.',effect_text):
                        effect_text='this creature connives'+(' '+m[1] if m[1] else '')+'.'
                if condition in ('SelfEnters','SelfAttacks','SelfEntersOrAttacks') and re.fullmatch(r'[Pp]ut (?:a|one|two|three|four|five|six) (?:\+1/\+1|-1/-1|stun) counters? on it if you gained life this turn\.',effect_text):
                    effect_text=effect_text.replace(' on it if ',' on this creature if ')
                if condition.startswith('SelfTargeted {'):
                    # Initial growth and a whole counter sentence have this
                    # self-event as their sole antecedent. Do not bind later it.
                    effect_text=re.sub(r'^[Ii]t gets ', 'this creature gets ', effect_text)
                    if re.fullmatch(r'[Pp]ut (?:a|one|two|three|four|five|six) (?:\+1/\+1|-1/-1|stun) counters? on it\.',effect_text):
                        effect_text=effect_text.replace(' on it.',' on this creature.')
                for alias in names:
                    if effect_text.startswith(alias + ' deals '):
                        effect_text = 'this creature deals ' + effect_text[len(alias) + len(' deals '):]
                    if m:=re.fullmatch(re.escape(alias)+r' connives(?: ([0-9]+))?\.',effect_text):
                        effect_text='this creature connives'+(' '+m[1] if m[1] else '')+'.'
                    if effect_text=='untap '+alias+'.':effect_text='untap this creature.'
                    if effect_text.startswith(alias + ' gets '):
                        effect_text = 'this creature gets ' + effect_text[len(alias) + len(' gets '):]
                # An explicit occurrence of this card's name denotes this source;
                # event-subject pronouns still have no inferred binding.
                for alias in names:
                    effect_text = effect_text.replace(' on ' + alias + '.', ' on this creature.')
                if (condition.startswith('PlayerDamaged { subject: DamageSubject::Source,')
                        and re.fullmatch(r'[Pp]ut (?:a|one|two|three|four|five|six) (?:\+1/\+1|-1/-1|stun) counters? on it\.',effect_text)):
                    # Only this whole source-counter sentence proves its antecedent.
                    # A later 'it' can refer to an earlier target or another object.
                    effect_text=effect_text.replace(' on it.',' on this creature.')
                if condition.startswith('CardDrawn {'):
                    result=draw_event_program(effect_text)
                    targets,program=result if result else ('None',None)
                elif prefix == 'Whenever one or more players discard one or more cards, ':
                    # An aggregated multi-player event has no unique that-player
                    # or quantity binding. Only this complete event-free body is
                    # currently admitted.
                    if effect_text.casefold() != 'investigate.': return None
                    targets,program='None',statement_program(effect_text)
                elif condition.startswith('Discarded {'):
                    result=discard_event_program(effect_text)
                    targets,program=result if result else ('None',None)
                elif condition.startswith('PlayerDamaged {'):
                    result=damage_event_program(effect_text)
                    targets,program=result if result else ('None',None)
                else:
                    targeted = targeted_program(effect_text)
                    if targeted:
                        targets, program = targeted
                    else:
                        targets, program = 'None', statement_program(effect_text)
                if program is None:
                    return None
                if intervening is not None:
                    condition=f'InterveningIf {{ guard: {intervening}, event: &TriggerCondition::{condition} }}'
                if limited: condition='OncePerTurn(&TriggerCondition::'+condition+')'
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
            or spell_cost_options(card.get('mana_cost', '')) is None):
        return None
    text = card.get('oracle_text')
    if isinstance(text, str): text = ' '.join(re.sub(r'\([^()]*\)', '', text).split())
    if isinstance(text, str) and card.get('name'):
        # Only exact sentence-start self references, including after an earlier
        # instruction. Other card names and unsupported clauses remain intact.
        text=re.sub(r'(^|\. )'+re.escape(card['name'])+r' deals ',r'\1this spell deals ',text)
    targeted = targeted_program(text)
    if targeted: return targeted
    program = statement_program(text)
    return ('None', program) if program else None


def modal_spell_template(card):
    """Every mode must compile completely; unsupported alternatives gate the card."""
    if (card.get('layout') != 'normal' or card.get('type_line') not in ('Instant', 'Sorcery')
            or spell_cost_options(card.get('mana_cost', '')) is None
            or not isinstance(card.get('oracle_text'), str)):
        return None
    lines = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip().split('\n')
    if not lines or lines[0] != 'Choose one —' or not 2 <= len(lines) - 1 <= 64:
        return None
    modes = []
    for line in lines[1:]:
        if not line.startswith('• '): return None
        text = line[2:].strip()
        body = text
        if ' — ' in body:
            label, body = body.split(' — ', 1)
            if not re.fullmatch(r"[A-Z][A-Za-z0-9' -]*", label): return None
        compiled = spell_program_template(dict(card, oracle_text=body))
        if compiled is None: return None
        modes.append((text, *compiled))
    return modes


def activation_cost(text):
    cost = dict(colored=[0] * 6, generic=0, alternatives=[], additional=None, mana_component=False, tap=False, sacrifice=False, exile_self=False, life=0, remove=None, loyalty=None, guard="Always")
    seen = set()
    external_sacrifice=False
    for component in text.split(', '):
        mana = spell_cost_options(component)
        if mana:
            if 'mana' in seen: return None
            seen.add('mana')
            cost['mana_component'] = True
            cost['colored'], cost['generic'] = mana[0]
            cost['alternatives'] = mana[1:]
        elif component == 'Discard a card':
            if cost['additional'] is not None: return None
            cost['additional'] = 'AdditionalCost { kind: EffectChoiceKind::Discard, filter: '+sacrifice_spec('a permanent')[1]+' }'
        elif component == '{T}':
            if cost['tap']: return None
            cost['tap'] = True
        elif component in ('Sacrifice this creature', 'Sacrifice this artifact', 'Sacrifice this enchantment', 'Sacrifice this permanent'):
            if cost['sacrifice']: return None
            cost['sacrifice'] = True
        elif component == 'Exile this card from your graveyard':
            if cost['exile_self']: return None
            cost.update(exile_self=True, origin='Graveyard')
        elif component.startswith('Sacrifice ') and (spec:=sacrifice_spec(component[len('Sacrifice '):])) is not None:
            if cost['additional'] is not None or spec[0]!=1:return None
            cost['additional']='AdditionalCost { kind: EffectChoiceKind::Sacrifice, filter: '+spec[1]+' }'
            external_sacrifice=True
        elif m := re.fullmatch(r'Pay ([0-9]+) life', component):
            if cost['life'] or int(m[1]) > 65535: return None
            cost['life'] = int(m[1])
        elif m := re.fullmatch(r'Remove (a|one|two|three|four|five|six) (?:(\+1/\+1|-1/-1|stun) )?counters? from this (?:creature|artifact|enchantment|permanent)', component):
            if cost['remove']: return None
            cost['remove'] = (COUNTER_KINDS[m[2]] if m[2] else None, COUNT_WORDS[m[1]])
        else:
            return None
    # Wider composite costs need payment-order choices before admission.
    if cost['additional'] is not None and (cost['life'] or cost['remove'] or cost['loyalty'] is not None): return None
    if external_sacrifice and cost['sacrifice']:return None
    if cost['exile_self'] and (cost['tap'] or cost['sacrifice'] or cost['life'] or cost['remove'] or cost['additional'] is not None or cost['loyalty'] is not None): return None
    return cost


def aura_template(card):
    if (card.get('layout') != 'normal' or permanent_type(card) != 'Enchantment'
            or card.get('type_line','').split(' — ')[-1] != 'Aura'
            or not isinstance(card.get('oracle_text'),str)
            or spell_cost_options(card.get('mana_cost','')) is None): return None
    text=re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    body=[]
    power,toughness,keywords=0,0,0
    relation=None
    for line in text.split('\n'):
        line=line.strip()
        if line in ('Enchant creature','Enchant creature you control','Enchant creature an opponent controls'):
            if relation is not None: return None
            relation={'Enchant creature':-1,'Enchant creature you control':0,'Enchant creature an opponent controls':1}[line]
        elif m:=re.fullmatch(r'Enchanted creature gets ([+-][0-9]+)/([+-][0-9]+)(?: and has (.+))?\.',line):
            if max(abs(int(m[1])),abs(int(m[2])))>32767: return None
            extra=keyword_list(m[3]) if m[3] else 0
            if extra is None: return None
            power+=int(m[1]);toughness+=int(m[2]);keywords|=extra
        elif m:=re.fullmatch(r'Enchanted creature has (.+)\.',line):
            extra=keyword_list(m[1])
            if extra is None: return None
            keywords|=extra
        elif line.startswith('Enchanted creature ') and ('this creature '+line[len('Enchanted creature '):]).casefold() in TEXT_FLAGS:
            keywords|=TEXT_FLAGS[('this creature '+line[len('Enchanted creature '):]).casefold()]
        else:
            body.append(line.replace('When this Aura enters, ','When this enchantment enters, '))
    if relation is None: return None
    result=permanent_program_template(dict(card,type_line='Enchantment',oracle_text='\n'.join(body)))
    return (*result,relation,power,toughness,keywords) if result else None


def equipment_template(card):
    if (card.get('layout') != 'normal' or permanent_type(card) != 'Artifact'
            or card.get('type_line', '').split(' — ')[-1] != 'Equipment'
            or not isinstance(card.get('oracle_text'),str)
            or spell_cost_options(card.get('mana_cost','')) is None): return None
    text=re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    body=[]
    power,toughness,keywords,equips=0,0,0,0
    for line in text.split('\n'):
        line=line.strip()
        if m:=re.fullmatch(r'Equipped creature gets ([+-][0-9]+)/([+-][0-9]+)(?: and has (.+))?\.',line):
            if max(abs(int(m[1])),abs(int(m[2])))>32767: return None
            extra=keyword_list(m[3]) if m[3] else 0
            if extra is None: return None
            power+=int(m[1]);toughness+=int(m[2]);keywords|=extra
        elif m:=re.fullmatch(r'Equipped creature has (.+)\.',line):
            extra=keyword_list(m[1])
            if extra is None: return None
            keywords|=extra
        elif m:=re.fullmatch(r'Equip ((?:\{(?:[0-9]+|[WUBRGC])\})+)',line):
            equips+=1
            body.append(m[1]+': Attach this Equipment to target creature you control. Activate only as a sorcery.')
        else:
            if line.startswith('When this Equipment enters, attach it to '):
                line=line.replace('enters, attach it to ','enters, attach this Equipment to ',1)
            body.append(line.replace('When this Equipment enters, ','When this artifact enters, '))
    if not equips: return None
    # Source-cost aliases are exact; granted abilities and residual clauses still fail.
    body=[line.replace('Sacrifice this Equipment','Sacrifice this artifact') for line in body]
    result=permanent_program_template(dict(card,type_line='Artifact',oracle_text='\n'.join(body)))
    return (*result,power,toughness,keywords) if result else None


def source_return_program(text):
    """Exact activated source returns; never infer a death-trigger zone binding."""
    if text == 'Return this card from your graveyard to your hand.':
        return 'Graveyard', ['ReturnSourceToHand(TargetZone::Graveyard)']
    if text in ('Return this card from your graveyard to the battlefield.',
                'Return this card from your graveyard to the battlefield tapped.'):
        return 'Graveyard', ['ReturnSourceToBattlefield { tapped: '+str(text.endswith(' tapped.')).lower()+' }']
    if re.fullmatch(r"Return this (?:creature|artifact|enchantment|permanent|Aura) to its owner's hand\.", text):
        return 'Battlefield', ['ReturnSourceToHand(TargetZone::Battlefield)']
    return None


def permanent_program_template(card):
    """Whole supported permanent text, including all activation restrictions."""
    if (card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str)
            or spell_cost_options(card.get('mana_cost', '')) is None):
        return None
    typ = permanent_type(card)
    if set(card.get('type_line','').split(' — ')[-1].split()) & {'Equipment','Aura'}: return None
    if typ not in ('Creature', 'Artifact Creature', 'Enchantment Creature', 'Artifact', 'Enchantment', 'Planeswalker'):
        return None
    creature = 'Creature' in typ
    if creature and (not card.get('power', '').isdigit() or not card.get('toughness', '').isdigit()):
        return None
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    activations, body = [], []
    enters = [0, 0, 0]
    for line in text.split('\n'):
        # Removing a reminder can leave trailing spaces before a newline.
        line=line.strip()
        for label in ('Do You Like Squirrels? — ', 'I LOVE Squirrels! — '):
            if line.startswith(label):line=line[len(label):]
        # These aliases denote the source only, not any arbitrary mentioned name.
        line = line.replace('When this artifact enters, ', 'When this creature enters, ').replace('When this enchantment enters, ', 'When this creature enters, ')
        if m := re.fullmatch(r'This creature enters with (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on it\.', line):
            if not creature: return None
            enters[list(COUNTER_KINDS).index(m[2])] += COUNT_WORDS[m[1]]
            continue
        if ': ' not in line:
            # Authenticate source aliases before the creature-shaped proxy
            # drops legendary type text. Only the whole untap clause is bound.
            name=card.get('name','')
            names=[name] if name else []
            if name and card.get('type_line','').startswith('Legendary '):
                names.append(name.split(',')[0])
                if ' the ' in name:names.append(name.split(' the ')[0])
            for alias in names:
                tail=', untap '+alias+'.'
                if line.endswith(tail):line=line[:-len(tail)]+', untap this creature.'
            body.append(line)
            continue
        renew=line.startswith('Renew — ')
        if renew: line=line[len('Renew — '):].strip()
        exhaust=line.startswith('Exhaust — ')
        if exhaust:line=line[len('Exhaust — '):].strip()
        cost_text, effect_text = line.split(': ', 1)
        # Exact source-name cost components, with the same authenticated short
        # legendary names used by self-event triggers. Never rewrite effect text.
        name = card.get('name', '')
        aliases = [name] if name else []
        if name and card.get('type_line', '').startswith('Legendary '):
            aliases.append(name.split(',')[0])
            if ' the ' in name: aliases.append(name.split(' the ')[0])
        for alias in sorted(set(aliases), key=len, reverse=True):
            cost_text = re.sub(r'(^|, )Sacrifice '+re.escape(alias)+r'(?=, |$)', r'\1Sacrifice this permanent', cost_text)
        if typ == 'Planeswalker':
            if not re.fullmatch(r'(?:0|[+−][0-9]+)',cost_text): return None
            change=int(cost_text.replace('−','-'))
            if change < -2147483648 or change > 2147483647: return None
            cost=activation_cost('{0}')
            cost.update(mana_component=False, loyalty=change)
        else:
            cost = activation_cost(cost_text)
        if cost is None: return None
        # Loyalty use resets each turn; exhaustion persists across turns. Their
        # shared use bits cannot represent both limits on the same ability.
        if exhaust and cost['loyalty'] is not None: return None
        cost['exhaust']=exhaust
        if guard:=re.fullmatch(r'(.+) Activate only if you control (?:a|an) ([A-Z][A-Za-z ]+?)(?: or (?:a|an) ([A-Z][A-Za-z ]+?))?\.',effect_text):
            subtypes=[word for word in guard.groups()[1:] if word is not None]
            vocabulary={'Plains','Island','Swamp','Mountain','Forest',*SUBTYPE_PLURALS.values()}
            if any(word not in vocabulary for word in subtypes): return None
            cost['guard']='ControlSubtypeAny(&['+', '.join(json.dumps(word) for word in sorted(set(subtypes)))+'])'
            effect_text=guard[1]
        if effect_text.endswith(' Activate only during your turn.'):
            if cost['guard'] != 'Always': return None
            cost['guard'] = 'OwnTurn'
            effect_text = effect_text[:-len(' Activate only during your turn.')]
        combined = effect_text.endswith(' Activate only as a sorcery and only once each turn.')
        if combined: effect_text = effect_text[:-len(' Activate only as a sorcery and only once each turn.')]
        sorcery = combined or effect_text.endswith(' Activate only as a sorcery.')
        once = combined or effect_text.endswith(' Activate only once each turn.')
        if sorcery and not combined: effect_text = effect_text[:-len(' Activate only as a sorcery.')]
        if once and not combined: effect_text = effect_text[:-len(' Activate only once each turn.')]
        if renew and (not cost['exile_self'] or not sorcery): return None
        for alias in aliases:
            if effect_text.startswith(alias+' deals '):
                effect_text='this permanent deals '+effect_text[len(alias+' deals '):]
        # This complete source-counter sentence proves the following keyword
        # pronoun. No intervening target or later pronoun receives a guessed bind.
        if re.fullmatch(r'Put (?:a|one|two|three|four|five|six) (?:\+1/\+1|-1/-1|stun) counters? on this creature\. It gains? [^.]+ until end of turn\.',effect_text):
            effect_text=effect_text.replace('. It ','. This creature ',1)
        # With an external card cost, an initial pronoun may refer to that card.
        if cost['additional'] is not None and re.match(r'^[Ii]t deals ', effect_text): return None
        # The remaining source aliases refer only to the source, mana, life or its counters;
        # an initial pronoun in this activation therefore denotes the source.
        effect_text = re.sub(r'^[Ii]t deals ', 'this permanent deals ', effect_text)
        for alias in aliases:
            if effect_text == 'Return '+alias+" to its owner's hand.":
                effect_text = "Return this permanent to its owner's hand."
        source_return = source_return_program(effect_text)
        if source_return:
            origin, program = source_return
            if cost.get('origin', origin) != origin: return None
            if origin == 'Graveyard' and (cost['tap'] or cost['sacrifice'] or cost['remove'] or cost['loyalty'] is not None or exhaust): return None
            cost['origin'] = origin
            targets = 'None'
        else:
            targeted = targeted_program(effect_text)
            if targeted: targets, program = targeted
            else: targets, program = 'None', statement_program(effect_text)
        if program is None: return None
        if any(instruction.startswith('AddMana') for instruction in program) and (len(program) != 1 or targets != 'None'):
            return None
        # Counterfactual mana search does not consume hand-card resources yet.
        if program[0].startswith('AddMana') and (cost['additional'] is not None or cost['exile_self']): return None
        if program[0].startswith('AddMana') and cost['loyalty'] is None and not (cost['tap'] or cost['sacrifice'] or cost['life'] or cost['remove'] or once or exhaust):
            # Unbounded positive production needs an infinite-resource solver.
            # Resource-free conversions that do not increase total mana are finite.
            fixed = re.search(r'Fixed\((\[[^]]*\])\)', program[0])
            produced = sum(json.loads(fixed[1])) if fixed else int(re.search(r'amount: ([0-9]+)', program[0])[1])
            if produced > min(sum(mana)+generic for mana,generic in [(cost['colored'],cost['generic']),*cost['alternatives']]): return None
        activations.append((cost, targets, program, sorcery or cost['loyalty'] is not None, once))
    if len(activations) > 64: return None
    body_text = '\n'.join(body).strip()
    proxy = dict(card, type_line='Creature', power=card.get('power', '0'),
                 toughness=card.get('toughness', '1'), oracle_text=body_text)
    template = creature_template(proxy)
    trigger = trigger_creature_template(proxy)
    if template: effect, keywords = template; triggers = []
    elif trigger: effect, keywords, triggers = trigger
    else: return None
    # Sacrificing a discard observer before versus after discarding may change
    # which events it observes. Do not hide that cost-order decision.
    if triggers and any(a[0]['additional'] is not None and a[0]['sacrifice'] for a in activations): return None
    if not creature: effect = 'Permanent'
    if not activations and not any(enters) and creature: return None
    return effect, keywords, triggers, activations, enters


# These are replacements evaluated before entry, not triggered abilities.
LAND_ENTRY = {
    'This land enters tapped.': 'Always',
    'This land enters tapped unless you control a basic land.': 'UnlessBasic',
    'This land enters tapped unless you control two or fewer other lands.': 'UnlessAtMostTwo',
    'This land enters tapped unless you control two or more other lands.': 'UnlessAtLeastTwo',
    'This land enters tapped unless a player has 13 or less life.': 'UnlessLowLife',
    'This land enters tapped unless you control a planeswalker.': 'UnlessPlaneswalker',
}


def land_template(card):
    """Complete normal lands; intrinsic basic-type mana is explicit reminder text."""
    if (card.get('layout') != 'normal' or card.get('mana_cost', '') != ''
            or permanent_type(card) not in ('Land', 'Basic Land')
            or not isinstance(card.get('oracle_text'), str)):
        return None
    text = card['oracle_text']
    intrinsic = []
    # A basic land type confers its mana ability even though it is reminder text.
    if match := re.match(r'^\((\{T\}: Add [^()]+\.)\)(?:\n|$)', text):
        if statement_program(match[1].split(': ', 1)[1]) is None: return None
        intrinsic.append(match[1])
        text = text[match.end():]
    subtypes = card.get('type_line', '').split(' — ')[1:]
    basic_types = set(subtypes[0].split()) & {'Plains', 'Island', 'Swamp', 'Mountain', 'Forest'} if subtypes else set()
    expected = {'Plains': 'W', 'Island': 'U', 'Swamp': 'B', 'Mountain': 'R', 'Forest': 'G'}
    intrinsic_symbols = set(re.findall(r'\{([WUBRGC])\}', ''.join(intrinsic)))
    if basic_types and intrinsic_symbols != {expected[t] for t in basic_types}: return None
    if intrinsic and not basic_types: return None
    text = re.sub(r'\([^()]*\)', '', text).strip()
    entry, lines = 'Never', list(intrinsic)
    for line in text.split('\n'):
        line = line.strip()
        if not line: continue
        if match := re.fullmatch(r"As this land enters, you may pay ([1-9][0-9]*) life\. If you don't, it enters tapped\.",line):
            if entry != 'Never' or int(match[1]) > 65535: return None
            entry = f'PayLife({int(match[1])})'
        elif line in LAND_ENTRY:
            if entry != 'Never': return None
            entry = LAND_ENTRY[line]
        else:
            lines.append(line.replace('this land', 'this artifact'))
    proxy = dict(card, type_line='Artifact', mana_cost='{0}', oracle_text='\n'.join(lines))
    result = permanent_program_template(proxy)
    if result is None: return None
    _, keywords, triggers, activations, enters = result
    return 'Land', keywords, triggers, activations, enters, entry


def cycling_abilities(card):
    """Extract only complete fixed-mana cycling lines; compile all residual text."""
    text=card.get('oracle_text')
    if not isinstance(text,str) or card.get('layout')!='normal': return card,[]
    body,abilities=[],[]
    for line in text.split('\n'):
        clean=re.sub(r'\([^()]*\)', '',line).strip()
        match=re.fullmatch(r'(Cycling|Basic landcycling|Plainscycling|Islandcycling|Swampcycling|Mountaincycling|Forestcycling|Halflingcycling) ((?:\{(?:[0-9]+|[WUBRGC])\})+)',clean)
        if match is None:
            body.append(line);continue
        kind,mana=match.groups()
        if mana_cost_spec(mana) is None:
            body.append(line);continue
        cost=activation_cost(mana)
        cost.update(discard=True,origin='Hand',cycling=True)
        if kind=='Cycling': program=['Draw(PlayerSet::Controller, 1)']
        else:
            filter_='BasicLand' if kind=='Basic landcycling' else 'Subtype('+json.dumps(kind.removesuffix('cycling'))+')'
            program=[f'Search(SearchSpec {{ filter: SearchFilter::{filter_}, maximum: 1, destination: SearchDestination::Hand, tapped: false, reveal: true }})']
        abilities.append((cost,'None',program,False,False))
    return dict(card,oracle_text='\n'.join(body)),abilities


def rust_activations(abilities):
    rows = []
    for cost, targets, program, sorcery, once in abilities:
        remove = 'None' if cost['remove'] is None else f"Some(({'None' if cost['remove'][0] is None else 'Some(CounterKind::' + cost['remove'][0] + ')'}, {cost['remove'][1]}))"
        alternatives = '&[' + ', '.join(f'ManaCost {{ colored: {mana}, generic: {generic} }}' for mana,generic in cost.get('alternatives', [])) + ']'
        rows.append(f"ActivatedAbility {{ exhaust: {str(cost.get('exhaust',False)).lower()}, guard: ActivationGuard::{cost.get('guard','Always')}, cycling: {str(cost.get('cycling',False)).lower()}, origin: ActivationOrigin::{cost.get('origin','Battlefield')}, cost: ActivationCost {{ colored: {cost['colored']}, generic: {cost['generic']}, alternatives: {alternatives}, additional: {'None' if cost.get('additional') is None else 'Some('+cost['additional']+')'}, mana_component: {str(cost['mana_component']).lower()}, tap: {str(cost['tap']).lower()}, sacrifice: {str(cost['sacrifice']).lower()}, exile_self: {str(cost['exile_self']).lower()}, discard: {str(cost.get('discard',False)).lower()}, life: {cost['life']}, remove: {remove}, loyalty: {'None' if cost.get('loyalty') is None else 'Some('+str(cost['loyalty'])+')'} }}, targets: {rust_targets(targets)}, program: {rust_program(program)}, sorcery: {str(sorcery).lower()}, once_per_turn: {str(once).lower()} }}")
    return '&[' + ', '.join(rows) + ']'


def rust_program(program):
    return '&[' + ', '.join('Instruction::' + instruction for instruction in program) + ']'

def compile_catalogue(snapshot, target):
    TOKEN_DEFS.clear()
    EMBLEM_DEFS.clear()
    creature_types_bytes = (ROOT / 'data/creature-types.json').read_bytes()
    creature_types = json.loads(creature_types_bytes)['types']
    if (creature_types != sorted(set(creature_types)) or not creature_types
            or any(not re.fullmatch(r"[A-Z][A-Za-z'-]*",name) and name != 'Time Lord' for name in creature_types)):
        raise ValueError('creature types must be sorted unique names; Time Lord is the two-word type')
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
    color_masks = []
    exhaust_rows = []
    uncounterable_rows = []
    loyalty_rows = []
    supported = []
    coverage = []
    static_rows = []
    grant_rows = []
    dynamic_rows = []
    scaling_rows = []
    additional_rows = []
    hybrid_rows = []
    reduction_rows = []
    entry_counter_rows = []
    equipment_rows = []
    aura_rows = []
    for index, name in enumerate(names):
        card, uncounterable = extract_uncounterable(cards.get(name, {}))
        card, cycling = cycling_abilities(card)
        card, additional = extract_additional_cost(card)
        card, reductions = extract_cost_reductions(card)
        card, dynamic, scaling = extract_stat_rules(card)
        card, creature_entry = extract_creature_entry_tapped(card)
        card, entry_counters = extract_entry_counters(card)
        cost = card.get('mana_cost', '')
        keywords = sum(INTRINSIC_KEYWORDS[word] for word in {keyword.casefold() for keyword in card.get('keywords', [])} & INTRINSIC_KEYWORDS.keys())
        effect = 'Unsupported'
        color = None
        template = creature_template(card)
        trigger_template = trigger_creature_template(card)
        spell = spell_program_template(card)
        modal = modal_spell_template(card)
        aura = aura_template(card)
        equipment = equipment_template(card)
        permanent = permanent_program_template(card)
        static = static_permanent_template(card)
        activations, enters = [], [0, 0, 0]
        abilities = []
        entry = 'Never'
        land = land_template(card)
        if name in legacy:
            effect, fallback_cost, color = legacy[name]
            # Lands have no mana cost. Their legacy cost token encodes colour only.
            cost = '' if effect == 'Land' else card.get('mana_cost', fallback_cost)
            kind = 2 if effect == 'Land' else 1 if effect.startswith('Creature') else 4 if effect in ('Draw',) else 8
        else:
            types = card.get('type_line', '').split(' — ')[0].split()
            kind = (1 if 'Creature' in types else 0) | (2 if 'Land' in types else 0) | (4 if 'Sorcery' in types else 0) | (8 if 'Instant' in types else 0) | (16 if 'Artifact' in types else 0) | (32 if 'Enchantment' in types else 0) | (64 if 'Planeswalker' in types else 0) | (128 if 'Kindred' in types else 0) | (256 if 'Battle' in types else 0)
            if land is not None:
                effect, keywords, abilities, activations, enters, entry = land
            elif template is not None:
                effect, keywords = template
            elif trigger_template is not None:
                effect, keywords, abilities = trigger_template
            elif modal is not None:
                effect = 'Modal'
            elif spell is not None:
                effect = 'Spell'
            elif aura is not None:
                _, keywords, abilities, activations, enters, relation, power, toughness, granted = aura
                target = {-1:'Creature',0:'CreatureYou',1:'CreatureOpponent'}[relation]
                effect = f'Aura({rust_targets(target)})'
                aura_rows.append(f'        {index} => Some(AuraSpec {{ relation: {relation}, bonus: EquipmentSpec {{ stats: Stats {{ power: {power}, toughness: {toughness} }}, keywords: Keywords({granted}) }} }}),')
            elif equipment is not None:
                effect, keywords, abilities, activations, enters, power, toughness, granted = equipment
                equipment_rows.append(f'        {index} => Some(EquipmentSpec {{ stats: Stats {{ power: {power}, toughness: {toughness} }}, keywords: Keywords({granted}) }}),')
            elif permanent is not None:
                effect, keywords, abilities, activations, enters = permanent
            elif static is not None:
                effect, keywords, abilities, activations, enters, modifiers, grants = static
                if modifiers: static_rows.append(f'        {index} => &[{", ".join(modifiers)}],')
                if grants: grant_rows.append(f'        {index} => &[{", ".join(grants)}],')
        if effect != 'Unsupported':
            if creature_entry != 'Never':entry=creature_entry
            activations = [*activations, *cycling]
            if len(activations) > 64: effect = 'Unsupported'; activations = []
        programs = {
            'Damage(3)': ('Any', ['Damage(3)']), 'Damage(2)': ('Any', ['Damage(2)']),
            'Grow': ('Creature', [grow_instruction('Targets', 3, 3)]), 'Draw': ('None', ['Draw(PlayerSet::Controller, 2)']),
            'Counter': ('Spell', ['Counter']),
        }
        if effect in programs:
            rule, program = programs[effect]
            effect = f'Spell({rust_targets(rule)}, {rust_program(program)})'
        elif effect == 'Modal':
            effect = 'Modal(&[' + ', '.join(f'SpellMode {{ text: {json.dumps(text, ensure_ascii=False)}, targets: {rust_targets(rule)}, program: {rust_program(program)} }}' for text, rule, program in modal) + '])'
        elif effect == 'Spell':
            rule, program = spell
            effect = f'Spell({rust_targets(rule)}, {rust_program(program)})'
        triggers_rust = '&[' + ', '.join(f'TriggeredAbility {{ condition: TriggerCondition::{condition}, targets: {rust_targets(targets)}, program: {rust_program(program)} }}' for condition, program, targets in abilities) + ']'

        if kind & 64:
            face = card
            if 'loyalty' not in face:
                faces = card.get('card_faces')
                if isinstance(faces,list) and faces and isinstance(faces[0],dict):face=faces[0]
            value=face.get('loyalty')
            if not isinstance(value,str) or not value.isdigit() or int(value)>18446744073709551615:
                raise ValueError(f'invalid or missing starting loyalty: {name}')
            loyalty_rows.append(f'        {index} => {int(value)},')
        if effect != 'Unsupported':
            supported.append(index)
            if entry_counters: entry_counter_rows.append(f'        {index} => &[{", ".join(entry_counters)}],')
            if reductions: reduction_rows.append(f'        {index} => &[{", ".join(reductions)}],')
            if additional is not None: additional_rows.append(f'        {index} => Some({additional}),')
        basic = name in {'Plains', 'Island', 'Swamp', 'Mountain', 'Forest'} or card.get('type_line', '').split(' — ')[0] == 'Basic Land'
        costs = spell_cost_options(cost) or [([0] * 6, 0)]
        colored, generic = costs[0]
        if len(costs)>1:
            hybrid_rows.append(f'        {index} => &['+', '.join(f'ManaCost {{ colored: {mana}, generic: {n} }}' for mana,n in costs[1:])+'],')
        deck_limit = 'None' if basic or name in UNLIMITED else 'Some(4)'
        quote = lambda value: json.dumps(value, ensure_ascii=False)
        oracle_id = f"Some({quote(card['oracle_id'])})" if card else 'None'
        type_line = card.get('type_line', {'Grizzly Bears':'Creature — Bear','Goblin Piker':'Creature — Goblin Warrior','Hill Giant':'Creature — Giant'}.get(name, 'Basic Land' if basic else 'Creature' if kind & 1 else 'Instant' if kind & 8 else 'Sorcery' if kind & 4 else ''))
        if card:
            # Multiple-face cards are still unsupported; expose front-face facts
            # rather than deriving colors from color identity or one hybrid cost.
            colors = card.get('colors')
            if colors is None:
                faces = card.get('card_faces')
                if isinstance(faces, list) and faces and isinstance(faces[0], dict):
                    colors = faces[0].get('colors')
            if (not isinstance(colors, list)
                    or not all(isinstance(color, str) and len(color) == 1 and color in 'WUBRG' for color in colors)
                    or len(colors) != len(set(colors))):
                raise ValueError(f'invalid or missing colors: {name}')
            color_mask = sum(1 << 'WUBRG'.index(color) for color in colors)
        else:
            color_mask = sum(1 << i for i in range(5) if colored[i])
        color_masks.append(color_mask)
        oracle_keywords = '&[' + ', '.join(quote(word) for word in card.get('keywords', [])) + ']'
        if effect != 'Unsupported':
            if dynamic is not None:dynamic_rows.append(f'        {index} => Some({dynamic}),')
            if scaling:scaling_rows.append(f'        {index} => &['+', '.join(scaling)+'],')
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {quote(name)}, oracle_id: {oracle_id}, mana_cost: {quote(cost)}, type_line: {quote(type_line)}, oracle_keywords: {oracle_keywords}, standard_legal: {str(name in cards).lower()}, legendary: {str(card.get('type_line', '').startswith('Legendary ')).lower()}, implemented: {str(effect != 'Unsupported').lower()}, keywords: Keywords({keywords}), deck_limit: {deck_limit} }}, effect: Effect::{effect}, triggers: {triggers_rust}, activated: {rust_activations(activations)}, enters: Counters {{ plus_one: {enters[0]}, minus_one: {enters[1]}, stun: {enters[2]}, loyalty: 0 }}, entry: EntryTapped::{entry}, kind: {kind}, basic: {str(basic).lower()}, mana_color: {'None' if color is None else f'Some({color})'}, colored: {colored}, generic: {generic} }},")
        if uncounterable and effect != 'Unsupported': uncounterable_rows.append(str(index))
        exhaust_mask=sum(1<<i for i,a in enumerate(activations) if a[0].get('exhaust',False))
        if exhaust_mask and effect!='Unsupported':exhaust_rows.append(f'        {index} => {exhaust_mask},')
        coverage.append({'name': name, 'standard_legal': name in cards, 'implemented': effect != 'Unsupported',
                         'implementation': 'explicit' if name in legacy else 'land_template' if land else 'creature_template' if template else 'trigger_template' if trigger_template else 'modal_template' if modal else 'spell_template' if spell else 'aura_template' if aura else 'equipment_template' if equipment else 'permanent_template' if permanent else 'static_template' if static else None})

    # Only factories referenced by wholly accepted cards enter the executable catalogue.
    used_tokens = sorted(set(re.findall(r'__TOKEN_([0-9a-f]+)__', '\n'.join(definitions))))
    token_base = len(names)
    token_infos = []
    for offset, key in enumerate(used_tokens):
        factory = TOKEN_DEFS[key]
        index = token_base + offset
        name = '@token/' + key
        names.append(name)

        predefined = factory.get('predefined')
        if predefined:
            effect, _, _, token_abilities, _ = permanent_program_template({
                'layout': 'normal', 'type_line': 'Artifact', 'mana_cost': '{0}',
                'oracle_text': PREDEFINED[predefined]})
            type_line, kind = 'Artifact — ' + factory['subtypes'], 16
        else:
            types = ((factory['extra_type'].title() + ' ') if factory['extra_type'] else '') + 'Creature'
            type_line = types + ' — ' + factory['subtypes']
            kind = 1 | (16 if factory['extra_type'] == 'artifact' else 32 if factory['extra_type'] == 'enchantment' else 0)
            effect = f"Creature({factory['power']}, {factory['toughness']})"
            token_abilities = []
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {json.dumps(name)}, oracle_id: None, mana_cost: \"\", type_line: {json.dumps(type_line, ensure_ascii=False)}, oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: Keywords({factory['keywords']}), deck_limit: None }}, effect: Effect::{effect}, triggers: &[], activated: {rust_activations(token_abilities)}, enters: Counters {{ plus_one: 0, minus_one: 0, stun: 0, loyalty: 0 }}, entry: EntryTapped::Never, kind: {kind}, basic: false, mana_color: None, colored: [0; 6], generic: 0 }},")
        token_infos.append(f"    TokenInfo {{ name: {json.dumps(factory['name'])}, subtypes: {json.dumps(factory['subtypes'])}, colors: {factory['colors']} }},")
        color_masks.append(factory['colors'])
        marker = '__TOKEN_' + key + '__'
        definitions = [definition.replace(marker, f'Card({index})') for definition in definitions]
    used_emblems=sorted(set(re.findall(r'__EMBLEM_([0-9a-f]+)__', '\n'.join(definitions))))
    emblem_base=len(names)
    emblem_infos=[]
    for key in used_emblems:
        index=len(names);name='@emblem/'+key;names.append(name);color_masks.append(0)
        body=EMBLEM_DEFS[key]
        triggers='&[TriggeredAbility { condition: TriggerCondition::LifeGained { first_only: false, own_turn: false }, targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player { opponent_only: true })), program: &[Instruction::LoseGainedLife(PlayerSet::Targeted)] }]'
        definitions.append(f'Definition {{ info: CardInfo {{ name: {json.dumps(name)}, oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: Keywords(0), deck_limit: None }}, effect: Effect::Emblem, triggers: {triggers}, activated: &[], enters: Counters {{ plus_one: 0, minus_one: 0, stun: 0, loyalty: 0 }}, entry: EntryTapped::Never, kind: 0, basic: false, mana_color: None, colored: [0; 6], generic: 0 }},')
        emblem_infos.append(f'EmblemInfo {{ oracle_text: {json.dumps(body)} }},')
        definitions=[row.replace('__EMBLEM_'+key+'__',f'Card({index})') for row in definitions]
    if len(names) > 65536:
        raise ValueError('catalogue exceeds u16 identity space')
    # The fingerprint covers data and compiler logic; card IDs are local to it.
    target_bytes = json.dumps(target, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    version = hashlib.sha256(canonical + target_bytes + creature_types_bytes + Path(__file__).read_bytes()).hexdigest()
    lines = ['// Generated by scripts/generate_catalogue.py. Do not edit.',
             f'pub const CATALOGUE_VERSION: &str = "{version}";',
             '#[rustfmt::skip]', f'pub(super) static COLORS: [u8; {len(names)}] = {color_masks};',
             '#[rustfmt::skip]', f'pub(super) static DEFINITIONS: [Definition; {len(names)}] = [', *definitions, '];',
             '#[rustfmt::skip]', f'pub(super) static NAME_INDEX: [(&str, u16); {len(names)}] = [']
    lines += [f'    ({json.dumps(name, ensure_ascii=False)}, {index}),' for name, index in sorted((name, index) for index, name in enumerate(names))]
    lines += ['];', f'pub(super) const TOKEN_BASE: usize = {token_base};', f'pub(super) const EMBLEM_BASE: usize = {emblem_base};', '#[rustfmt::skip]', f'pub(super) static EMBLEMS: [EmblemInfo; {len(emblem_infos)}] = [', *emblem_infos, '];',
              '#[rustfmt::skip]', f'pub(super) static TOKENS: [TokenInfo; {len(token_infos)}] = [', *token_infos, '];',
              '#[allow(non_upper_case_globals)]', 'impl Card {', '#[rustfmt::skip]',
              f'    pub const ALL: [Card; {len(supported)}] = [{", ".join(f"Card({i})" for i in supported)}];']
    # Preserve existing Rust constants and JSON tokens. New callers use English names.
    for index, (name, _, _, _) in enumerate(LEGACY):
        lines.append(f'    pub const {name.replace(" ", "")}: Card = Card({index});')
    lines += ['}', '#[rustfmt::skip]', "pub(super) fn static_modifiers(card: Card) -> &'static [StaticModifier] {", '    match card.0 {', *static_rows, '        _ => &[],', '    }', '}', '#[rustfmt::skip]', 'pub(super) fn equipment(card: Card) -> Option<EquipmentSpec> {', '    match card.0 {', *equipment_rows, '        _ => None,', '    }', '}', '#[rustfmt::skip]', 'pub(super) fn aura(card: Card) -> Option<AuraSpec> {', '    match card.0 {', *aura_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn dynamic_stats(card: Card) -> Option<DynamicStats> {', '    match card.0 {', *dynamic_rows, '        _ => None,', '    }', '}', '#[rustfmt::skip]', "pub(super) fn scaling_modifiers(card: Card) -> &'static [ScalingModifier] {", '    match card.0 {', *scaling_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn keyword_grants(card: Card) -> &'static [KeywordGrant] {", '    match card.0 {', *grant_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn additional_cost(card: Card) -> Option<AdditionalCost> {', '    match card.0 {', *additional_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub const CREATURE_TYPES: &[&str] = &[', *('    '+json.dumps(name)+',' for name in creature_types), '];', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn cost_reductions(card: Card) -> &'static [SpellReduction] {", '    match card.0 {', *reduction_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn cost_alternatives(card: Card) -> &'static [ManaCost] {", '    match card.0 {', *hybrid_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn entry_counter_rules(card: Card) -> &'static [EntryCounterRule] {", '    match card.0 {', *entry_counter_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn starting_loyalty(card: Card) -> u64 {', '    match card.0 {', *loyalty_rows, '        _ => 0,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn exhaust_mask(card: Card) -> u64 {', '    match card.0 {', *exhaust_rows, '        _ => 0,', '    }', '}', '']
    protection = 'matches!(card.0, ' + ' | '.join(uncounterable_rows) + ')' if uncounterable_rows else 'false'
    lines += ['#[rustfmt::skip]', 'pub(super) fn uncounterable(card: Card) -> bool {', '    ' + protection, '}', '']
    return '\n'.join(lines), {'schema': 1, 'catalogue_version': version, 'standard_legal_cards': len(cards),
        'implemented_standard_cards': sum(row['implemented'] and row['standard_legal'] for row in coverage),
        'implemented_total': len(supported), 'token_definitions': len(used_tokens), 'emblem_definitions': len(used_emblems), 'cards': coverage}


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
