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
            'lifelink': 64, 'deathtouch': 128, 'trample': 256, 'first strike': 512, 'double strike': 1024, 'hexproof': 4096, 'indestructible': 8192, 'menace': 32768}
TEXT_FLAGS = {"this creature can't block.": 2048, "this creature can't be blocked.": 16384,
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


def permanent_type(card):
    typ = card.get('type_line', '').split(' — ')[0]
    return typ.removeprefix('Legendary ')


def creature_template(card):
    # Missing/unknown rules text is not a blank text box. A fresh checksum only
    # establishes data integrity; it cannot establish omitted card semantics.
    if not isinstance(card.get('oracle_text'), str):
        return None
    if card.get('layout') != 'normal' or permanent_type(card) not in ('Creature', 'Artifact Creature', 'Enchantment Creature'):
        return None
    if not card.get('power', '').isdigit() or not card.get('toughness', '').isdigit():
        return None
    if fixed_cost(card.get('mana_cost', '')) is None:
        return None
    # Remove only balanced, unnested reminder text; fail closed on residual text.
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    keywords = 0
    body = []
    for line in text.split('\n'):
        if line.casefold() in TEXT_FLAGS: keywords |= TEXT_FLAGS[line.casefold()]
        else: body.append(line)
    words = {word.strip().casefold() for word in ','.join(body).split(',') if word.strip()}
    if not words <= KEYWORDS.keys(): return None
    return f"Creature({int(card['power'])}, {int(card['toughness'])})", keywords | sum(KEYWORDS[word] for word in words)



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
        if ability in ('"This token can\'t block."', '"This token can\'t be blocked."'):
            keywords = 2048 if ability == '"This token can\'t block."' else 16384
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
                   'kithkin': 'Kithkin', 'knights': 'Knight', 'warriors': 'Warrior', 'spirits': 'Spirit'}


def creature_filter(text):
    text = text.casefold()
    if text.startswith('each '): text = text[5:].replace('creature', 'creatures', 1)
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
    subtype, tokens, artifact = 'None', 'None', False
    if text == 'creatures': pass
    elif text == 'creature tokens': tokens = 'Some(true)'
    elif text == 'nontoken creatures': tokens = 'Some(false)'
    elif text == 'artifact creatures': artifact = True
    elif text in SUBTYPE_PLURALS: subtype = 'Some(' + json.dumps(SUBTYPE_PLURALS[text]) + ')'
    else: return None
    return (f'CreatureFilter {{ relation: {relation}, other: {str(other).lower()}, subtype: {subtype}, '
            f'keyword: Keywords({keyword}), attacking: {str(attacking).lower()}, tokens: {tokens}, artifact: {str(artifact).lower()} }}')


def grow_instruction(subject, power, toughness):
    return f'Grow {{ subject: Subject::{subject}, power: {power}, toughness: {toughness} }}'


def static_permanent_template(card):
    if not isinstance(card.get('oracle_text'), str): return None
    body, modifiers = [], []
    for line in re.sub(r'\([^()]*\)', '', card['oracle_text']).strip().split('\n'):
        m = re.fullmatch(r'(.+) get ([+-][0-9]+)/([+-][0-9]+)\.', line)
        if m:
            filt = creature_filter(m[1])
            if filt is None or max(abs(int(m[2])), abs(int(m[3]))) > 32767: return None
            modifiers.append(f'StaticModifier {{ filter: {filt}, stats: Stats {{ power: {int(m[2])}, toughness: {int(m[3])} }} }}')
        else: body.append(line)
    if not modifiers: return None
    proxy = dict(card, oracle_text='\n'.join(body).strip())
    result = permanent_program_template(proxy)
    if result is None:
        template = creature_template(proxy)
        trigger = trigger_creature_template(proxy)
        if template: effect, keywords = template; result = effect, keywords, [], [], [0,0,0]
        elif trigger: effect, keywords, triggers = trigger; result = effect, keywords, triggers, [], [0,0,0]
    return (*result, modifiers) if result else None


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
    plurals={word+'s':word for word in ('creature','artifact','enchantment','land','permanent','planeswalker','token','food','clue','treasure','lander')}
    subject=' '.join(plurals.get(word,word) for word in subject.split())
    kinds={'creature':1,'artifact':16,'enchantment':32,'land':2,'permanent':0,
           'creature or planeswalker':65,'artifact or creature':17,'creature or artifact':17,
           'creature or enchantment':33,'enchantment or creature':33,
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


def statement_program(text):
    """Compile only these complete sentences; residual text never gets ignored."""
    if not isinstance(text, str) or not text:
        return None
    text = normalize_duration(text[0].lower() + text[1:])
    if mill := mill_program(text): return mill
    if choice := optional_payment_program(text, False): return choice
    if sacrifice := sacrifice_program(text):
        if sacrifice[0] == 'None': return sacrifice[1]
    token = token_program(text) or predefined_program(text)
    if token: return token
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
    if text == 'untap this creature.': return ['UntapSource']
    if text == 'tap this creature.': return ['TapSource']
    if m := re.fullmatch(r'untap (.+)\.', text):
        if filt := creature_filter(m[1]): return [f'UntapCreatures({filt})']
    if m := re.fullmatch(r'(?:you )?draw (a card|two cards|three cards|four cards) and (?:you )?lose ([0-9]+) life\.', text):
        if int(m[2])>65535: return None
        return [f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[1]]})',f'LoseLife(PlayerSet::Controller, {int(m[2])})']
    if m := re.fullmatch(r'you lose ([0-9]+) life and (create .+)\.', text):
        if int(m[1])>65535: return None
        if program := token_program(m[2]+'.'): return [f'LoseLife(PlayerSet::Controller, {int(m[1])})', *program]
    if m := re.fullmatch(r'this (?:creature|artifact|enchantment|permanent|spell) deals ([0-9]+) damage to each opponent(?: and you gain ([0-9]+) life)?\.', text):
        if int(m[1]) > 32767 or (m[2] and int(m[2]) > 65535): return None
        return [f'DamagePlayers(PlayerSet::Opponent, {int(m[1])})'] + ([f'GainLife(PlayerSet::Controller, {int(m[2])})'] if m[2] else [])
    if text == "exile each opponent's graveyard.": return ['ExileGraveyard(PlayerSet::Opponent)']
    if text == "exile your graveyard.": return ['ExileGraveyard(PlayerSet::Controller)']
    if text == 'investigate.': return predefined_program('create a Clue token.')
    if mana := mana_program(text): return mana
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
    if m := re.fullmatch(r'this creature deals ([0-9]+) damage to you and you draw (a card|two cards|three cards|four cards)\.', text):
        return [f'DamagePlayers(PlayerSet::Controller, {int(m[1])})',f'Draw(PlayerSet::Controller, {DRAW_COUNTS[m[2]]})'] if int(m[1]) <= 32767 else None
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
    'CreatureVehicle': 'TargetRule::CreatureOrVehicle { relation: -1 }',
    'CreatureVehicleOpponent': 'TargetRule::CreatureOrVehicle { relation: 1 }',
    'Player': 'TargetRule::Player { opponent_only: false }',
    'Opponent': 'TargetRule::Player { opponent_only: true }',
}
for _name,_kinds,_excluded in [('SpellCreature',1,0),('SpellNoncreature',0,1),('SpellInstantSorcery',12,0),('SpellArtifact',16,0),('SpellEnchantment',32,0)]:
    TARGET_RULES[_name]=f'TargetRule::SpellKinds {{ kinds: {_kinds}, excluded: {_excluded} }}'


for _name, _zone, _kinds, _relation, _other, _nonland in [
    ('OtherCreature', 'Battlefield', 1, -1, True, False),
    ('CreatureYou', 'Battlefield', 1, 0, False, False),
    ('OtherCreatureYou', 'Battlefield', 1, 0, True, False),
    ('CreatureOpponent', 'Battlefield', 1, 1, False, False),
    ('Permanent', 'Battlefield', 0, -1, False, False),
    ('Nonland', 'Battlefield', 0, -1, False, True),
    ('OtherNonlandYou', 'Battlefield', 0, 0, True, True),
    ('NonlandOpponent', 'Battlefield', 0, 1, False, True),
    ('CreatureEnchantment', 'Battlefield', 33, -1, False, False),
    ('CardGrave', 'Graveyard', 0, -1, False, False),
    ('Land', 'Battlefield', 2, -1, False, False),
    ('Artifact', 'Battlefield', 16, -1, False, False),
    ('ArtifactCreature', 'Battlefield', 17, -1, False, False),
    ('Enchantment', 'Battlefield', 32, -1, False, False),
    ('ArtifactEnchantment', 'Battlefield', 48, -1, False, False),
    ('CreatureGraveYou', 'Graveyard', 1, 0, False, False),
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


def targeted_program(text):
    """Complete target occurrences with independent predicates and effect focus."""
    if not isinstance(text, str) or not text.endswith('.'):
        return None
    text = normalize_duration(text[0].lower() + text[1:])
    if counter:=counter_program(text): return counter
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
    if m := re.fullmatch(r'(another target creature you control|target creature you control|target creature) (.+)', text):
        rule = {'target creature': 'Creature', 'target creature you control': 'CreatureYou', 'another target creature you control': 'OtherCreatureYou'}[m[1]]
        if program := keyword_effect(m[2], 'Targets'): return rule, program
    if m := re.fullmatch(r'(another target creature you control|target creature you control|target creature) (.+)', text):
        rule = {'target creature': 'Creature', 'target creature you control': 'CreatureYou', 'another target creature you control': 'OtherCreatureYou'}[m[1]]
        if m[2] == "can't be blocked this turn.":
            return rule, ['GrantKeywords { subject: Subject::Targets, keywords: Keywords(16384) }']
        if m[2].endswith(" and can't be blocked this turn."):
            clause = m[2][:-len(" and can't be blocked this turn.")] + '.'
            if program := keyword_effect(clause, 'Targets'):
                return rule, program + ['GrantKeywords { subject: Subject::Targets, keywords: Keywords(16384) }']
    if text == "exile target player's graveyard.": return 'Player', ['ExileGraveyard(PlayerSet::Targeted)']
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
        'untap one or two target creatures. They each get +2/+2 until end of turn.': (('Creature', 1, 2), ['Untap', grow_instruction('Targets', 2, 2)]),
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
    if m := re.fullmatch(r'this (?:creature|artifact|enchantment|permanent|spell) deals ([0-9]+) damage to (any target|target creature|target opponent|target player)\.', text):
        if int(m[1]) > 32767: return None
        return {'any target': 'Any', 'target creature': 'Creature', 'target opponent': 'Opponent', 'target player': 'Player'}[m[2]], [f'Damage({int(m[1])})']
    if m := re.fullmatch(r'(target creature|target creature you control|another target creature you control) gets ([+-][0-9]+)/([+-][0-9]+) until end of turn\.', text):
        if max(abs(int(m[2])), abs(int(m[3]))) > 32767: return None
        rule = {'target creature': 'Creature', 'target creature you control': 'CreatureYou', 'another target creature you control': 'OtherCreatureYou'}[m[1]]
        return rule, [grow_instruction('Targets', int(m[2]), int(m[3]))]
    # A targeted clause followed by wholly non-targeted instructions is supported.
    if '. ' in text:
        first, rest = text.split('. ', 1)
        result = targeted_program(first + '.')
        tail = statement_program(rest)
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
    if text == 'you attacked this turn': return 'StepGuard::Attacked'
    if text == 'this creature is tapped': return 'StepGuard::SourceTapped'
    def threshold(value, minimum=None, maximum=None):
        lo = 'None' if minimum is None else f'Some({minimum})'
        hi = 'None' if maximum is None else f'Some({maximum})'
        return f'StepGuard::Threshold {{ value: StepValue::{value}, minimum: {lo}, maximum: {hi} }}'
    if text == "you didn't cast a spell this turn": return threshold('SpellsCast', maximum=0)
    if m := re.fullmatch(r'you control a creature with power ([0-9]+) or greater', text):
        return threshold('GreatestCreaturePower', minimum=int(m[1])) if int(m[1]) <= 32767 else None
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
        'Whenever this creature attacks, ': 'SelfAttacks',
        'Whenever this creature enters or attacks, ': 'SelfEntersOrAttacks',
        'Whenever another artifact you control enters, ': 'OtherArtifactEntersYou',
        'Whenever another creature you control enters, ': 'OtherCreatureEntersYou',
        'Whenever this creature or another creature you control enters, ': 'CreatureEntersYou',
        'Landfall — Whenever a land you control enters, ': 'LandEntersYou',
        **STEP_PREFIXES,
        **VITAL_PREFIXES,
    }
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    for line in text.split('\n'):
        line = line.strip()
        name = card.get('name', '')
        names = [name] if name else []
        if name and card.get('type_line', '').startswith('Legendary '):
            names.append(name.split(',')[0])
        for alias in names:
            if line.startswith('When ' + alias + ' enters, '):
                line = 'When this creature enters, ' + line[len('When ' + alias + ' enters, '):]
        for label in ('Raid — ', 'Formidable — ', 'Survival — '):
            if line.startswith(label + 'At the beginning of '): line = line[len(label):]
        for alias in names:
            line=re.sub(r'^Whenever '+re.escape(alias)+r' deals? ((?:combat )?damage to )',r'Whenever this creature deals \1',line)
            line=line.replace('Whenever '+alias+' enters or attacks, ','Whenever this creature enters or attacks, ')
        if line.casefold() in TEXT_FLAGS:
            keywords |= TEXT_FLAGS[line.casefold()]; continue
        words = [word.strip().casefold() for word in line.split(',') if word.strip()]
        if words and all(word in KEYWORDS or word == 'prowess' or ward_word(word) is not None for word in words):
            keywords |= sum(KEYWORDS[word] for word in set(words) if word in KEYWORDS)
            for word in words:
                if word=='prowess': abilities.append((cast_condition(filter_='Noncreature'),[grow_instruction('Source',1,1)],'None'))
                elif (cost:=ward_word(word)) is not None: abilities.append(('Ward',[counter_unless(cost,'Related')],'None'))
            continue
        damage_prefix=player_damage_prefix(line)
        line_prefixes=[damage_prefix] if damage_prefix else []
        for prefix, condition in [*line_prefixes,*prefixes.items()]:
            if line.startswith(prefix):
                effect_text = line[len(prefix):]
                if condition.startswith('AtStep {') and effect_text.startswith('if '):
                    parts = effect_text[3:].split(', ', 1)
                    if len(parts) != 2 or (guard := step_guard(parts[0])) is None: return None
                    condition = condition.replace('StepGuard::Always', guard)
                    effect_text = parts[1]
                # Only self-event subjects prove an initial 'it' denotes this source.
                # Other-creature events and later targeted clauses have other antecedents.
                if condition in ('SelfEnters', 'SelfDies', 'SelfEntersOrDies', 'SelfAttacks', 'SelfEntersOrAttacks', 'SelfDiscarded', 'SelfCycled'):
                    effect_text = re.sub(r'^[Ii]t deals ', 'this creature deals ', effect_text)
                for alias in names:
                    if effect_text.startswith(alias + ' deals '):
                        effect_text = 'this creature deals ' + effect_text[len(alias) + len(' deals '):]
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
    if isinstance(text, str): text = ' '.join(re.sub(r'\([^()]*\)', '', text).split())
    if isinstance(text, str) and text.startswith(card.get('name', '') + ' deals '):
        text = 'this spell deals ' + text[len(card['name']) + len(' deals '):]
    targeted = targeted_program(text)
    if targeted: return targeted
    program = statement_program(text)
    return ('None', program) if program else None


def modal_spell_template(card):
    """Every mode must compile completely; unsupported alternatives gate the card."""
    if (card.get('layout') != 'normal' or card.get('type_line') not in ('Instant', 'Sorcery')
            or fixed_cost(card.get('mana_cost', '')) is None
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
    cost = dict(colored=[0] * 6, generic=0, mana_component=False, tap=False, sacrifice=False, life=0, remove=None)
    seen = set()
    for component in text.split(', '):
        mana = fixed_cost(component)
        if mana:
            if 'mana' in seen: return None
            seen.add('mana')
            cost['mana_component'] = True
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


def aura_template(card):
    if (card.get('layout') != 'normal' or permanent_type(card) != 'Enchantment'
            or card.get('type_line','').split(' — ')[-1] != 'Aura'
            or not isinstance(card.get('oracle_text'),str)
            or fixed_cost(card.get('mana_cost','')) is None): return None
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
            or fixed_cost(card.get('mana_cost','')) is None): return None
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


def permanent_program_template(card):
    """Whole supported permanent text, including all activation restrictions."""
    if (card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str)
            or fixed_cost(card.get('mana_cost', '')) is None):
        return None
    typ = permanent_type(card)
    if set(card.get('type_line','').split(' — ')[-1].split()) & {'Equipment','Aura'}: return None
    if typ not in ('Creature', 'Artifact Creature', 'Enchantment Creature', 'Artifact', 'Enchantment'):
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
        # All admitted costs refer only to the source, mana, life or its counters;
        # an initial pronoun in this activation therefore denotes the source.
        effect_text = re.sub(r'^[Ii]t deals ', 'this permanent deals ', effect_text)
        targeted = targeted_program(effect_text)
        if targeted: targets, program = targeted
        else: targets, program = 'None', statement_program(effect_text)
        if program is None: return None
        if any(instruction.startswith('AddMana') for instruction in program) and (len(program) != 1 or targets != 'None'):
            return None
        if program[0].startswith('AddMana') and not (cost['tap'] or cost['sacrifice'] or cost['life'] or cost['remove'] or once):
            # Unbounded positive production needs an infinite-resource solver.
            # Resource-free conversions that do not increase total mana are finite.
            fixed = re.search(r'Fixed\((\[[^]]*\])\)', program[0])
            produced = sum(json.loads(fixed[1])) if fixed else int(re.search(r'amount: ([0-9]+)', program[0])[1])
            if produced > sum(cost['colored']) + cost['generic']: return None
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


# These are replacements evaluated before entry, not triggered abilities.
LAND_ENTRY = {
    'This land enters tapped.': 'Always',
    'This land enters tapped unless you control a basic land.': 'UnlessBasic',
    'This land enters tapped unless you control two or fewer other lands.': 'UnlessAtMostTwo',
    'This land enters tapped unless you control two or more other lands.': 'UnlessAtLeastTwo',
    'This land enters tapped unless a player has 13 or less life.': 'UnlessLowLife',
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
        if line in LAND_ENTRY:
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
        rows.append(f"ActivatedAbility {{ cycling: {str(cost.get('cycling',False)).lower()}, origin: ActivationOrigin::{cost.get('origin','Battlefield')}, cost: ActivationCost {{ colored: {cost['colored']}, generic: {cost['generic']}, mana_component: {str(cost['mana_component']).lower()}, tap: {str(cost['tap']).lower()}, sacrifice: {str(cost['sacrifice']).lower()}, discard: {str(cost.get('discard',False)).lower()}, life: {cost['life']}, remove: {remove} }}, targets: {rust_targets(targets)}, program: {rust_program(program)}, sorcery: {str(sorcery).lower()}, once_per_turn: {str(once).lower()} }}")
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
    static_rows = []
    equipment_rows = []
    aura_rows = []
    for index, name in enumerate(names):
        card, cycling = cycling_abilities(cards.get(name, {}))
        cost = card.get('mana_cost', '')
        keywords = sum(KEYWORDS[word] for word in {keyword.casefold() for keyword in card.get('keywords', [])} & KEYWORDS.keys())
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
            kind = (1 if 'Creature' in types else 0) | (2 if 'Land' in types else 0) | (4 if 'Sorcery' in types else 0) | (8 if 'Instant' in types else 0) | (16 if 'Artifact' in types else 0) | (32 if 'Enchantment' in types else 0) | (64 if 'Planeswalker' in types else 0)
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
                effect, keywords, abilities, activations, enters, modifiers = static
                static_rows.append(f'        {index} => &[{", ".join(modifiers)}],')
        if effect != 'Unsupported':
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

        if effect != 'Unsupported':
            supported.append(index)
        basic = name in {'Plains', 'Island', 'Swamp', 'Mountain', 'Forest'} or card.get('type_line', '').split(' — ')[0] == 'Basic Land'
        colored, generic = fixed_cost(cost) or ([0] * 6, 0)
        deck_limit = 'None' if basic or name in UNLIMITED else 'Some(4)'
        quote = lambda value: json.dumps(value, ensure_ascii=False)
        oracle_id = f"Some({quote(card['oracle_id'])})" if card else 'None'
        type_line = card.get('type_line', 'Basic Land' if basic else 'Creature' if kind & 1 else 'Instant' if kind & 8 else 'Sorcery' if kind & 4 else '')
        oracle_keywords = '&[' + ', '.join(quote(word) for word in card.get('keywords', [])) + ']'
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {quote(name)}, oracle_id: {oracle_id}, mana_cost: {quote(cost)}, type_line: {quote(type_line)}, oracle_keywords: {oracle_keywords}, standard_legal: {str(name in cards).lower()}, legendary: {str(card.get('type_line', '').startswith('Legendary ')).lower()}, implemented: {str(effect != 'Unsupported').lower()}, keywords: Keywords({keywords}), deck_limit: {deck_limit} }}, effect: Effect::{effect}, triggers: {triggers_rust}, activated: {rust_activations(activations)}, enters: Counters {{ plus_one: {enters[0]}, minus_one: {enters[1]}, stun: {enters[2]} }}, entry: EntryTapped::{entry}, kind: {kind}, basic: {str(basic).lower()}, mana_color: {'None' if color is None else f'Some({color})'}, colored: {colored}, generic: {generic} }},")
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
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {json.dumps(name)}, oracle_id: None, mana_cost: \"\", type_line: {json.dumps(type_line, ensure_ascii=False)}, oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: Keywords({factory['keywords']}), deck_limit: None }}, effect: Effect::{effect}, triggers: &[], activated: {rust_activations(token_abilities)}, enters: Counters {{ plus_one: 0, minus_one: 0, stun: 0 }}, entry: EntryTapped::Never, kind: {kind}, basic: false, mana_color: None, colored: [0; 6], generic: 0 }},")
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
    lines += ['}', '#[rustfmt::skip]', "pub(super) fn static_modifiers(card: Card) -> &'static [StaticModifier] {", '    match card.0 {', *static_rows, '        _ => &[],', '    }', '}', '#[rustfmt::skip]', 'pub(super) fn equipment(card: Card) -> Option<EquipmentSpec> {', '    match card.0 {', *equipment_rows, '        _ => None,', '    }', '}', '#[rustfmt::skip]', 'pub(super) fn aura(card: Card) -> Option<AuraSpec> {', '    match card.0 {', *aura_rows, '        _ => None,', '    }', '}', '']
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
