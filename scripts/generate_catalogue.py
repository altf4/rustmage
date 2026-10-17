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

def extract_target_cost_reduction(card):
    if card.get('layout')!='normal' or not isinstance(card.get('oracle_text'),str):return card,None
    def obj(kinds=0,relation=-1,tapped=False,min_mana=0,max_mana=4294967295):
        return f'TargetRule::Object {{ zone: TargetZone::Battlefield, kinds: {kinds}, relation: {relation}, other: false, nonland: false, tapped: {str(tapped).lower()}, min_mana: {min_mana}, max_mana: {max_mana} }}'
    def filtered(base,predicate):return 'TargetRule::Filtered { base: &'+base+', predicate: &TargetPredicate::'+predicate+' }'
    attacking=filtered('TargetRule::Creature','Attacking')
    rules={
        'a tapped creature':obj(1,tapped=True),'a tapped permanent':obj(tapped=True),
        'a blocking creature':filtered('TargetRule::Creature','Blocking'),
        'an attacking creature':attacking,
        'an attacking or tapped creature':'TargetRule::Union(&['+attacking+', '+obj(1,tapped=True)+'])',
        'an attacking nontoken creature':filtered(attacking,'Nontoken'),
        'a Spider':filtered(obj(),'Subtype("Spider")'),
        'a creature with flying':filtered('TargetRule::Creature','Keyword(Keywords(1))'),
        'a permanent with mana value 1':obj(min_mana=1,max_mana=1),
        'a Mount or Vehicle you control':'TargetRule::Union(&['+', '.join(filtered(obj(relation=0),'Subtype('+json.dumps(t)+')') for t in ['Mount','Vehicle'])+'])',
        'a creature spell':'TargetRule::SpellKinds { kinds: 1, excluded: 0 }',
        'an instant or sorcery spell':'TargetRule::SpellKinds { kinds: 12, excluded: 0 }',
    }
    body=[];found=[]
    for line in card['oracle_text'].split('\n'):
        m=re.fullmatch(r'This spell costs ((?:\{(?:[0-9]+|[WUBRGC])\})+) less to cast if it targets (.+)\.',line)
        cost=fixed_cost(m[1]) if m else None;rule=rules.get(m[2]) if m else None
        if cost and rule:found.append('TargetSpellReduction { rule: '+rule+', discount: ManaCost { colored: '+str(cost[0])+', generic: '+str(cost[1])+' } }')
        else:body.append(line)
    if len(found)!=1:return card,None
    return dict(card,oracle_text='\n'.join(body)),found[0]


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

def extract_cast_timing_grants(card):
    """Exact top-level permission; every other sentence stays for whole-body parsing."""
    if card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str):
        return card, []
    # This metadata describes a static ability of a permanent on the battlefield.
    # A spell instruction needs a duration-bearing resolution program instead.
    kinds = card.get('type_line', '').split(' — ')[0].split()
    if not any(kind in kinds for kind in ('Creature', 'Artifact', 'Enchantment', 'Land', 'Planeswalker', 'Battle')):
        return card, []
    # Do not expose the inherited broad parenthetical stripping to a new route.
    # Audited two-face wrappers remove only authenticated reminders beforehand.
    if '(' in card['oracle_text'] or ')' in card['oracle_text']:
        return card, []
    residual, grants = [], []
    for original in card['oracle_text'].splitlines():
        line = original.strip()
        match = re.fullmatch(r'You may cast (.+) spells as though they had flash\.', line)
        filters = None
        if match:
            nouns = match[1].split(' spells and ')
            if 1 <= len(nouns) <= 2:
                filters = [('CastFilter::Sorcery' if noun == 'sorcery' else reduction_filter(noun)) for noun in nouns]
                if not all(filters): filters = None
        if filters:
            grants.extend(filters)
        else:
            residual.append(original)
    return dict(card, oracle_text='\n'.join(residual)), grants


def extract_cost_reductions(card):
    text=card.get('oracle_text')
    if not isinstance(text,str) or card.get('layout')!='normal':return card,[]
    body,rules=[],[]
    for original in text.split('\n'):
        line=original.strip();turn='Any';guard='StepGuard::Always';source_only=False;filt=None;amount=None;quantity=None;derived=None
        for prefix,timing in [('During turns other than yours, ','Other'),('During your turn, ','Own')]:
            if line.startswith(prefix):turn=timing;line=line[len(prefix):];break
        if m:=re.fullmatch(r'Permanent spells you cast that have an Adventure cost \{([0-9]+)\} less to cast\.',line):
            filt='CastFilter::PermanentAdventure';amount=int(m[1])
        elif m:=re.fullmatch(r'([A-Za-z -]+) spells you cast cost \{X\} less to cast, where X is equipped creature[’\']s power\.',line):
            filt=reduction_filter(m[1]);amount=1;derived='SpellReductionAmount::EquippedPower'
        elif m:=re.fullmatch(r'Affinity for ([A-Za-z -]+)(?: \(This spell costs \{1\} less to cast for each (.+) you control\.\))?',line):
            noun=m[1]
            candidate=reduction_quantity(noun.casefold()+' you control')
            if m[2] is None or candidate==reduction_quantity(m[2].casefold()+' you control'):
                quantity=candidate;amount=1;source_only=True;filt='CastFilter::Any'
        elif m:=re.fullmatch(r'(?:Vivid — )?This spell costs \{([0-9]+)\} less to cast for each (.+)\.',line):
            quantity=reduction_quantity(m[2]);amount=int(m[1]);source_only=True;filt='CastFilter::Any'
            if m[2] in ('attacking creature','attacking creatures'):quantity='StatValue::AllAttackingCreatures'
            if '{X}' in card.get('mana_cost','') and amount==1 and m[2]=='creature you control with power 4 or greater':derived='SpellReductionAmount::CreaturePowerCount { minimum: 4 }'
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



def intrinsic_reminder_line_valid(line):
    """A complete keyword line proves its own trailing reminder, on any host."""
    grammar = {
        'changeling': r'This card is every creature type\.',
        'deathtouch': r'Any amount of damage this deals to a creature is enough to destroy it\.',
        'defender': r"This creature can't attack\.",
        'double strike': r'This creature deals both first-strike and regular combat damage\.',
        'first strike': r'This creature deals combat damage before creatures without first strike\.',
        'flash': r'You may cast this spell any time you could cast an instant\.',
        'flying': r"This creature can't be blocked except by creatures with flying or reach\.",
        'haste': r'This creature can attack and \{T\} as soon as it comes under your control\.',
        'hexproof': r"It can't be the target of spells or abilities your opponents control\.",
        'indestructible': r'''Damage and effects that say "destroy" don't destroy it\.(?: If its toughness is 0 or less, it still dies\.)?''',
        'lifelink': r'Damage dealt by this creature also causes you to gain that much life\.',
        'menace': r"This creature can't be blocked except by two or more creatures\.",
        'reach': r'This creature can block creatures with flying\.',
        'skulk': r"This creature can't be blocked by creatures with greater power\.",
        'trample': r"This creature can deal excess combat damage to the player or planeswalker it's attacking\.",
        'vigilance': r"Attacking doesn't cause this creature to tap\.",
    }
    match = re.fullmatch(r'([^()]+) \(([^()]+)\)', line)
    if match is None:
        return False
    words = [word.strip().casefold() for word in match[1].split(',')]
    return (bool(words) and all(word in INTRINSIC_KEYWORDS for word in words)
            and words[-1] in grammar
            and re.fullmatch(grammar[words[-1]], match[2]) is not None)


def intrinsic_reminders_valid(card):
    """Authenticate complete intrinsic bodies before any generic text extraction.

    Erased text detects the semantic route only. It never becomes executable
    text: the original body must prove every reminder in its own grammar slot.
    Other whole-body families have separate reminder audits.
    """
    raw = card.get('oracle_text')
    if not isinstance(raw, str) or not any(c in raw for c in '()'):
        return True
    bare = raw
    while True:
        stripped = re.sub(r'\([^()]*\)', '', bare)
        if stripped == bare:
            break
        bare = stripped
    if creature_template(dict(card, oracle_text=bare)) is not None:
        return all(intrinsic_reminder_line_valid(line) for line in raw.splitlines()
                   if any(c in line for c in '()'))
    if (card.get('layout') == 'normal' and card.get('mana_cost', '') == ''
            and permanent_type(card) in ('Land', 'Basic Land') and not bare.strip()):
        # The complete intrinsic land grammar includes its actual basic subtypes
        # and matching mana production. No other reminder is discarded.
        return (re.fullmatch(r'\(\{T\}: Add \{[WUBRG]\}(?: or \{[WUBRG]\})?\.\)', raw) is not None
                and land_template(card) is not None)
    return True


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


ROLE_BODIES = {
    'Cursed': 'Enchanted creature is 1/1.',
    'Monster': 'Enchanted creature gets +1/+1 and has trample.',
    'Royal': 'Enchanted creature gets +1/+1 and has ward {1}.',
    'Sorcerer': 'Enchanted creature gets +1/+1 and has "Whenever this creature attacks, scry 1."',
    'Wicked': 'Enchanted creature gets +1/+1. When this Aura is put into a graveyard from the battlefield, each opponent loses 1 life.',
    'Young Hero': 'Enchanted creature has "Whenever this creature attacks, if its toughness is 3 or less, put a +1/+1 counter on it."',
}


def role_route(text):
    """Detection can erase prose; production authentication always uses raw text."""
    while isinstance(text,str):
        if re.search(r'\bRole\s+token\b',text,re.IGNORECASE):return True
        stripped=re.sub(r'\([^()]*\)', '', text)
        if stripped==text:return False
        text=stripped
    return False


def role_reminder_text(text):
    """Authenticate complete predefined characteristics before presentation removal."""
    for kind, body in ROLE_BODIES.items():
        prefix = 'If you control another Role on it, put that one into the graveyard. '
        # These two printed presentations describe the same predefined Role.
        bodies = [body, prefix + body]
        if kind == 'Wicked':
            bodies += [v.replace('When this Aura is put into a graveyard from the battlefield', 'When this token is put into a graveyard') for v in bodies]
            bodies += [v.replace('this Aura', 'this enchantment') for v in bodies]
        for accepted in bodies:
            pattern = r'(' + re.escape(kind) + r' Role token attached to [^.]+\.)( Activate only as a sorcery\.)? \(' + re.escape(accepted) + r'\)'
            text = re.sub(pattern, r'\1\2', text)
    return text


def role_consumer_reminders_valid(card):
    """Authenticate raw context for every production opened by Role consumers.

    Detection follows parenthetical erasure/whitespace stages only to identify
    grammar use. Execution still requires the original raw body: an unrelated
    recognized reminder cannot authenticate an instruction's own parentheses.
    """
    raw = card.get('oracle_text')
    if not isinstance(raw, str) or not any(c in raw for c in '()'):
        return True
    text = raw
    seen = role_route(raw)
    productions = (
        r'destroy target creature an opponent controls with mana value [0-9]+ or less\.',
        r'destroy target creature or enchantment\.',
        r'\bif it was bargained,',
    )
    while True:
        normalized = ' '.join(text.casefold().split())
        seen = seen or any(re.search(pattern, normalized) for pattern in productions)
        stripped = re.sub(r'\([^()]*\)', '', text)
        if stripped == text:
            break
        text = stripped
    if not seen:
        return True
    if card.get('layout') == 'saga':
        # This grammar already authenticates its header, all chapters, exact
        # Food/Role reminders and the matching final sacrifice number.
        return saga_body(card) is not None
    authenticated = role_reminder_text(raw)
    authenticated = re.sub(
        r'^Bargain \(You may sacrifice an artifact, enchantment, or token as you cast this spell\.\)$',
        'Bargain', authenticated, flags=re.MULTILINE)
    return all(intrinsic_reminder_line_valid(line)
               for line in authenticated.splitlines() if any(c in line for c in '()'))


def enchanted_draw_reminders_valid(card):
    """New generic guard/draw routes authenticate raw input before erasure."""
    raw=card.get('oracle_text','')
    if not isinstance(raw,str):return True
    surface=raw;seen=False
    while True:
        normalized=' '.join(surface.casefold().split())
        seen |= re.search(r'\byou control an enchanted creature\b|\byou lose [0-9]+ life and you draw an additional card\b',normalized) is not None
        stripped=re.sub(r'\([^()]*\)','',surface)
        if stripped==surface:break
        surface=stripped
    if not seen:return True
    for line in role_reminder_text(raw).splitlines():
        if '(' not in line and ')' not in line:continue
        bare=re.sub(r'\([^()]*\)','',line).strip()
        proxy=dict(card,oracle_text=bare)
        if creature_template(proxy) is None or not intrinsic_reminders_valid(dict(card,oracle_text=line)):
            return False
    return True


def role_program(text):
    if not isinstance(text,str): return None
    body = text
    before = []
    fixed_rule = None
    if body.startswith('tap target creature. '):
        body = body[len('tap target creature. '):]; before = ['Tap']; fixed_rule = 'Creature'
    elif body.startswith('target creature you control gains hexproof until end of turn. '):
        body = body[len('target creature you control gains hexproof until end of turn. '):]
        before = [f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({KEYWORDS["hexproof"]}) }}']; fixed_rule = 'CreatureYou'
    elif body.startswith('draw a card. '):
        body = body[len('draw a card. '):]; before = ['Draw(PlayerSet::Controller, 1)']
    if body.startswith('one or two target creatures each gain haste until end of turn. For each of those creatures, '):
        tail=body[len('one or two target creatures each gain haste until end of turn. For each of those creatures, '):]
        result=role_program(tail.replace('attached to it.','attached to target creature.'))
        if result is not None and result[0]=='Creature':
            return ('Creature',1,2),[f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({KEYWORDS["haste"]}) }}',*result[1]]
        return None
    body = body[0].lower() + body[1:] if body else body
    m = re.fullmatch(r'create a (Cursed|Monster|Royal|Sorcerer|Wicked|Young Hero) Role token attached to (target creature(?: you control| an opponent controls)?|another target creature(?: you control)?|up to one (?:other )?target creature you control|it|that creature)\.', body)
    if not m:return None
    kind, recipient = m.groups()
    rules = {'target creature':'Creature','target creature you control':'CreatureYou','target creature an opponent controls':'CreatureOpponent','another target creature':'OtherCreature','another target creature you control':'OtherCreatureYou','up to one other target creature you control':('OtherCreatureYou',0,1),'up to one target creature you control':('CreatureYou',0,1)}
    rule = rules.get(recipient)
    if rule is None:
        if fixed_rule is None:return None
        rule = fixed_rule
    elif fixed_rule is not None and rule != fixed_rule:return None
    factory = {'role':kind, 'name':kind, 'subtypes':'Aura Role', 'colors':0, 'keywords':0}
    key = hashlib.sha256(json.dumps(factory,sort_keys=True).encode()).hexdigest()[:24]
    if key in TOKEN_DEFS and TOKEN_DEFS[key] != factory:raise ValueError('token factory collision')
    TOKEN_DEFS[key] = factory
    return rule, [*before, f'CreateRole {{ token: __TOKEN_{key}__, subject: Subject::Targets }}']



def source_role_program(text, condition):
    """Only a complete self-event clause proves the recipient is this source."""
    if condition not in ('SelfEnters','SelfAttacks','SelfEntersOrAttacks'):return None
    pay=None
    if match:=re.fullmatch(r'you may pay ((?:\{[0-9WUBRGC]+\})+)\. If you do, (.+)',text):
        pay=mana_cost_spec(match[1])
        if pay is None or spell_cost_options(match[1]) is None:return None
        text=match[2]
    match=re.fullmatch(r'[Cc]reate a (Cursed|Monster|Royal|Sorcerer|Wicked|Young Hero) Role token attached to (?:it|this creature)\.',text)
    if match is None:return None
    result=role_program('create a '+match[1]+' Role token attached to target creature.')
    assert result is not None
    program=[v.replace('Subject::Targets','Subject::Source') for v in result[1]]
    return 'None',([f'MayPayMana {{ cost: {pay}, skip: {len(program)} }}'] if pay else [])+program


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
    landfall_growth=None
    if suffix != '.':
        if not suffix.startswith(' with '): return None
        ability = suffix[len(' with '):]
        if ability in ('"This token can\'t block."', '"This token can\'t be blocked."'):
            keywords = 2048 if ability == '"This token can\'t block."' else 16384
        elif ability=='"Whenever a land you control enters, this token gets +1/+0 until end of turn."':
            landfall_growth=[1,0]
        else:
            if not ability.endswith('.'): return None
            words = [word.strip() for word in ability[:-1].replace(' and ', ',').split(',')]
            if not words or not all(word in INTRINSIC_KEYWORDS for word in words): return None
            keywords = sum(INTRINSIC_KEYWORDS[word] for word in set(words))
    factory = {'name': name, 'subtypes': subtypes, 'colors': sum(colors[word] for word in color_names),
               'power': int(power), 'toughness': int(toughness), 'extra_type': extra_type,
               'keywords': keywords}
    if landfall_growth is not None:factory['landfall_growth']=landfall_growth
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
    'Map': '{1}, {T}, Sacrifice this artifact: Target creature you control explores. Activate only as a sorcery.',
    'Lander': '{2}, {T}, Sacrifice this artifact: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.',
}

def predefined_program(text):
    m = re.fullmatch(r'create (a|an|one|two|three|four|five|six|seven|eight) (tapped )?(Treasure|Gold|Food|Clue|Mutagen|Heartwood|Lander|Map) tokens?\.', text)
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
        singular={word.casefold():word for word in [*CREATURE_TYPE_NAMES,'Plains','Island','Swamp','Mountain','Forest','Equipment','Desert','Gate','Cave','Town','Treasure','Lesson','Food']}
        plurals={word.casefold()+'s':word for word in ['Island','Swamp','Mountain','Forest','Desert','Gate','Cave','Town','Treasure','Food']}
        subtype=SUBTYPE_PLURALS.get(noun,plurals.get(noun,singular.get(noun)))
        if subtype is None:return None
        if basic and subtype not in ('Plains','Island','Swamp','Mountain','Forest'):return None
    return f'CountFilter {{ zone: CountZone::{zone}, relation: 0, kind: {kind}, exclude: {exclude}, subtype: {"None" if subtype is None else "Some("+json.dumps(subtype)+")"}, basic: {str(basic).lower()}, other: {str(other).lower()}, cards_only: {str(cards).lower()} }}'


def stat_value(text):
    if text in ('attacking creature','attacking creatures'):return 'StatValue::AttackingCreatures'
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
        if line == 'Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it.':
            rules.append('EntryCounterRule { guard: StepGuard::Always, amount: EntryCounterAmount::ColorsSpent }')
        elif line == "This creature enters with two +1/+1 counters on it if it wasn't cast or no mana was spent to cast it.":
            rules.append('EntryCounterRule { guard: StepGuard::Always, amount: EntryCounterAmount::IfNotCastOrZero(2) }')
        elif line == 'Raid — This creature enters with a +1/+1 counter on it if you attacked this turn.':
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
    if text=='two or more nonland permanents entered the battlefield under your control this turn':return 'StatValue::AtLeast { value: &StatValue::NonlandEntriesThisTurn, minimum: 2 }'
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
            parsed='StatValue::ControllerHandSize' if value=='card in your hand' else stat_value(value)
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


def extract_counter_grants(card):
    if card.get('layout')!='normal' or not isinstance(card.get('oracle_text'),str):return card,[]
    body,grants=[],[]
    for line in card['oracle_text'].splitlines():
        m=re.fullmatch(r'Each creature you control with a \+1/\+1 counter on it has (.+)\.',line)
        if m and (keywords:=keyword_list(m[1].casefold())) is not None:
            grants.append(f'(CounterKind::PlusOne, Keywords({keywords}))')
        else:body.append(line)
    return dict(card,oracle_text='\n'.join(body)),grants


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
            filt = creature_filter(m[1])
            if filt is None or max(abs(int(m[2])), abs(int(m[3]))) > 32767: return None
            if m[4]:
                extra=grant(m[1],m[4],own_turn,requires_life_gain)
                if extra is None: return None
                grants.append(extra)
            modifiers.append(f'StaticModifier {{ own_turn: {str(own_turn).lower()}, requires_life_gain: {str(requires_life_gain).lower()}, filter: {filt}, stats: Stats {{ power: {int(m[2])}, toughness: {int(m[3])} }} }}')
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


RULE_LABELS = ('Celebration — ', 'Infusion — ', 'Delirium — ', 'Threshold — ', 'Descend 4 — ', 'Descend 8 — ', 'Ferocious — ', 'Alliance — ')

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


def search_filter(noun):
    """Closed card qualities, with explicit conjunctions and branch-local unions."""
    fixed={
        'basic land card':'BasicLand','land card':'Land','creature card':'Creature',
        'creature or basic land card':'AnyOf(&[SearchFilter::Creature, SearchFilter::BasicLand])',
        'artifact card':'Artifact','enchantment card':'Enchantment',
        'basic land cards and/or Gate cards':'AnyOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Gate")])',
        'basic land cards and/or Desert cards':'AnyOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Desert")])',
        'basic land card or a Desert card':'AnyOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Desert")])',
        'instant card or a card with flash':'AnyOf(&[SearchFilter::Kinds(8), SearchFilter::Keyword(Keywords(32))])',
        'basic Forest card':'AllOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Forest")])',
        'Spider Hero card':'AllOf(&[SearchFilter::Subtype("Spider"), SearchFilter::Subtype("Hero")])',
        'Mount or Vehicle card':'AnyOf(&[SearchFilter::Subtype("Mount"), SearchFilter::Subtype("Vehicle")])',
        'Equipment card':'Subtype("Equipment")','Mountain card':'Subtype("Mountain")',
        'card named Dragonstorm Globe or Boulderborn Dragon':'AnyOf(&[SearchFilter::Name("Dragonstorm Globe"), SearchFilter::Name("Boulderborn Dragon")])',
        'card named Magitek Infantry':'Name("Magitek Infantry")',
    }
    if noun in fixed:return fixed[noun]
    if noun in ('basic land cards','land cards','creature cards','artifact cards','enchantment cards','basic Forest cards'):
        return fixed[noun[:-1]]
    return None


def split_search_route_text(text, *, include_other_search=False):
    """Detect the new route through legacy reminder erasure; never rewrite its body."""
    if not isinstance(text, str):return False
    while True:
        normalized=re.sub(r'\s+', ' ', text).casefold()
        if ('put one onto the battlefield tapped and the other into your hand' in normalized
                or include_other_search and 'search your library' in normalized):
            return True
        reduced=re.sub(r'\([^()]*\)', '', text)
        if reduced==text:return False
        text=reduced


def search_program(text):
    if m:=re.fullmatch(r'search your library for up to two (.+?), reveal them, put one onto the battlefield tapped and the other into your hand, then shuffle\.',text):
        noun=m[1];filter_=search_filter(noun)
        if not noun.endswith('cards') or filter_ is None:return None
        return [f'Search(SearchSpec {{ filter: SearchFilter::{filter_}, maximum: 2, destination: SearchDestination::BattlefieldThenHand {{ battlefield: 1 }}, tapped: true, reveal: true }})']
    match=re.fullmatch(r'search your library for (a |an |up to two |up to three |up to four )(.+?), (reveal (?:it|them), )?put (it|them|that card) (into your hand|onto the battlefield)( tapped)?, then shuffle\.',text)
    if not match:return None
    count,noun,reveal,pronoun,destination,tapped=match.groups()
    filter_=search_filter(noun)
    if filter_ is None:return None
    plural=count.startswith('up to ')
    if plural != noun.endswith('cards') or plural != (pronoun=='them'):return None
    if count not in ('a ','an ','up to two ','up to three ','up to four '):return None
    if reveal and plural != ('them' in reveal):return None
    count={'a ':1,'an ':1,'up to two ':2,'up to three ':3,'up to four ':4}[count]
    if destination=='into your hand' and (tapped or not reveal):return None
    destination='Hand' if destination=='into your hand' else 'Battlefield'
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
    if subject=='frog':subtype,subject='Some("Frog")','permanent'
    if subject not in kinds: return None
    filt=(f'SacrificeFilter {{ artifact_enchantment_or_token: false, keywords: Keywords({keyword}), source_only: {str(source_only).lower()}, kinds: {kinds[subject]}, nonland: {str(nonland).lower()}, '
          f'other: {str(other).lower()}, tokens: {tokens}, subtype: {subtype} }}')
    return amount,filt


def extract_convoke(card):
    lines=card.get('oracle_text','').splitlines()
    declaration="Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)"
    if lines.count(declaration)!=1 or card.get('layout')!='normal':return card,False
    return dict(card,oracle_text='\n'.join(line for line in lines if line!=declaration)),True


def extract_plot(card):
    if card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str): return card,None
    lines=card['oracle_text'].splitlines()
    declarations=[line for line in lines if line.startswith('Plot ')]
    if len(declarations)!=1: return card,None
    m=re.fullmatch(r'Plot ((?:\{(?:[0-9]+|[WUBRGC])\})+) \(You may pay \1 and exile this card from your hand\. Cast it as a sorcery on a later turn without paying its mana cost\. Plot only as a sorcery\.\)', declarations[0])
    if not m or fixed_cost(m[1]) is None or spell_cost_options(m[1]) is None: return card,None
    colored,generic=fixed_cost(m[1])
    return dict(card,oracle_text='\n'.join(line for line in lines if line!=declarations[0])), f'ManaCost {{ colored: {colored}, generic: {generic} }}'


def extract_warp(card):
    if card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str): return card,None
    lines=card['oracle_text'].splitlines()
    declarations=[line for line in lines if line.startswith('Warp')]
    if len(declarations)!=1: return card,None
    m=re.fullmatch(r'Warp ((?:\{(?:[0-9]+|[WUBRGC])\})+) \(You may cast this card from your hand for its warp cost\. Exile this (creature|enchantment) at the beginning of the next end step, then you may cast it from exile on a later turn\.\)',declarations[0])
    if not m or fixed_cost(m[1]) is None or spell_cost_options(m[1]) is None: return card,None
    types=card.get('type_line','').split(' — ')[0].split()
    if not any(t in types for t in ['Creature','Artifact','Enchantment','Planeswalker']) or any(t in types for t in ['Instant','Sorcery','Land']):return card,None
    if (m[2]=='creature') != ('Creature' in types):return card,None
    colored,generic=fixed_cost(m[1])
    return dict(card,oracle_text='\n'.join(line for line in lines if line!=declarations[0])),f'ManaCost {{ colored: {colored}, generic: {generic} }}'


def extract_bargain(card):
    lines=card.get('oracle_text','').splitlines()
    declaration='Bargain (You may sacrifice an artifact, enchantment, or token as you cast this spell.)'
    if lines.count(declaration)!=1 or card.get('layout')!='normal':return card,False
    return dict(card,oracle_text='\n'.join(line for line in lines if line!=declaration)),True


def fixed_bargain_body(card):
    """Complete Bargain bodies; generic creature fallback authenticates raw reminders."""
    cost=card.get('mana_cost','')
    if fixed_cost(cost) is None or spell_cost_options(cost) is None:return None
    text=card.get('oracle_text','')
    branch=lambda yes,no:['IfBargained { skip: '+str(len(yes)+1)+' }',*yes,'Skip('+str(len(no))+')',*no]
    selection=lambda n:['SelectLibrary(LibrarySelectionSpec { look: '+str(n)+', count: LibrarySelectionCount::Fixed(2), filter: LibrarySelectionFilter::Any, optional: false, reveal: false, remainder: LibraryRemainder::BottomRandom })']
    spells={
      'Target creature gets +2/+2 until end of turn. If this spell was bargained, that creature also gains flying and lifelink until end of turn.':('Creature',[grow_instruction('Targets',2,2),'IfBargained { skip: 1 }',f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({KEYWORDS["flying"]|KEYWORDS["lifelink"]}) }}'],0),
      'Target creature gets -3/-3 until end of turn. If this spell was bargained, that creature gets -5/-5 until end of turn instead.':('Creature',branch([grow_instruction('Targets',-5,-5)],[grow_instruction('Targets',-3,-3)]),0),
      'Look at the top four cards of your library. If this spell was bargained, look at the top eight cards of your library instead. Put two of them into your hand and the rest on the bottom of your library in a random order.':('None',branch(selection(8),selection(4)),0),
      "This spell costs {1} less to cast if it's bargained.\nCounter target spell.":('Spell',['Counter'],1),
      "This spell costs {2} less to cast if it's bargained.\nReturn target nonland permanent to its owner's hand. Draw a card.":('Nonland',['ReturnToHand','Draw(PlayerSet::Controller, 1)'],2),
      card.get('name','')+' deals 3 damage to target attacking or blocking creature. If this spell was bargained, destroy that creature instead.':('CreatureAttackingBlocking',branch(['Destroy'],['Damage(3)']),0),
    }
    if text in spells and card.get('type_line','').split(' — ')[0] in ('Instant','Sorcery'):
        target,program,reduction=spells[text]
        return 'Spell',0,[],(target,program),reduction
    base=creature_template(dict(card,oracle_text=''))
    if base is None:return None
    effect,_=base
    creatures={
      "This spell costs {2} less to cast if it's bargained.\nTrample\nWhen this creature enters, you gain 3 life.":(KEYWORDS['trample'],[('SelfEnters',['GainLife(PlayerSet::Controller, 3)'],'None')],2),
      'Flying\nWhen this creature enters, if it was bargained, each opponent loses 3 life and you gain 3 life.':(KEYWORDS['flying'],[('SelfBargained',['LoseLife(PlayerSet::Opponent, 3)','GainLife(PlayerSet::Controller, 3)'],'None')],0),
      'When this creature enters, if it was bargained, return target instant or sorcery card from your graveyard to your hand.':(0,[('SelfBargained',['ReturnToHand'],'InstantSorceryGraveYou')],0),
      'When this creature enters, if it was bargained, exile target artifact or enchantment an opponent controls.':(0,[('SelfBargained',['Exile'],'ArtifactEnchantmentOpponent')],0),
    }
    if text not in creatures:
        raw=role_reminder_text(text)
        if '(' in raw or ')' in raw:return None
        parsed=trigger_creature_template(dict(card,oracle_text=raw))
        return (parsed[0],parsed[1],parsed[2],None,0) if parsed else None
    keywords,triggers,reduction=creatures[text]
    return effect,keywords,triggers,None,reduction


def extract_evidence(card):
    text=card.get('oracle_text','')
    pattern=r'As an additional cost to cast this spell, you may collect evidence ([0-9]+)\. (?:(This spell costs \{2\} less to cast if evidence was collected\.) )?\((?:To collect evidence ([0-9]+), exile|Exile) cards with total mana value ([0-9]+) or greater from your graveyard\.\)'
    lines=text.splitlines();found=[(i,re.fullmatch(pattern,line)) for i,line in enumerate(lines) if re.fullmatch(pattern,line)]
    if len(found)!=1 or card.get('layout')!='normal':return card,None
    i,m=found[0]
    # The two repeated thresholds must agree; arbitrary reminders cannot qualify.
    if any(value is not None and value!=m[1] for value in (m[3],m[4])):return card,None
    return dict(card,oracle_text='\n'.join(lines[:i]+lines[i+1:])),(int(m[1]),2 if m[2] else 0)


def evidence_text(text):
    return "\n".join(line.rstrip() for line in re.sub(r"\([^()]*\)","",text).strip().splitlines()).replace("’", "\'").strip()

def fixed_evidence_body(card, declaration):
    if (card.get('layout')!='normal' or fixed_cost(card.get('mana_cost','')) is None
            or spell_cost_options(card.get('mana_cost','')) is None):return None
    text=evidence_text(card.get('oracle_text',''))
    text='\n'.join(line.rstrip() for line in text.splitlines())
    branch=lambda yes,no:['IfEvidence { skip: '+str(len(yes)+1)+' }',*yes,'Skip('+str(len(no))+')',*no]
    if declaration:
        n,discount=declaration
        spells={
          "Search your library for a basic land card. If evidence was collected, instead search your library for a creature or land card. Reveal that card, put it into your hand, then shuffle.":(8,0,'None',branch(['Search(SearchSpec { filter: SearchFilter::Kinds(3), maximum: 1, destination: SearchDestination::Hand, tapped: false, reveal: true })'],search_program('search your library for a basic land card, reveal it, put it into your hand, then shuffle.'))),
          "Target creature you control gets +2/+0 until end of turn. It deals damage equal to its power to target creature you don’t control.":(6,2,['CreatureYou','CreatureOpponent'],[grow_instruction('Targets',2,0),'PowerDamage { source: 0, recipient: 1 }']),
          "Each opponent sacrifices a creature of their choice. If evidence was collected, instead each opponent sacrifices a creature with the greatest power among creatures they control.":(6,0,'None',branch(['SacrificeGreatestPower'],sacrifice_program('each opponent sacrifices a creature.')[1])),
        }
        spells={evidence_text(k):v for k,v in spells.items()}
        if text in spells and card.get('type_line','').split(' — ')[0] in ('Instant','Sorcery'):
            expected,reduction,target,program=spells[text]
            return ('Spell',0,[],(target,program),[]) if (n,discount)==(expected,reduction) else None
        base=creature_template(dict(card,oracle_text=''))
        if not base:return None
        bodies={
          "Flying\nWhen this creature enters, tap target creature. If evidence was collected, put a stun counter on it.":(6,KEYWORDS['flying'],[('SelfEnters',['Tap','IfEvidence { skip: 1 }',counter_instruction('Targets','a','stun')],'Creature')]),
          "Reach\nWhen this creature enters, if evidence was collected, put a +1/+1 counter on target creature and you gain 2 life.":(6,KEYWORDS['reach'],[('SelfEvidence',[counter_instruction('Targets','a','+1/+1'),'GainLife(PlayerSet::Controller, 2)'],'Creature')]),
        }
        bodies={evidence_text(k):v for k,v in bodies.items()}
        if text in bodies:
            expected,keywords,triggers=bodies[text]
            if n==expected and discount==0:return base[0],keywords,triggers,None,[]
        return None
    # Each remaining full body is exact; no unknown residuals are swallowed.
    base=creature_template(dict(card,oracle_text=''))
    if not base:return None
    if text==evidence_text('Deathtouch, haste\nWard—Collect evidence 4.'):return base[0],KEYWORDS['deathtouch']|KEYWORDS['haste'],[('Ward',['EvidenceWard(4)'],'None')],None,[]
    if text==evidence_text('At the beginning of combat on your turn, you may collect evidence 4. (Exile cards with total mana value 4 or greater from your graveyard.)\nWhenever you collect evidence, investigate. (Create a Clue token. It’s an artifact with "{2}, Sacrifice this token: Draw a card.")'):
        return base[0],0,[('AtStep { step: TriggerStep::BeginCombat, relation: CastRelation::You, guard: StepGuard::Always }',['CollectEvidence { threshold: 4, skip: 0 }'],'None'),('EvidenceCollected',predefined_program('create a Clue token.'),'None')],None,[]
    if text==evidence_text('{T}: Untap another target permanent you control.\n{T}, Collect evidence 3: Tap target creature you don’t control.'):
        return base[0],0,[],None,[(activation_cost('{T}'),'OtherPermanentYou',['Untap'],False,False),(activation_cost('{T}, Collect evidence 3'),'CreatureOpponent',['Tap'],False,False)]
    if text==evidence_text('Whenever this creature attacks, you may collect evidence 3. When you do, put a +1/+1 counter on target creature you control. (To collect evidence 3, exile cards with total mana value 3 or greater from your graveyard.)'):
        return base[0],0,[('SelfAttacks',['CollectEvidence { threshold: 3, skip: 1 }','Reflexive { ability: 1 }'],'None'),('Reflexive',[counter_instruction('Targets','a','+1/+1')],'CreatureYou')],None,[]
    if text==evidence_text('When this creature enters, you may collect evidence 4. (Exile cards with total mana value 4 or greater from your graveyard.)\nWhenever you collect evidence, create a 1/1 colorless Thopter artifact creature token with flying.'):
        return base[0],0,[('SelfEnters',['CollectEvidence { threshold: 4, skip: 0 }'],'None'),('EvidenceCollected',token_program('create a 1/1 colorless Thopter artifact creature token with flying.'),'None')],None,[]
    return None

def extract_kicker(card):
    """One fixed additional mana cost; no multikicker, variables or duplicate declarations."""
    lines=card.get('oracle_text','').splitlines()
    found=[line for line in lines if line.startswith('Kicker ')]
    if len(found)!=1 or card.get('layout')!='normal':return card,None
    match=re.fullmatch(r'Kicker (?P<cost>(?:\{(?:[0-9]+|[WUBRGC])\})+)(?: \(You may pay an additional (?P=cost) as you cast this spell\.\))?',found[0])
    if not match:return card,None
    choices=spell_cost_options(match['cost']);base=spell_cost_options(card.get('mana_cost',''))
    if choices is None or len(choices)!=1 or base is None:return card,None
    colored,generic=choices[0]
    if any(generic+n>4294967295 or any(a+b>4294967295 for a,b in zip(colored,mana)) for mana,n in base):return card,None
    return dict(card,oracle_text='\n'.join(line for line in lines if line!=found[0])),f'ManaCost {{ colored: {colored}, generic: {generic} }}'


def scalar_x_body(card):
    """Complete fixed-target, single-X bodies. Other X/card-cost systems stay closed."""
    cost=card.get('mana_cost','')
    base=cost.replace('{X}','') or '{0}'
    if card.get('layout')!='normal' or cost.count('{X}')!=1 or fixed_cost(base) is None or spell_cost_options(base) is None:return None
    if not isinstance(card.get('oracle_text'),str):return None
    creature='Creature' in card.get('type_line','').split(' — ')[0].split()
    if creature and any(not isinstance(card.get(k),str) or re.fullmatch(r'-?[0-9]+',card[k]) is None or abs(int(card[k]))>32767 for k in ('power','toughness')):return None
    text=re.sub(r'\([^()]*\)','',card['oracle_text'])
    text='\n'.join(line.strip() for line in text.splitlines() if line.strip())
    names=[card.get('name',''),card.get('name','').split(',')[0]]
    for name in names:
        if name:text=text.replace(name,'this creature' if creature else 'this spell')
    text=text.lower()
    if not creature and card.get('type_line') not in ('Instant','Sorcery'):return None
    if creature and not text.startswith(('lifelink\n','flying\n')):return None
    x='AmountValue::ChosenX'
    def scaled_token(body):
        program=statement_program(body)
        if program is None or len(program)!=1:return None
        return program[0].replace('CreateTokens {','CreateTokensScaled {').replace('amount: 1,','amount: '+x+',')
    choco='Create a 2/2 green Bird creature token with "Whenever a land you control enters, this token gets +1/+0 until end of turn."'
    bodies={
        'draw x cards.':('None',['DrawScaled(PlayerSet::Controller, '+x+')']),
        'target creature gets +x/+0 and gains first strike until end of turn.':('Creature',['GrowScaled { subject: Subject::Targets, amount: '+x+', power: 1, toughness: 0 }','GrantKeywords { subject: Subject::Targets, keywords: Keywords(512) }']),
        'target creature gets -x/-x until end of turn.':('Creature',['GrowScaled { subject: Subject::Targets, amount: '+x+', power: -1, toughness: -1 }']),
        'tap target creature. put twice x stun counters on it.':('Creature',['Tap','PutCountersScaled { subject: Subject::Targets, kind: CounterKind::Stun, amount: AmountValue::Multiple { value: &'+x+', factor: 2 } }']),
        'counter target spell unless its controller pays {x}.':('Spell',['CounterUnlessPayScaled { amount: '+x+', exile: false }']),
        "counter target spell unless its controller pays {x}. if that spell is countered this way, exile it instead of putting it into its owner's graveyard.":('Spell',['CounterUnlessPayScaled { amount: '+x+', exile: true }']),
        'this spell deals x damage to any target. draw two cards, then discard a card.':('Any',['DamageScaled('+x+')','Draw(PlayerSet::Controller, 2)','Discard(PlayerSet::Controller, 1)']),
    }
    if m := re.fullmatch(r'each non-([a-z]+) creature gets -x/-x until end of turn\.', text):
        # Detection may use normalized text; unknown raw parentheses are never
        # authenticated by this new full-body grammar.
        if '(' in card['oracle_text'] or ')' in card['oracle_text']:return None
        subtype=next((name for name in CREATURE_TYPE_NAMES if name.casefold()==m[1]),None)
        if subtype is None:return None
        filt=creature_filter('creatures')
        return 'Spell',0,[],('None',[f'GrowCreaturesScaled {{ filter: {filt}, excluding_subtype: Some({json.dumps(subtype)}), amount: {x}, power: -1, toughness: -1 }}']),False
    if text in bodies:
        target,program=bodies[text];return 'Spell',0,[],(target,program),False
    if text=='this spell deals x damage to any target.\n'+choco.lower():
        tokens=statement_program(choco)
        if tokens:return 'Spell',0,[],('Any',['DamageScaled('+x+')',*tokens]),False
    if text=='create x 1/1 black rat creature tokens with "this token can\'t block." creatures you control gain haste until end of turn.':
        token=scaled_token('Create a 1/1 black Rat creature token with "This token can\'t block."')
        tail=statement_program('Creatures you control gain haste until end of turn.')
        if token and tail:return 'Spell',0,[],('None',[token,*tail]),False
    if text=='create x 1/1 white ally creature tokens, then put a +1/+1 counter on each creature you control.':
        token=scaled_token('Create a 1/1 white Ally creature token.')
        tail=statement_program('Put a +1/+1 counter on each creature you control.')
        if token and tail:return 'Spell',0,[],('None',[token,*tail]),False
    if creature and text=='lifelink\nthis creature enters with x +1/+1 counters on him.\nwhen this creature enters, he deals x damage to target opponent.':
        return 'Creature('+str(int(card['power']))+', '+str(int(card['toughness']))+')',64,[('SelfEnters',['DamageScaled('+x+')'],'Opponent')],None,True
    if creature and text=='flying\nthis creature enters with x +1/+1 counters on it.\nwhen this creature enters, you gain 2 life.':
        return 'Creature('+str(int(card['power']))+', '+str(int(card['toughness']))+')',1,[('SelfEnters',['GainLife(PlayerSet::Controller, 2)'],'None')],None,True
    return None


def fixed_kicker_body(card):
    """Complete closed bodies. Replacement branches execute a single real operation."""
    text=card.get('oracle_text','')
    for reminder in [" (Damage dealt by this creature also causes you to gain that much life.)", " (This creature can't be blocked except by two or more creatures.)", " (It can deal excess combat damage to the player or planeswalker it's attacking.)"]:
        text=text.replace(reminder,'')
    text=text.replace(card.get('name','')+' deals ','this spell deals ')
    branch=lambda yes,no:['IfKicked { skip: '+str(len(yes)+1)+' }',*yes,'Skip('+str(len(no))+')',*no]
    spells={
        'this spell deals 2 damage to any target. If this spell was kicked, it deals 4 damage instead.':('Any',branch(['Damage(4)'],['Damage(2)'])),
        'this spell deals 2 damage to target creature. If this spell was kicked, it deals 5 damage to that creature instead.':('Creature',branch(['Damage(5)'],['Damage(2)'])),
        "Return target nonland permanent to its owner's hand. If this spell was kicked, draw a card.":('Nonland',['ReturnToHand','IfKicked { skip: 1 }','Draw(PlayerSet::Controller, 1)']),
        'Return target creature card from your graveyard to your hand. If this spell was kicked, instead put that card onto the battlefield tapped.':('CreatureGraveYou',branch(['ReturnToBattlefieldTapped'],['ReturnToHand'])),
        'Search your library for a basic land card, put it onto the battlefield, then shuffle. If this spell was kicked, instead search your library for two basic land cards, put them onto the battlefield, then shuffle.':('None',branch(search_program('search your library for up to two basic land cards, put them onto the battlefield, then shuffle.'),search_program('search your library for a basic land card, put it onto the battlefield, then shuffle.'))),
    }
    if text in spells and card.get('type_line','').split(' — ')[0] in ('Instant','Sorcery'):
        return 'Spell',0,[],spells[text],0,False
    base=creature_template(dict(card,oracle_text=''))
    if base is None:return None
    effect,_=base
    if text=='When this creature enters, if it was kicked, target player sacrifices a creature of their choice.':
        target,program=sacrifice_program('target player sacrifices a creature of their choice.')
        return effect,0,[('SelfKicked',program,target)],None,0,False
    if text=='Lifelink\nMenace\nWhen this creature enters, if it was kicked, return target creature card from your graveyard to the battlefield.':
        return effect,KEYWORDS['lifelink']|KEYWORDS['menace'],[('SelfKicked',['ReturnToBattlefield'],'CreatureGraveYou')],None,0,False
    if text=='Lifelink\nWhen this creature enters, if it was kicked, return target nonland permanent card with mana value 2 or less from your graveyard to the battlefield.':
        return effect,KEYWORDS['lifelink'],[('SelfKicked',['ReturnToBattlefield'],'NonlandPermanentGraveYouMaxTwo')],None,0,False
    if text=='If this creature was kicked, it enters with two +1/+1 counters on it.\nEach creature you control with a +1/+1 counter on it has trample.':
        return effect,0,[],None,2,True
    return None


def extract_offspring(card):
    lines=card.get('oracle_text','').splitlines()
    found=[line for line in lines if line.startswith('Offspring ')]
    if not found:return card,None
    if len(found)!=1 or card.get('layout')!='normal' or 'Creature' not in card.get('type_line','').split(' — ')[0].split():return card,None
    m=re.fullmatch(r'Offspring (?P<cost>(?:\{[0-9WUBRGC]+\})+)(?: \(You may pay an additional (?P=cost) as you cast this spell\. If you do, when this creature enters, create a 1/1 token copy of it\.\))?',found[0])
    if not m:return card,None
    choices=spell_cost_options(m[1])
    if choices is None or len(choices)!=1:return card,None
    colored,generic=choices[0]
    base=spell_cost_options(card.get('mana_cost',''))
    if base is None or any(generic+n>4294967295 or any(a+b>4294967295 for a,b in zip(colored,mana)) for mana,n in base):return card,None
    return dict(card,oracle_text='\n'.join(line for line in lines if line!=found[0])), f'ManaCost {{ colored: {colored}, generic: {generic} }}'


def extract_flashback(card):
    """One exact fixed mana alternative; every residual sentence still compiles."""
    if card.get('layout') != 'normal' or card.get('type_line') not in ('Instant', 'Sorcery'):
        return card, None
    lines = card.get('oracle_text', '').split('\n')
    found = [line for line in lines if line.startswith('Flashback ')]
    if len(found) != 1: return card, None
    match = re.fullmatch(r'Flashback ((?:\{[0-9WUBRGC]+\})+)(?: \(You may cast this card from your graveyard for its flashback cost(?: and any additional costs)?\. Then exile it\.\))?', found[0])
    if not match: return card, None
    options = spell_cost_options(match[1])
    if options is None or len(options) != 1: return card, None
    mana, generic = options[0]
    return dict(card, oracle_text='\n'.join(line for line in lines if line != found[0])), f'ManaCost {{ colored: {mana}, generic: {generic} }}'


def extract_additional_cost(card):
    """One mandatory object cost; the entire residual text must still compile."""
    text = card.get('oracle_text', '')
    lines = text.split('\n')
    costs = [line for line in lines if line.startswith('As an additional cost to cast this spell, ')]
    if len(costs) != 1: return card, None
    line = costs[0]
    if line in ('As an additional cost to cast this spell, forage or pay {B}.','As an additional cost to cast this spell, forage or pay {B}. (To forage, exile three cards from your graveyard or sacrifice a Food.)'):
        kind, spec = 'ForageOrBlack', sacrifice_spec('a permanent')
    elif line == 'As an additional cost to cast this spell, discard a card.':
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


def milled_selection_program(text):
    def emit(amount,filter_,count,destination):
        return [mill_instruction('Controller',amount),'SelectMilled(MilledSelectionSpec { filter: MilledFilter::'+filter_+', count: MilledCount::'+count+', destination: MilledDestination::'+destination+' })']
    if m:=re.fullmatch(r'mill (.+?)(?:, then|\. Then) put (?:a|an) (creature, enchantment, or land|instant or sorcery|instant, sorcery, or Faerie) card from among (?:them|the milled cards) (into your hand|onto the battlefield)\.',text):
        amount=mill_quantity(m[1]);filter_={'creature, enchantment, or land':'Kinds(35)','instant or sorcery':'Kinds(12)','instant, sorcery, or Faerie':'KindsOrSubtype { kinds: 12, subtype: "Faerie" }'}[m[2]]
        if amount is not None:return emit(amount,filter_,'Range { minimum: 1, maximum: 1 }','Hand' if m[3]=='into your hand' else 'Battlefield')
    if m:=re.fullmatch(r'mill (.+?)(?:, then|\. Then) put up to two land cards from among (?:them|the milled cards) into your hand\.',text):
        amount=mill_quantity(m[1])
        if amount is not None:return emit(amount,'Kinds(2)','Range { minimum: 0, maximum: 2 }','Hand')
    if m:=re.fullmatch(r'mill (.+?)(?:, then|\. Then) put all (instant and sorcery cards|cards that have an Adventure) from among (?:them|the milled cards) into your hand\.',text):
        amount=mill_quantity(m[1])
        if amount is not None:return emit(amount,'Kinds(12)' if m[2]=='instant and sorcery cards' else 'Adventure','All','Hand')
    return None


def mill_program(text):
    if selected:=milled_selection_program(text):return selected
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


def library_selection_filter(noun):
    """Exact Oracle noun phrases; subtype unions preserve all card kinds."""
    def join(kind,*filters):
        return kind+'(&['+', '.join('LibrarySelectionFilter::'+f for f in filters)+'])'
    def sub(name):return 'Subtype('+json.dumps(name)+')'
    return {
        'a permanent card':'Kinds(371)',
        'an artifact, creature, or land card':'Kinds(19)',
        'a creature card':'Kinds(1)',
        'a creature card with power 2 or less':'CreaturePowerAtMost(2)',
        'an artifact card':'Kinds(16)',
        'a Mutant, Ninja, Turtle, or land card':join('AnyOf',sub('Mutant'),sub('Ninja'),sub('Turtle'),'Kinds(2)'),
        'a Goblin, Swamp, or Mountain card':join('AnyOf',sub('Goblin'),sub('Swamp'),sub('Mountain')),
        'an Elf, Swamp, or Forest card':join('AnyOf',sub('Elf'),sub('Swamp'),sub('Forest')),
        'an Elemental, Island, or Mountain card':join('AnyOf',sub('Elemental'),sub('Island'),sub('Mountain')),
        'a Kithkin, Forest, or Plains card':join('AnyOf',sub('Kithkin'),sub('Forest'),sub('Plains')),
        'a Merfolk, Plains, or Island card':join('AnyOf',sub('Merfolk'),sub('Plains'),sub('Island')),
        'a Mount creature card or a Plains card':join('AnyOf',join('AllOf','Kinds(1)',sub('Mount')),sub('Plains')),
        'an Ally, Equipment, or Lesson card':join('AnyOf',sub('Ally'),sub('Equipment'),sub('Lesson')),
        'an artifact or Pirate card':join('AnyOf','Kinds(16)',sub('Pirate')),
        'a noncreature, nonland card':'ExcludingKinds(3)',
    }.get(noun)


def library_selection_program(text):
    """Complete private look/pick/remainder clauses; no guessed pronouns or tails."""
    m=re.fullmatch(r'look at the top (three|four|five) cards of your library\. (.+)',text)
    if m is None:return None
    look={'three':3,'four':4,'five':5}[m[1]];body=m[2]
    def emit(count,filter_='Any',optional=False,reveal=False,remainder='BottomAny'):
        return ['SelectLibrary(LibrarySelectionSpec { look: '+str(look)+', count: LibrarySelectionCount::'+count+', filter: LibrarySelectionFilter::'+filter_+', optional: '+str(optional).lower()+', reveal: '+str(reveal).lower()+', remainder: LibraryRemainder::'+remainder+' })']
    if body=='Put one of those cards into your hand and the rest on the bottom of your library in any order. Put each of those cards into your hand instead if there are three or more Lesson cards in your graveyard.':
        return emit('Lessons')
    if body=='Put one of them into your hand and the rest on the bottom of your library in any order. If there is an instant card and a sorcery card in your graveyard, instead put two of them into your hand and the rest on the bottom of your library in any order.':
        return emit('InstantAndSorcery')
    if body=='You may reveal a creature or land card from among them and put it into your hand. If you gained life this turn, you may instead reveal two creature and/or land cards from among them and put them into your hand. Put the rest on the bottom of your library in a random order.':
        return emit('GainedLife','Kinds(3)',True,True,'BottomRandom')
    if m:=re.fullmatch(r'Put (one|two) of (?:them|those cards) into your hand and the rest (into your graveyard|on the bottom of your library in any order)\.(?: (You gain [0-9]+ life\.))?',body):
        tail=statement_program(m[3]) if m[3] else []
        if tail is None:return None
        return emit('Fixed('+str({'one':1,'two':2}[m[1]])+')',remainder='Graveyard' if m[2]=='into your graveyard' else 'BottomAny')+tail
    if m:=re.fullmatch(r'You may reveal (.+) from among them and put it into your hand\. Put the rest on the bottom of your library in (any|a random) order\.',body):
        filter_=library_selection_filter(m[1])
        if filter_ is None:return None
        return emit('Fixed(1)',filter_,True,True,'BottomAny' if m[2]=='any' else 'BottomRandom')
    return None


def exile_play_program(text):
    # Only entire owned face-up fixed top-N clauses. No theft, free costs or
    # source-linked durations are inferred from fragments.
    m=re.fullmatch(r'exile the top (card|two cards) of your library\. (Until (?:the )?end of (?:your next turn|turn), you may play (?:that card|those cards|it)|You may play (?:that card|those cards|it) (?:this turn|until the end of your next turn))\.',text)
    if not m:return None
    amount=1 if m[1]=='card' else 2
    # A singular grant cannot accidentally extend to a plural batch.
    if amount==2 and 'those cards' not in m[2]:return None
    if amount==1 and 'those cards' in m[2]:return None
    return ['ExileTopPlay { amount: '+str(amount)+', next_turn: '+str('your next turn' in m[2]).lower()+' }']


def statement_program(text):
    """Compile only these complete sentences; residual text never gets ignored."""
    if not isinstance(text, str) or not text:
        return None
    text = strip_rule_label(text)
    if not text:return None
    text = normalize_duration(text[0].lower() + text[1:])
    if m:=re.fullmatch(r'you lose ([0-9]+) life and you draw an additional card\.',text):
        amount=int(m[1])
        return [f'LoseLife(PlayerSet::Controller, {amount})','Draw(PlayerSet::Controller, 1)'] if amount<=65535 else None
    if text=='you may play an additional land this turn.':
        return ['AdditionalLandPlays(1)']
    if text=='discard a card. If you do, draw two cards.':
        # Resolution is mandatory if a card exists. The guard skips both the
        # discard and dependent draw when the hand is empty; no optional decline.
        return ['If { guard: StepGuard::Threshold { value: StepValue::Hand, minimum: Some(1), maximum: None }, skip: 2 }',
                object_choice('Discard','Controller',(1,sacrifice_spec('a permanent')[1])), 'Draw(PlayerSet::Controller, 2)']
    if selected:=library_selection_program(text):return selected
    if impulse:=exile_play_program(text):return impulse
    prefix='if you control an outlaw, '
    if text.startswith(prefix):
        if impulse:=exile_play_program(text[len(prefix):]):return ['If { guard: StepGuard::Outlaw, skip: 1 }',*impulse]
    first,rest=split_sentence(text)
    if rest:
        lower=rest[0].lower()+rest[1:]
        if exile_play_program(lower) or (lower.startswith(prefix) and exile_play_program(lower[len(prefix):])):
            head=statement_program(first);tail=statement_program(rest)
            if head and tail:return head+tail

    if text=='you may forage.': return ['Forage { skip: 0 }']
    if m:=re.fullmatch(r'you may forage\. If you do, ([^.]+)\.',text):
        body=statement_program(m[1]+'.')
        return ['Forage { skip: '+str(len(body))+' }',*body] if body else None

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
    if text=='draw a card if you control three or more attacking creatures.':
        return ['If { guard: StepGuard::Expression { value: StatValue::AttackingCreatures, minimum: Some(3), maximum: None }, skip: 1 }','Draw(PlayerSet::Controller, 1)']
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
    if text=='you create a Mutagen token.':
        return predefined_program('create a Mutagen token.')
    if text=='you lose 2 life unless you control a Villain.':
        return ['If { guard: StepGuard::Expression { value: '+stat_value('Villains you control')+', minimum: None, maximum: Some(0) }, skip: 1 }','LoseLife(PlayerSet::Controller, 2)']
    if text=='this creature explores.':return ['Explore(Subject::Source)']
    if text=="this creature gets +1/+0 until end of turn and can't be blocked this turn.":return [grow_instruction('Source',1,0),'GrantKeywords { subject: Subject::Source, keywords: Keywords(16384) }']
    # A complete quoted token ability can end the final sentence without an
    # extra period outside the quote. Split only at an unquoted sentence edge.
    if not text.endswith('.') and '"' in text:
        first,tail=split_sentence(text)
        if tail:
            head=statement_program(first);rest=statement_program(tail)
            return [*head,*rest] if head and rest else None
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
for _name,_colors,_minimum,_maximum in [('SpellRedGreen',24,0,'u64::MAX'),('SpellWhiteBlack',5,0,'u64::MAX'),('SpellMana2',0,2,2),('SpellManaMax2',0,0,2),('SpellManaMin4',0,4,'u64::MAX')]:
    TARGET_RULES[_name]=f'TargetRule::SpellCharacteristics {{ colors: {_colors}, min_mana: {_minimum}, max_mana: {_maximum} }}'


for _name, _zone, _kinds, _relation, _other, _nonland in [
    ('OtherCreature', 'Battlefield', 1, -1, True, False),
    ('CreatureYou', 'Battlefield', 1, 0, False, False),
    ('OtherCreatureYou', 'Battlefield', 1, 0, True, False),
    ('CreatureOpponent', 'Battlefield', 1, 1, False, False),
    ('Permanent', 'Battlefield', 0, -1, False, False),
    ('PermanentYou', 'Battlefield', 0, 0, False, False),
    ('Nonland', 'Battlefield', 0, -1, False, True),
    ('OtherNonlandYou', 'Battlefield', 0, 0, True, True),
    ('OtherNonland', 'Battlefield', 0, -1, True, True),
    ('OtherPermanentYou', 'Battlefield', 0, 0, True, False),
    ('NonlandOpponent', 'Battlefield', 0, 1, False, True),
    ('CreatureEnchantment', 'Battlefield', 33, -1, False, False),
    ('CardGrave', 'Graveyard', 0, -1, False, False),
    ('Land', 'Battlefield', 2, -1, False, False),
    ('Artifact', 'Battlefield', 16, -1, False, False),
    ('ArtifactOpponent', 'Battlefield', 16, 1, False, False),
    ('ArtifactCreature', 'Battlefield', 17, -1, False, False),
    ('ArtifactCreatureYou', 'Battlefield', 17, 0, False, False),
    ('ArtifactCreatureOpponent', 'Battlefield', 17, 1, False, False),
    ('ArtifactCreatureEnchantmentOpponent', 'Battlefield', 49, 1, False, False),
    ('ArtifactYou', 'Battlefield', 16, 0, False, False),
    ('Enchantment', 'Battlefield', 32, -1, False, False),
    ('ArtifactEnchantment', 'Battlefield', 48, -1, False, False),
    ('ArtifactEnchantmentOpponent', 'Battlefield', 48, 1, False, False),
    ('PermanentGraveYou', 'Graveyard', 0, 0, False, False),
    ('CreatureGraveYou', 'Graveyard', 1, 0, False, False),
    ('OtherCreatureGraveYou', 'Graveyard', 1, 0, True, False),
    ('InstantSorceryGraveYou', 'Graveyard', 12, 0, False, False),
    ('ArtifactEnchantmentGraveYou', 'Graveyard', 48, 0, False, False),
    ('CreatureGrave', 'Graveyard', 1, -1, False, False),
]:
    TARGET_RULES[_name] = (f'TargetRule::Object {{ zone: TargetZone::{_zone}, kinds: {_kinds}, '
                          f'relation: {_relation}, other: {str(_other).lower()}, nonland: {str(_nonland).lower()}, tapped: false, min_mana: 0, max_mana: u32::MAX }}')


TARGET_RULES['PermanentGraveYou']='TargetRule::Filtered { base: &'+TARGET_RULES['PermanentGraveYou']+', predicate: &TargetPredicate::Permanent }'
TARGET_RULES['NonlandPermanentGraveYouMaxTwo']='TargetRule::Filtered { base: &TargetRule::Object { zone: TargetZone::Graveyard, kinds: 0, relation: 0, other: false, nonland: true, tapped: false, min_mana: 0, max_mana: 2 }, predicate: &TargetPredicate::Permanent }'

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
    'target artifact or creature an opponent controls': 'ArtifactCreatureOpponent',
    'target artifact, creature, or enchantment an opponent controls': 'ArtifactCreatureEnchantmentOpponent',
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
TARGET_RULES['CreatureEnchantmentPlaneswalker']='TargetRule::Object { zone: TargetZone::Battlefield, kinds: 97, relation: -1, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX }'
TARGET_RULES['DefendingCreature'] = 'TargetRule::DefendingCreature'
TARGET_RULES['LegendaryCreature'] = filtered_target('Creature','Legendary')
TARGET_RULES['PirateYou'] = filtered_target('CreatureYou','Subtype("Pirate")')
TARGET_RULES['DwarfYou'] = filtered_target('PermanentYou','Subtype("Dwarf")')
TARGET_RULES['CaveGraveYou'] = 'TargetRule::Filtered { base: &TargetRule::Object { zone: TargetZone::Graveyard, kinds: 0, relation: 0, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX }, predicate: &TargetPredicate::Subtype("Cave") }'
TARGET_RULES['VampireYou'] = ('TargetRule::Filtered { base: &TargetRule::Object { zone: TargetZone::Battlefield, kinds: 0, relation: 0, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: 4294967295 }, predicate: &TargetPredicate::Subtype("Vampire") }')

for _name, _base, _predicate in [
    ('CreatureAttacking', 'Creature', 'Attacking'),
    ('OtherCreatureAttackingYou', 'OtherCreatureYou', 'Attacking'),
    ('CreatureAttackingBlocking', 'Creature', 'AttackingOrBlocking'),
    ('CreatureDamagedOpponent', 'CreatureOpponent', 'Damaged'),
    ('CreatureDealtDamageOpponent', 'CreatureOpponent', 'DealtDamage'),
    ('CreaturePowerMin3', 'Creature', 'PowerAtLeast(3)'),
    ('CreaturePowerMax3', 'Creature', 'PowerAtMost(3)'),
    ('CreatureWithoutFlying', 'Creature', 'WithoutKeyword(Keywords(1))'),
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
TARGET_RULES['GoblinAttackingYou'] = 'TargetRule::Filtered { base: &'+TARGET_RULES['CreatureYou']+', predicate: &TargetPredicate::Attacking }'
TARGET_RULES['GoblinAttackingYou'] = 'TargetRule::Filtered { base: &'+TARGET_RULES['GoblinAttackingYou']+', predicate: &TargetPredicate::Subtype("Goblin") }'

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
    'target creature without flying':'CreatureWithoutFlying',
    'any target':'Any',
    'target creature':'Creature',
    'target creature an opponent controls':'CreatureOpponent',
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
    'target creature with power 3 or less': 'CreaturePowerMax3',
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
    if rule == 'CardSingleGrave':
        return f'TargetSpec {{ rule: TargetRule::Grouped {{ base: &{TARGET_RULES["CardGrave"]}, constraint: TargetGroupConstraint::SameGraveyard }}, min: {minimum}, max: {maximum} }}'
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
    'target spell with mana value 2':'SpellMana2','target spell with mana value 2 or less':'SpellManaMax2','target spell with mana value 4 or greater':'SpellManaMin4',
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


def subject_guard(subject, skip, *, relation=-1, subtype=None, tokens=None, other=False):
    """Immutable predicate over one exact live creature; no event/LKI inference."""
    filter_=(f'CreatureFilter {{ relation: {relation}, other: {str(other).lower()}, '
             f'subtype: '+('Some('+json.dumps(subtype)+')' if subtype else 'None')+
             ', keyword: Keywords(0), attacking: false, tokens: '+
             ('None' if tokens is None else 'Some('+str(tokens).lower()+')')+', artifact: false }')
    return f'IfSubject {{ subject: Subject::{subject}, filter: {filter_}, skip: {skip} }}'


def target_subject_conditional(text):
    """Closed complete antecedent/tail clauses; all targets belong to one creature."""
    if '. If ' not in text:return None
    first, tail=text.split('. If ',1)
    clauses={
        'you control that creature, draw two cards.': ({'relation':0}, ['Draw(PlayerSet::Controller, 2)']),
        'an opponent controls that creature, put a stun counter on it.': ({'relation':1}, [counter_instruction('Targets','a','stun')]),
        'that creature is a token, it also gains deathtouch until end of turn.': ({'tokens':True}, [f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({KEYWORDS["deathtouch"]}) }}']),
        'that creature is a Cat, it gains first strike until end of turn.': ({'subtype':'Cat'}, [f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({KEYWORDS["first strike"]}) }}']),
        'that creature is a Bear, put a +1/+1 counter on it.': ({'subtype':'Bear'}, [counter_instruction('Targets','a','+1/+1')]),
        "it's a Spider, you gain 2 life.": ({'subtype':'Spider'}, ['GainLife(PlayerSet::Controller, 2)']),
        'that creature is an Ally, it also gains flying until end of turn.': ({'subtype':'Ally'}, [f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({KEYWORDS["flying"]}) }}']),
    }
    replacement='that creature is another Hero, put two +1/+1 counters on it instead.'
    if tail not in clauses and tail!=replacement:return None
    result=targeted_program(first+'.')
    if not result:return None
    spec,program=result
    if isinstance(spec,list):return None
    rule=spec[0] if isinstance(spec,tuple) else spec
    maximum=spec[2] if isinstance(spec,tuple) else 1
    if maximum!=1 or rule not in ('Creature','CreatureYou','CreatureOpponent','OtherCreatureYou','OtherCreature','OtherCreatureAttackingYou'):return None
    if any(not (p in ('Tap','Untap') or p.startswith(('Grow { subject: Subject::Targets,','GrantKeywords { subject: Subject::Targets,','PutCounters { subject: Subject::Targets,'))) for p in program):return None
    if tail==replacement:
        if program!=[counter_instruction('Targets','a','+1/+1')]:return None
        alternative=[counter_instruction('Targets','two','+1/+1')]
        return spec,[subject_guard('Targets',len(alternative)+1,subtype='Hero',other=True),*alternative,f'Skip({len(program)})',*program]
    kwargs,body=clauses[tail]
    return spec,[*program,subject_guard('Targets',len(body),**kwargs),*body]


def recovery_target_program(text):
    """Exact own-graveyard targets; generated keys encode semantics, never identity."""
    match = re.fullmatch(r'(return|put) (another target|target|up to one target|up to two target|one or two target|two target) (?:(.*?) )?cards? from your graveyard (to your hand|on (?:the )?(top|bottom) of your library)\.', text)
    if match is None:
        return None
    action, cardinality, noun, destination, where = match.groups()
    # Zone-trigger packets bind their exact first public destination separately
    # from source LKI, so `other` excludes that incarnation, never every CardId.
    if (action == 'return') != (where is None):
        return None
    noun = noun or ''
    minimum, maximum = 0, 'u32::MAX'
    if bound := re.fullmatch(r'(.+) with mana value ([0-9]+) or (less|greater)', noun):
        noun = bound[1]
        number = int(bound[2])
        if number > 4294967295:
            return None
        if bound[3] == 'less': maximum = number
        else: minimum = number
    kinds = {'': 0, 'creature': 1, 'land': 2, 'artifact': 16,
             'enchantment': 32, 'instant or sorcery': 12,
             'artifact or enchantment': 48, 'artifact and/or enchantment': 48,
             'creature or planeswalker': 65, 'creature and/or planeswalker': 65}.get(noun)
    predicate, nonland = None, False
    if noun in ('permanent', 'nonland permanent'):
        kinds, predicate, nonland = 0, 'Permanent', noun.startswith('nonland')
    elif noun == 'legendary':
        kinds, predicate = 0, 'Legendary'
    elif noun in CREATURE_TYPE_NAMES:
        kinds, predicate = 0, 'Subtype(' + json.dumps(noun) + ')'
    if kinds is None:
        return None
    expression = (f'TargetRule::Object {{ zone: TargetZone::Graveyard, kinds: {kinds}, relation: 0, '
                  f'other: {str(cardinality.startswith("another")).lower()}, nonland: {str(nonland).lower()}, '
                  f'tapped: false, min_mana: {minimum}, max_mana: {maximum} }}')
    if predicate:
        expression = 'TargetRule::Filtered { base: &' + expression + ', predicate: &TargetPredicate::' + predicate + ' }'
    key = 'Recovery' + hashlib.sha256(expression.encode()).hexdigest()
    TARGET_RULES[key] = expression
    bounds = {'up to one target': (0, 1), 'up to two target': (0, 2),
              'one or two target': (1, 2), 'two target': (2, 2)}.get(cardinality)
    targets = (key, *bounds) if bounds else key
    return targets, ['ReturnToHand' if where is None else 'TopLibrary' if where == 'top' else 'BottomLibrary']


def recovery_route(text):
    """Detect even nested/infix attempted recovery; erasure never authenticates it."""
    if not isinstance(text, str): return False
    while True:
        # Detection only: erasing an infix parenthesis can leave extra spaces,
        # while whole-body parsers normalize those spaces before recognizing it.
        # The strict validator below still receives the untouched raw body.
        if re.search(r'from\s+your\s+graveyard\s+(?:to\s+your\s+hand|on\s+(?:the\s+)?(?:top|bottom)\s+of\s+your\s+library)', text, re.IGNORECASE):
            return True
        stripped = re.sub(r'\([^()]*\)', '', text)
        if stripped == text: return False
        text = stripped


def recovery_reminders_valid(card):
    raw = card.get('oracle_text', '')
    if not recovery_route(raw) or not any(c in raw for c in '()'):
        return True
    # Each pair is trailing prose of its actual production. No nested, detached,
    # infix, duplicate or unrelated reminder can disappear into generic parsers.
    for line in raw.splitlines():
        if not any(c in line for c in '()'): continue
        match = re.fullmatch(r'([^()]+) \(([^()]+)\)', line)
        if match is None: return False
        body, reminder = match.groups()
        intrinsic = dict(card, oracle_text=line)
        if creature_template(dict(intrinsic, oracle_text=body)) is not None:
            if intrinsic_reminders_valid(intrinsic): continue
        if body == 'Prowess' and reminder == 'Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.': continue
        if body.startswith('Kicker ') and extract_kicker(dict(card, oracle_text=line))[1] is not None: continue
        if body == 'Bargain' and extract_bargain(dict(card, oracle_text=line))[1]: continue
        if (re.fullmatch(r'When this creature enters, you may forage\. When you do, return target permanent card from your graveyard to your hand\.', body)
                and reminder == 'To forage, exile three cards from your graveyard or sacrifice a Food.'): continue
        if (re.fullmatch(r'(?:• )?Put (?:a|one|two|three|four|five|six) \+1/\+1 counters? on target creature\. It gains (?:[a-z -]+ and )?indestructible until end of turn\.', body)
                and reminder == 'Damage and effects that say "destroy" don\'t destroy it.'): continue
        if match := re.fullmatch(r'(?:• )?Scry ([1-6])\.', body):
            count = int(match[1])
            words = {1:'one', 2:'two', 3:'three', 4:'four', 5:'five', 6:'six'}
            expected = ('Look at the top card of your library. You may put it on the bottom.' if count == 1 else
                        f'Look at the top {words[count]} cards of your library, then put any number of them on the bottom and the rest on top in any order.')
            if reminder == expected: continue
        if body == 'Create a Clue token.' and reminder == 'It\'s an artifact with "{2}, Sacrifice this token: Draw a card."': continue
        return False
    return True


def target_exclusion_reminders_valid(card):
    """Detect every new exclusion route before any presentation erasure."""
    raw=card.get('oracle_text','')
    if not isinstance(raw,str):return True
    surface=raw;seen=False
    while True:
        normalized=' '.join(surface.casefold().split())
        seen |= re.search(r'\bnonenchantment creature\b|\bnon-mount creature\b|\bnoncreature (?:artifact|enchantment)\b|\bchoose one or both\b',normalized) is not None
        stripped=re.sub(r'\([^()]*\)','',surface)
        if stripped==surface:break
        surface=stripped
    if not seen:return True
    for line in raw.splitlines():
        if '(' not in line and ')' not in line:continue
        if not intrinsic_reminder_line_valid(line):
            return False
    return True


def target_exclusion_program(text):
    nouns={
        'nonenchantment creature':('Creature','WithoutKinds(32)'),
        'non-mount creature':('Creature','WithoutSubtype("Mount")'),
        'noncreature artifact':('Artifact','WithoutKinds(1)'),
        'noncreature enchantment':('Enchantment','WithoutKinds(1)'),
    }
    match=re.fullmatch(r'(tap|untap|destroy) (up to one |)target (nonenchantment creature|non-mount creature|noncreature artifact|noncreature enchantment)( or noncreature enchantment|)\.',text.casefold())
    if match is None:return None
    verb,optional,noun,union=match.groups()
    if union and noun!='noncreature artifact':return None
    def rule_for(noun):
        base,predicate=nouns[noun]
        return filtered_target(base,predicate)
    expression=rule_for(noun)
    if union:expression='TargetRule::Union(&['+expression+', '+rule_for('noncreature enchantment')+'])'
    key='Exclusion'+hashlib.sha256(expression.encode()).hexdigest()
    TARGET_RULES[key]=expression
    return ((key,0,1) if optional else key),[verb.capitalize()]


def targeted_program(text):
    """Complete target occurrences with independent predicates and effect focus."""
    if not isinstance(text, str) or not text.endswith('.'):
        return None
    text = strip_rule_label(text)
    if match := re.fullmatch(r'(Return (?:up to (?:one|two) |one or two |two |)target [^.]+ cards? from your graveyard to your hand), then (discard (?:a card|two cards|three cards))\.', text):
        text = match[1] + '. ' + match[2][0].upper() + match[2][1:] + '.'
    if text.casefold()=='untap target attacking creature.':return 'CreatureAttacking',['Untap']
    text = normalize_duration(text[0].lower() + text[1:])
    if 'another target attacking creature you control' in text:
        result=targeted_program(text.replace('another target attacking creature you control','another target creature you control',1))
        return ('OtherCreatureAttackingYou',result[1]) if result and result[0]=='OtherCreatureYou' else None
    if result:=target_exclusion_program(text):return result
    if result:=role_program(text):return result
    if match:=re.fullmatch(r'destroy target creature an opponent controls with mana value ([0-9]+) or less\.',text):
        maximum=int(match[1])
        if maximum>4294967295:return None
        key='CreatureOpponentMax'+str(maximum)
        TARGET_RULES[key]=f'TargetRule::Object {{ zone: TargetZone::Battlefield, kinds: 1, relation: 1, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: {maximum} }}'
        return key,['Destroy']
    if match:=re.fullmatch(r'(you may discard (?:a card|two cards|three cards)\. If you do, [^.]+\.) ([Cc]reate a .+ Role token attached to .+\.)',text):
        prefix=statement_program(match[1]);tail=role_program(match[2])
        if prefix and tail:return tail[0],[*prefix,*tail[1]]
        return None
    if result:=target_subject_conditional(text):return result
    for subject,rule in [('another target attacking creature you control','OtherCreatureAttackingYou'),('target attacking creature','CreatureAttacking'),('target attacking Goblin you control','GoblinAttackingYou')]:
        if text.startswith(subject+' '):
            result=targeted_program('target creature '+text[len(subject)+1:])
            return (rule,result[1]) if result and result[0]=='Creature' else None
    if text=="return up to two other target nonland permanents to their owners' hands.":
        return ('OtherNonland',0,2),['ReturnToHand']
    if text=='remove all counters from up to one target creature.':
        return ('Creature',0,1),['RemoveAllCounters']
    if m:=re.fullmatch(r"put target (permanent|instant or sorcery) card from your graveyard on top of your library\.",text):
        return ('PermanentGraveYou' if m[1]=='permanent' else 'InstantSorceryGraveYou'),['TopLibrary']
    if text=="return target creature you don't control with mana value 3 or less to its owner's hand.":
        return 'CreatureOpponentMax3',['ReturnToHand']
    if text=='tap one or two target creatures.':
        return ('Creature',1,2),['Tap']
    if text=="up to two target creatures can't block this turn.":
        return ('Creature',0,2),['GrantKeywords { subject: Subject::Targets, keywords: Keywords(2048) }']
    if text=='return up to two target artifact and/or enchantment cards from your graveyard to your hand.':
        return ('ArtifactEnchantmentGraveYou',0,2),['ReturnToHand']
    if m:=re.fullmatch(r'target artifact or creature you control (.+)',text):
        if program:=keyword_effect(m[1], 'Targets'):return 'ArtifactCreatureYou',program
    # Exact single target moves. Owner choice never denotes a spell target.
    library_subjects={'target creature':'Creature','target artifact or creature':'ArtifactCreature','target nonland permanent':'Nonland'}
    if m:=re.fullmatch(r"(?:(target creature|target artifact or creature|target nonland permanent)'s owner|the owner of (target nonland permanent)) puts it on their choice of the top or bottom of their library\.",text):
        return library_subjects[m[1] or m[2]],['OwnerLibrary']
    if m:=re.fullmatch(r"put (target creature|target card from a graveyard) on the bottom of its owner's library\.",text):
        return {'target creature':'Creature','target card from a graveyard':'CardGrave'}[m[1]],['BottomLibrary']
    # Exact subject/cardinality mapping shared by announcement and resolution.
    object_subjects={
        'target creature':'Creature','target creature you control':'CreatureYou',
        'another target creature you control':'OtherCreatureYou',
        'target creature an opponent controls':'CreatureOpponent',
        'target creature or Vehicle':'CreatureVehicle',
        'target creature or Vehicle an opponent controls':'CreatureVehicleOpponent',
        'target nonland permanent':'Nonland','other target creature':'OtherCreature',
        'other target permanent you control':'OtherPermanentYou',
        'other target nonland permanent you control':'OtherNonlandYou',
        'target tapped creature':'CreatureTapped','target artifact or creature':'ArtifactCreature',
        'target artifact or creature you control':'ArtifactCreatureYou',
        'target nonland permanent an opponent controls':'NonlandOpponent','target land':'Land',
    }
    if m:=re.fullmatch(r"return (up to one |)(.+) to its owner's hand\.",text):
        if rule:=object_subjects.get(m[2]):
            return ((rule,0,1) if m[1] else rule),['ReturnToHand']
    if m:=re.fullmatch(r'(tap|untap) (up to one |)(.+)\.',text):
        if rule:=object_subjects.get(m[3]):
            spec=(rule,0,1) if m[2] else (rule,1,1) if rule=='Creature' else rule
            return spec,[m[1].capitalize()]
    if m:=re.fullmatch(r'tap (target creature|target artifact or creature) an opponent controls and put (a|one|two|three) stun counters? on it\.',text):
        return ('CreatureOpponent' if m[1]=='target creature' else 'ArtifactCreatureOpponent'),['Tap',counter_instruction('Targets',m[2],'stun')]
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
    if m:=re.fullmatch(r'put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on target legendary creature\.',text):
        return 'LegendaryCreature',[counter_instruction('Targets',m[1],m[2])]
    if counter:=counter_program(text): return counter
    if text=='exile up to one target card from a graveyard.': return ('CardGrave',0,1),['Exile']
    if text=='exile up to two target cards from a single graveyard.': return ('CardSingleGrave',0,2),['Exile']
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
    if text=='attach this Equipment to up to one target creature you control.':
        return ('CreatureYou',0,1),['AttachSource']
    if text=='attach this Equipment to target Dwarf you control.':
        return 'DwarfYou',['AttachSource']
    if text=='attach this Equipment to target Pirate creature you control.':
        return 'PirateYou',['AttachSource']
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
    if text=='target creature you control explores.':return 'CreatureYou',['Explore(Subject::Targets)']
    if text=='target creature you control explores, then it explores again.':return 'CreatureYou',['Explore(Subject::Targets)','Explore(Subject::Targets)']
    if text=='destroy target creature, enchantment, or planeswalker. Its controller creates two Map tokens.':
        token=predefined_program('create two Map tokens.')[0]
        key=re.search(r'token: (__TOKEN_[0-9a-f]+__)',token)[1]
        return 'CreatureEnchantmentPlaneswalker',[f'DestroyAndCreateForController {{ token: {key}, amount: 2 }}']
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
        "return up to one other target nonland permanent to its owner's hand.": (('OtherNonland', 0, 1), ['ReturnToHand']),
        "one or two target creatures each get +2/+1 until end of turn.": (('Creature',1,2), [grow_instruction('Targets',2,1)]),
        "return up to one other target nonland permanent you control to its owner's hand.": (('OtherNonlandYou', 0, 1), ['ReturnToHand']),
        'untap target artifact or creature.': ('ArtifactCreature', ['Untap']),
        'untap target nonland permanent.': ('Nonland', ['Untap']),
        'untap one or two target creatures. They each get +2/+2 until end of turn.': (('Creature', 1, 2), ['Untap', grow_instruction('Targets', 2, 2)]),
        "return target creature card from your graveyard to the battlefield.": ('CreatureGraveYou', ['ReturnToBattlefield']),
        "return target Cave card from your graveyard to the battlefield tapped.": ('CaveGraveYou', ['ReturnToBattlefieldTapped']),
        "return another target creature card from your graveyard to your hand.": ('OtherCreatureGraveYou', ['ReturnToHand']),
        "return target creature card from your graveyard to your hand.": ('CreatureGraveYou', ['ReturnToHand']),
        "return target instant or sorcery card from your graveyard to your hand.": ('InstantSorceryGraveYou', ['ReturnToHand']),
        "return another target permanent you control to its owner's hand.": ('OtherPermanentYou', ['ReturnToHand']),
        "return another target creature you control to its owner's hand.": ('OtherCreatureYou', ['ReturnToHand']),
        "return target creature card from a graveyard to its owner's hand.": ('CreatureGrave', ['ReturnToHand']),
        'destroy target creature.': ('Creature', ['Destroy']),
        'destroy target artifact.': ('Artifact', ['Destroy']),
        'destroy target enchantment.': ('Enchantment', ['Destroy']),
        'destroy target creature or enchantment.': ('CreatureEnchantment', ['Destroy']),
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
    if m:=re.fullmatch(r'up to two target creatures each get ([+-][0-9]+)/([+-][0-9]+) until end of turn\.',text):
        if max(abs(int(m[1])),abs(int(m[2])))<=32767:return ('Creature',0,2),[grow_instruction('Targets',int(m[1]),int(m[2]))]
    if recovery := recovery_target_program(text):
        return recovery
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
            if maximum == 1 and not rule.startswith('Recovery') and rule not in ('None', 'Player', 'Opponent', 'Spell', 'CreatureGrave', 'CreatureGraveYou'): tail = ['Untap']
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

OPPONENT_TURN_CAST_PREFIX = "Whenever you cast a spell during an opponent's turn, "

def adventure_event_route(text):
    """Detect new event routes through erased stages; never authenticate erasure."""
    if not isinstance(text, str): return False
    while True:
        surface = ' '.join(text.casefold().split())
        if (re.search(r'you cast your (?:first|second) instant or sorcery spell each turn', surface)
                or 'attacks alone' in surface
                or (re.search(r'\bcasts?\b', surface) and 'damage to that player' in surface)):
            return True
        stripped = re.sub(r'\([^()]*\)', '', text)
        if stripped == text: return False
        text = stripped


def adventure_event_reminders_valid(card):
    raw = card.get('oracle_text', '')
    if not adventure_event_route(raw) or not any(c in raw for c in '()'): return True
    for line in raw.splitlines():
        if not any(c in line for c in '()'): continue
        match = re.fullmatch(r'([^()]+) \(([^()]+)\)', line)
        if match is None: return False
        body, reminder = match.groups()
        intrinsic = dict(card, oracle_text=line)
        if (creature_template(dict(intrinsic, oracle_text=body)) is not None
                and intrinsic_reminders_valid(intrinsic)):
            continue
        # Authenticate the complete Food creation production, not a reminder
        # copied onto another ability or detached from its actual token clause.
        if (reminder == 'It\'s an artifact with "{2}, {T}, Sacrifice this token: You gain 3 life."'
                and re.search(r'(?:^|, |\. )create (?:a|one|two|three|four|five|six) (?:tapped )?Food tokens?\.$', body)
                and trigger_creature_template(dict(card, oracle_text=body)) is not None):
            continue
        return False
    return True


def qualified_cast_prefix(line):
    """Printed spell characteristics and event-only turn qualifications."""
    targeted = {
        'Whenever you cast a spell that targets a creature you control, ': 'Targeted { filter: &CastFilter::Any, target: CastTarget::Creature(CastRelation::You) }',
        'Whenever you cast a spell that targets an opponent or a creature an opponent controls, ': 'Targeted { filter: &CastFilter::Any, target: CastTarget::OpponentOrCreature }',
        'Whenever you cast a spell that targets this creature, ': 'Targeted { filter: &CastFilter::Any, target: CastTarget::Source }',
        'Whenever you cast an Equipment spell or a spell that targets a creature you control, ': 'AnyOf(&[CastFilter::Subtype("Equipment"), CastFilter::Targeted { filter: &CastFilter::Any, target: CastTarget::Creature(CastRelation::You) }])',
        'Whenever you cast an instant or sorcery spell that targets a creature, ': 'Targeted { filter: &CastFilter::InstantSorcery, target: CastTarget::Creature(CastRelation::Any) }',
        'Repartee — Whenever you cast an instant or sorcery spell that targets a creature, ': 'Targeted { filter: &CastFilter::InstantSorcery, target: CastTarget::Creature(CastRelation::Any) }',
        'Whenever you cast a noncreature spell or a Dragon spell, ': 'AnyOf(&[CastFilter::Noncreature, CastFilter::Subtype("Dragon")])',
        'Whenever you cast a noncreature or Dragon spell, ': 'AnyOf(&[CastFilter::Noncreature, CastFilter::Subtype("Dragon")])',
        'Whenever you cast a Villain spell, ': 'Subtype("Villain")',
    }
    for prefix,filter_ in targeted.items():
        if line.startswith(prefix):return prefix,cast_condition(filter_=filter_)
    for word,ordinal in [('first',1),('second',2)]:
        prefix=f'Whenever you cast your {word} instant or sorcery spell each turn, '
        if line.startswith(prefix):return prefix,cast_condition(filter_=f'InstantSorceryOrdinal({ordinal})')
    prefix='Whenever you cast your first spell each turn, '
    if line.startswith(prefix):return prefix,cast_condition(ordinal=1)

    if line.startswith(OPPONENT_TURN_CAST_PREFIX):
        return OPPONENT_TURN_CAST_PREFIX, cast_condition()
    prefix='Whenever you cast a multicolored spell, '
    if line.startswith(prefix):return prefix,cast_condition(filter_='Multicolored')
    if m:=re.match(r'Whenever (you cast|an opponent casts) a spell with mana value ([0-9]+) or (greater|less), ',line):
        value=int(m[2])
        if value>65535:return None
        kind,field=('Minimum','minimum') if m[3]=='greater' else ('Maximum','maximum')
        return m[0],cast_condition(relation='You' if m[1]=='you cast' else 'Opponent',filter_=f'ManaValue{kind} {{ filter: &CastFilter::Any, {field}: {value} }}')
    return None


def cast_condition(relation='You', filter_='Any', ordinal=None):
    ordinal = 'None' if ordinal is None else f'Some({ordinal})'
    return f'SpellCast {{ relation: CastRelation::{relation}, filter: CastFilter::{filter_}, ordinal: {ordinal} }}'


def step_guard(text):
    if text=='two or more nonland permanents entered the battlefield under your control this turn':
        return 'StepGuard::Expression { value: StatValue::NonlandEntriesThisTurn, minimum: Some(2), maximum: None }'
    if text == 'you control an outlaw': return 'StepGuard::Outlaw'
    if text == 'you gained life this turn': return 'StepGuard::GainedLife'
    if text == 'you attacked this turn': return 'StepGuard::Attacked'
    if text == 'a creature died this turn': return 'StepGuard::CreatureDied'
    if text == 'an opponent lost life this turn': return 'StepGuard::OpponentLostLife'
    if text == 'this creature is tapped': return 'StepGuard::SourceTapped'
    if text == 'you control an enchanted creature':
        return 'StepGuard::Threshold { value: StepValue::EnchantedCreatures, minimum: Some(1), maximum: None }'
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
    if text == "you've cast two or more spells this turn": return threshold('SpellsCast', minimum=2)
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


def entering_object_program(text, condition):
    """Bind only the whole explicit event-object counter clause; never a target."""
    events=('SelfEnters','OtherCreatureEntersYou','CreatureEntersYou','ArtifactEntersYou','OtherArtifactEntersYou','EnchantmentEntersYou','LandEntersYou')
    if condition not in events and not condition.startswith(('OtherSubtypeEntersYou(','SubtypeEntersYou(','PermanentEntersYou {','OtherFilteredEntersYou(','CreatureEnters {')):
        return None
    if text=="you gain 1 life. If it's a Spider, put a +1/+1 counter on it.":
        return 'None',['GainLife(PlayerSet::Controller, 1)',subject_guard('Entering',1,subtype='Spider'),counter_instruction('Entering','a','+1/+1')]
    m=re.fullmatch(r'[Pp]ut (a|one|two|three|four|five|six) \+1/\+1 counters? on that creature(?: and it gains haste until end of turn)?\.',text)
    if m is None:return None
    program=[f'PutCounters {{ subject: Subject::Entering, kind: CounterKind::PlusOne, amount: {COUNT_WORDS[m[1]]} }}']
    if ' gains haste ' in text:program.append(f'GrantKeywords {{ subject: Subject::Entering, keywords: Keywords({KEYWORDS["haste"]}) }}')
    return 'None',program


def entry_predicate_prefix(line):
    """Exact singular events; power is tested at entry, unions count once."""
    if m:=re.match(r'Whenever another creature you control with power ([0-9]+) or less enters, ',line):
        if int(m[1]) > 32767:return None
        return m[0],f'OtherFilteredEntersYou(EntryPredicate::CreaturePowerAtMost({int(m[1])}))'
    if m:=re.match(r'Whenever another outlaw you control enters, ',line):
        return m[0],'OtherFilteredEntersYou(EntryPredicate::SubtypeAny(&["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]))'
    if m:=re.match(r'Whenever another ([A-Z][a-z]+) and/or artifact you control enters, ',line):
        if m[1] not in CREATURE_TYPE_NAMES:return None
        return m[0],f'OtherFilteredEntersYou(EntryPredicate::KindOrSubtype {{ kinds: 16, subtype: {json.dumps(m[1])} }})'
    if m:=re.match(r'Whenever another ([A-Z][a-z]+(?:, [A-Z][a-z]+)*,? or [A-Z][a-z]+) you control enters, ',line):
        names=re.split(r', or |, | or ',m[1])
        canonical=(' or '.join(names) if len(names)==2 else ', '.join(names[:-1])+', or '+names[-1])
        if (m[1]!=canonical or not 2<=len(names)<=16 or len(names)!=len(set(names))
                or any(name not in CREATURE_TYPE_NAMES for name in names)):return None
        return m[0],'OtherFilteredEntersYou(EntryPredicate::SubtypeAny(&['+', '.join(json.dumps(name) for name in names)+']))'
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
        'Whenever you forage, ': 'Foraged',
        'Whenever you sacrifice one or more Foods, ': 'FoodsSacrificed',
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
        'When this creature enters, if it was bargained, ': 'SelfBargained',
        'When this creature enters, ': 'SelfEnters',
        'When this creature dies, ': 'SelfDies',
        'When this creature enters or dies, ': 'SelfEntersOrDies',
        **{f'When this {noun} is put into a graveyard from the battlefield, ': 'SelfDies'
           for noun in ('creature','artifact','enchantment','Aura','Equipment')},
        **{f'When this {noun} enters or is put into a graveyard from the battlefield, ': 'SelfEntersOrDies'
           for noun in ('creature','artifact','enchantment','Aura','Equipment')},
        'Whenever this creature attacks, ': 'SelfAttacks',
        'Whenever this creature attacks alone, ': 'SelfAttacksAlone',
        'Whenever this creature becomes tapped, ': 'SelfBecomesTapped',
        'Whenever you attack, ': 'AttackDeclared { minimum: 1, power_min: None, guard: StepGuard::Always }',
        'Whenever one or more creatures you control attack, ': 'AttackDeclared { minimum: 1, power_min: None, guard: StepGuard::Always }',
        'Whenever you attack with three or more creatures, ': 'AttackDeclared { minimum: 3, power_min: None, guard: StepGuard::Always }',
        'Whenever you attack with one or more creatures with power 4 or greater, ': 'AttackDeclared { minimum: 1, power_min: Some(4), guard: StepGuard::Always }',
        'Whenever you attack while you control a creature with power 4 or greater, ': 'AttackDeclared { minimum: 1, power_min: None, guard: StepGuard::Threshold { value: StepValue::GreatestCreaturePower, minimum: Some(4), maximum: None } }',
        'Whenever this creature enters or attacks, ': 'SelfEntersOrAttacks',
        'Whenever another artifact you control enters, ': 'OtherArtifactEntersYou',
        'Whenever an artifact you control enters, ': 'ArtifactEntersYou',
        'Whenever an enchantment you control enters, ': 'EnchantmentEntersYou',
        'Whenever another creature you control enters, ': 'OtherCreatureEntersYou',
        'Whenever a creature you control enters, ': 'CreatureEntersYou',
        'Whenever a creature you control explores, ': 'CreatureExploresYou',
        'Whenever a creature an opponent controls enters, ': 'CreatureEnters { relation: CastRelation::Opponent, other: false }',
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
            if line.startswith('Whenever ' + alias + ' becomes tapped, '):
                line='Whenever this creature becomes tapped, '+line[len('Whenever ' + alias + ' becomes tapped, '):]
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
        if qualified:=qualified_cast_prefix(line):line_prefixes.append(qualified)
        entry_prefix=entry_predicate_prefix(line)
        if entry_prefix:line_prefixes.append(entry_prefix)
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
                            or condition.startswith(('SpellCast {','LifeGained {','CardDrawn {','PlayerDamaged {','OtherSubtypeEntersYou(','SubtypeEntersYou(','PermanentEntersYou {','OtherFilteredEntersYou(','CreatureEnters {'))):
                        intervening=guard
                    else:return None  # Lookback-event conditions need prior-event state.
                # Only self-event subjects prove an initial 'it' denotes this source.
                # Other-creature events and later targeted clauses have other antecedents.
                if condition in ('SelfEnters', 'SelfDies', 'SelfEntersOrDies', 'SelfAttacks', 'SelfEntersOrAttacks', 'SelfDiscarded', 'SelfCycled'):
                    effect_text = re.sub(r'^(?:[Ii]t|[Hh]e|[Ss]he) deals ', 'this creature deals ', effect_text)
                    if effect_text=='it explores.':effect_text='this creature explores.'
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
                if condition=='SelfEnters' and effect_text=='you may forage. When you do, return target permanent card from your graveyard to your hand.':
                    result=('PermanentGraveYou',['ReturnToHand'])
                    if result is None:return None
                    target,body=result
                    index=len(abilities)+1
                    abilities.append((condition,['Forage { skip: 1 }',f'Reflexive {{ ability: {index} }}'],'None'))
                    abilities.append(('Reflexive',body,target))
                    break
                if condition=='SelfEnters' and effect_text=='you may forage. If you do, put two +1/+1 counters on it.':
                    effect_text=effect_text.replace(' on it.',' on this creature.')
                # 'When you do' queues a separate targeted reflexive ability;
                # target selection must not occur before the optional payment.
                role_payment=re.fullmatch(r'you may pay ((?:\{[0-9WUBRGC]+\})+)\. When you do, (.+)',effect_text)
                if role_payment and (result:=role_program(role_payment[2])):
                    if limited or intervening is not None:return None
                    cost=mana_cost_spec(role_payment[1])
                    if cost is None or spell_cost_options(role_payment[1]) is None:return None
                    targets,body=result;index=len(abilities)+1
                    abilities.append((condition,[f'MayPayMana {{ cost: {cost}, skip: 1 }}',f'Reflexive {{ ability: {index} }}'],'None'))
                    abilities.append(('Reflexive',body,targets))
                    break
                source_role=source_role_program(effect_text,condition)
                entering_program=entering_object_program(effect_text,condition)
                if source_role is not None:
                    targets,program=source_role
                elif entering_program is not None:
                    targets,program=entering_program
                elif condition.startswith('SpellCast {') and (m:=re.fullmatch(r'this creature deals ([0-9]+) damage to that player\.',effect_text)):
                    amount=int(m[1])
                    targets,program=('None',[f'DamagePlayers(PlayerSet::CastingPlayer, {amount})']) if amount<=32767 else ('None',None)
                elif condition.startswith('CardDrawn {'):
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
                if prefix==OPPONENT_TURN_CAST_PREFIX:condition='DuringOpponentsTurn(&TriggerCondition::'+condition+')'
                if limited: condition='OncePerTurn(&TriggerCondition::'+condition+')'
                abilities.append((condition, program, targets))
                break
        else:
            return None
    if not abilities:
        return None
    return f"Creature({int(card['power'])}, {int(card['toughness'])})", keywords, abilities


def spell_program_template(card):
    if card.get('layout')=='normal' and spell_cost_options(card.get('mana_cost','')) is not None and card.get('type_line','').split(' — ')[0] in ('Instant','Sorcery'):
        if card.get('oracle_text')=='Counter target spell. If the amount of mana spent to cast that spell was less than its mana value, you draw a card.':
            return 'Spell',['CounterThenDrawIfDiscounted']
        if card.get('oracle_text')=='Converge — Target player discards X cards, where X is the number of colors of mana spent to cast this spell.':
            return 'Player',['DiscardScaled(PlayerSet::Targeted, AmountValue::ColorsSpent)']

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
    if not lines or lines[0] not in ('Choose one —', 'Choose one or both —') or not 2 <= len(lines) - 1 <= 64:
        return None
    if lines[0] == 'Choose one or both —' and len(lines) != 3:return None
    modes = []
    bodies = []
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
        bodies.append(body)
    if lines[0] == 'Choose one or both —':
        combined=spell_program_template(dict(card,oracle_text=" ".join(bodies)))
        if combined is None:return None
        modes.append(("Choose both", *combined))
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
        elif re.fullmatch(r'Collect evidence [0-9]+',component):
            if cost['additional'] is not None:return None
            n=int(component.rsplit(' ',1)[1])
            if not 0<n<=65535:return None
            cost['additional']='AdditionalCost { kind: EffectChoiceKind::CollectEvidence('+str(n)+'), filter: '+sacrifice_spec('a permanent')[1]+' }'
        elif component == 'Forage':
            if cost['additional'] is not None:return None
            cost['additional']='AdditionalCost { kind: EffectChoiceKind::Forage, filter: '+sacrifice_spec('a permanent')[1]+' }'
        elif component == 'Discard this card':
            if cost.get('discard') or cost.get('origin','Hand')!='Hand':return None
            cost.update(discard=True,origin='Hand')
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
    if cost['additional'] is not None and ('Forage' in cost['additional'] or 'CollectEvidence' in cost['additional']) and cost['sacrifice']:return None
    if cost.get('discard') and (cost['tap'] or cost['sacrifice'] or cost['exile_self'] or cost['remove'] or cost['loyalty'] is not None or cost['additional'] is not None or cost['life']):return None
    if cost['exile_self'] and (cost['tap'] or cost['sacrifice'] or cost['life'] or cost['remove'] or cost['additional'] is not None or cost['loyalty'] is not None): return None
    return cost


def attachment_ward_route(card):
    """New typed attachment and linked-union routes, selected by grammar alone."""
    text='\n'.join(line.strip() for line in re.sub(r'\([^()]*\)', '',card.get('oracle_text','')).casefold().splitlines())
    return (re.search(r'(?m)^(?:enchanted|equipped) (?:creature|permanent)[^\n]*\bward\b',text) is not None
            or 'enchant artifact you control' in text.splitlines()
            or any(phrase in text for phrase in (
                'exile target artifact or creature an opponent controls until ',
                'exile target artifact, creature, or enchantment an opponent controls until ')))


def attachment_ward_reminders_valid(card):
    """Authenticate exact reminder syntax and repeated printed costs before extraction."""
    mana=r'(?:\{(?:[0-9]+|[WUBRGC])\})+'
    for line in card.get('oracle_text','').splitlines():
        if '(' not in line and ')' not in line:continue
        if line.count('(')!=1 or line.count(')')!=1 or not line.endswith(')') or ' (' not in line:return False
        prefix,reminder=line.rsplit(' (',1)
        if m:=re.fullmatch(r'Ward ('+mana+r')',prefix):
            expected=r'Whenever this (?:artifact|enchantment|creature) becomes the target of a spell or ability an opponent controls, counter it unless that player pays '+re.escape(m[1])+r'\.\)'
        elif m:=re.fullmatch(r'(Enchanted|Equipped) (?:creature|permanent)[^\n]*\bward ('+mana+r')\.',prefix):
            recipient=m[1].lower()+r' (?:creature|permanent)'
            expected=r'Whenever (?:'+recipient+r'|it) becomes the target of a spell or ability an opponent controls, counter it unless that player pays '+re.escape(m[2])+r'\.\)'
        elif m:=re.fullmatch(r'(?:[^\n()]+ — )?Equip ('+mana+r')',prefix):
            expected=re.escape(m[1])+r': Attach to target creature you control\. Equip only as a sorcery\.\)'
        elif m:=re.fullmatch(r'Cycling ('+mana+r')',prefix):
            expected=re.escape(m[1])+r', Discard this card: Draw a card\.\)'
        elif prefix=='Job select':
            expected=r'When this Equipment enters, create a 1/1 colorless Hero creature token, then attach this to it\.\)'
        elif m:=re.fullmatch(r'Crew ([0-9]+)',prefix):
            expected=r'Tap any number of (?:other untapped )?creatures you control with total power '+re.escape(m[1])+r' or more: This Vehicle becomes an artifact creature until end of turn\.\)'
        else:return False
        if re.fullmatch(expected,reminder) is None:return False
    return True


def attachment_grants(text):
    """Keywords and independently triggered Ward instances; never merge Ward costs."""
    if not text: return 0, []
    words=re.split(r',? and |, ',text.casefold())
    keywords=0; triggers=[]
    for word in words:
        if word in KEYWORDS:
            keywords|=KEYWORDS[word]
        elif (ward:=ward_word(word)) is not None:
            triggers.append(('Granted(&TriggerCondition::Ward)',[counter_unless(ward,'Related')],'None'))
        else:return None
    return keywords,triggers


def aura_template(card):
    if (card.get('layout') != 'normal' or permanent_type(card) != 'Enchantment'
            or card.get('type_line','').split(' — ')[-1] != 'Aura'
            or not isinstance(card.get('oracle_text'),str)
            or spell_cost_options(card.get('mana_cost','')) is None): return None
    text=re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    body=[]
    power,toughness,keywords=0,0,0
    relation=None; kinds=1; creature_grants=False; attached_abilities=[]
    for line in text.split('\n'):
        line=line.strip()
        if line in ('Enchant creature','Enchant creature you control','Enchant creature an opponent controls','Enchant artifact you control'):
            if relation is not None: return None
            relation={'Enchant creature':-1,'Enchant creature you control':0,'Enchant creature an opponent controls':1,'Enchant artifact you control':0}[line]
            kinds=16 if line=='Enchant artifact you control' else 1
        elif m:=re.fullmatch(r'Enchanted creature gets ([+-][0-9]+)/([+-][0-9]+)(?: and has (.+))?\.',line):
            if max(abs(int(m[1])),abs(int(m[2])))>32767: return None
            extra=attachment_grants(m[3])
            if extra is None: return None
            power+=int(m[1]);toughness+=int(m[2]);keywords|=extra[0];attached_abilities.extend(extra[1]);creature_grants=True
        elif m:=re.fullmatch(r'Enchanted (creature|permanent) has (.+)\.',line):
            # A creature-only triggered grant needs a live guard on an artifact Aura.
            creature_grants |= m[1]=='creature'
            extra=attachment_grants(m[2])
            if extra is None: return None
            keywords|=extra[0];attached_abilities.extend(extra[1])
        elif line.startswith('Enchanted creature ') and ('this creature '+line[len('Enchanted creature '):]).casefold() in TEXT_FLAGS:
            keywords|=TEXT_FLAGS[('this creature '+line[len('Enchanted creature '):]).casefold()]
        else:
            body.append(line.replace('When this Aura enters, ','When this enchantment enters, ').replace('until this Aura leaves the battlefield.','until this enchantment leaves the battlefield.'))
    if relation is None or (kinds!=1 and (creature_grants or power or toughness or keywords)): return None
    result=permanent_program_template(dict(card,type_line='Enchantment',oracle_text='\n'.join(body)))
    if result is None:return None
    effect,intrinsic,abilities,activations,enters=result
    return effect,intrinsic,[*abilities,*attached_abilities],activations,enters,kinds,relation,power,toughness,keywords


def equipment_template(card):
    if (card.get('layout') != 'normal' or permanent_type(card) != 'Artifact'
            or 'Equipment' not in card.get('type_line', '').split(' — ')[-1].split()
            or not isinstance(card.get('oracle_text'),str)
            or spell_cost_options(card.get('mana_cost','')) is None): return None
    text=re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    body=[]
    attached_abilities=[]
    def attack_ability(text,condition):
        if text=='tap target creature defending player controls.':
            return condition,['Tap'],'DefendingCreature'
        result=targeted_program(text)
        if result:return condition,result[1],result[0]
        program=statement_program(text)
        return (condition,program,'None') if program else None
    power,toughness,keywords,equips=0,0,0,0
    for line in text.split('\n'):
        line=line.strip()
        if m:=re.fullmatch(r'Equipped creature gets ([+-][0-9]+)/([+-][0-9]+) and has (.*?)"Whenever this creature attacks, (.+)"',line):
            if max(abs(int(m[1])),abs(int(m[2])))>32767:return None
            prefix=m[3]
            if prefix and not prefix.endswith(' and '):return None
            extra=keyword_list(prefix[:-5]) if prefix else 0
            if extra is None:return None
            power+=int(m[1]);toughness+=int(m[2]);keywords|=extra
            ability=attack_ability(m[4],'GrantedAttacks')
            if ability is None:return None
            attached_abilities.append(ability)
        elif line.startswith('Whenever equipped creature attacks, '):
            ability=attack_ability(line[len('Whenever equipped creature attacks, '):],'EquippedAttacks')
            if ability is None:return None
            attached_abilities.append(ability)
        elif m:=re.fullmatch(r'Equipped creature gets ([+-][0-9]+)/([+-][0-9]+)(?: and has (.+))?\.',line):
            if max(abs(int(m[1])),abs(int(m[2])))>32767: return None
            extra=attachment_grants(m[3])
            if extra is None: return None
            power+=int(m[1]);toughness+=int(m[2]);keywords|=extra[0];attached_abilities.extend(extra[1])
        elif m:=re.fullmatch(r'Equipped creature has (.+)\.',line):
            extra=attachment_grants(m[1])
            if extra is None: return None
            keywords|=extra[0];attached_abilities.extend(extra[1])
        elif m:=re.fullmatch(r'Equip Pirate ((?:\{(?:[0-9]+|[WUBRGC])\})+)',line):
            equips+=1
            body.append(m[1]+': Attach this Equipment to target Pirate creature you control. Activate only as a sorcery.')
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
    if not result:return None
    effect,intrinsic,abilities,activations,enters=result
    return effect,intrinsic,[*abilities,*attached_abilities],activations,enters,power,toughness,keywords



JOB_TYPES = {'Artificer','Bard','Cleric','Knight','Monk','Ninja','Rogue','Samurai','Shaman','Warrior','Wizard'}
JOB_REMINDER = 'Job select (When this Equipment enters, create a 1/1 colorless Hero creature token, then attach this to it.)'

def job_granted_abilities(card, text):
    """The quoted ability belongs to the recipient; no leftover text is erased."""
    text=text.rstrip(',')
    if not text.endswith('.'):text+='.'
    if text=='Whenever this creature attacks, you may put a creature card from your hand onto the battlefield. If that card is an enchantment card, it enters tapped and attacking.':
        return [('Granted(&TriggerCondition::SelfAttacks)',['PutHandCreature'],'None')]
    ninja="Whenever this creature deals combat damage to a player, draw a card, then discard a card. That player loses life equal to the discarded card's mana value."
    if text==ninja:
        return [('Granted(&TriggerCondition::'+player_damage_prefix(text)[1]+')',['Draw(PlayerSet::Controller, 1)','DiscardManaValueLifeLoss'],'None')]
    prefix='Whenever you cast a noncreature spell and whenever you draw your third card each turn, '
    if text.startswith(prefix):
        program=statement_program(text[len(prefix):])
        if program is None:return None
        result=[(cast_condition(filter_='Noncreature'),program,'None'),(draw_condition(ordinal=3),program,'None')]
    else:
        result=trigger_creature_template(dict(card,type_line='Creature',power='0',toughness='1',oracle_text=text))
        if result is None or result[1] != 0:return None
        result=result[2]
    if not result:return None
    # Only event collectors with real recipient provenance are implemented.
    # Other intrinsic trigger grammar must not silently revive a granted ability.
    if any(not (condition=='SelfAttacks' or condition.startswith(('SpellCast {','CardDrawn {','PlayerDamaged { subject: DamageSubject::Source,'))) for condition,_,_ in result):return None
    return [('Granted(&TriggerCondition::'+condition+')',program,targets) for condition,program,targets in result]

def job_granted_line(line):
    # Exact grammatical arrangements in the snapshot, retaining every quoted clause.
    patterns=[
      r'Equipped creature gets ([+][0-9]+)/([+][0-9]+), has "(.+)"(?:,)? and is (?:a|an) ([A-Z][a-z]+) in addition to its other types\.',
      r'Equipped creature gets ([+][0-9]+)/([+][0-9]+), is (?:a|an) ([A-Z][a-z]+) in addition to its other types, and has "(.+)"',
      r'Equipped creature is (?:a|an) ([A-Z][a-z]+) in addition to its other types and has "(.+)"',
      r'Equipped creature has "(.+)" and is (?:a|an) ([A-Z][a-z]+) in addition to its other types\.',
    ]
    for i,pattern in enumerate(patterns):
        if m:=re.fullmatch(pattern,line):
            if i==0:return int(m[1]),int(m[2]),m[4],m[3]
            if i==1:return int(m[1]),int(m[2]),m[3],m[4]
            if i==2:return 0,0,m[1],m[2]
            return 0,0,m[2],m[1]
    return None

def job_select_template(card):
    """Closed full Job Select Equipment bodies; subtype/turn grants are immutable."""
    if (card.get('layout') != 'normal' or permanent_type(card) != 'Artifact'
            or 'Equipment' not in card.get('type_line','').split(' — ')[-1].split()
            or spell_cost_options(card.get('mana_cost','')) is None):return None
    lines=card.get('oracle_text','').split('\n')
    if not lines or lines[0] not in ('Job select', JOB_REMINDER):return None
    body=[]
    for line in lines[1:]:
        line=line.strip()
        if m:=re.fullmatch(r'(.+ Equip (\{[^}]+\}(?:\{[^}]+\})*)) \((.+)\)',line):
            if m[3] != m[2]+': Attach to target creature you control. Equip only as a sorcery.':return None
            line=m[1]
        elif m:=re.fullmatch(r'(Equip (\{[^}]+\}(?:\{[^}]+\})*)) \((.+)\)',line):
            if m[3] != m[2]+': Attach to target creature you control. Equip only as a sorcery.':return None
            line=m[1]
        body.append(line)
    lines=body
    subtype=None;power=toughness=keywords=turn_keywords=0;scales=[];equip=None;granted=[]
    for line in lines:
        if parsed:=job_granted_line(line):
            if subtype is not None:return None
            power,toughness,subtype,quoted=parsed
            if subtype not in JOB_TYPES or max(power,toughness)>32767:return None
            granted=job_granted_abilities(card,quoted)
            if granted is None:return None
        elif m:=re.fullmatch(r'Equipped creature gets ([+][0-9]+)/([+][0-9]+)(?:, has ([a-z {}0-9]+),)? and is (?:a|an) ([A-Z][a-z]+) in addition to its other types\.',line):
            if subtype is not None or m[4] not in JOB_TYPES:return None
            power,toughness=int(m[1]),int(m[2]);subtype=m[4]
            if max(power,toughness)>32767:return None
            if m[3] and (ward:=ward_word(m[3])) is not None:
                granted=[('Granted(&TriggerCondition::Ward)',[counter_unless(ward,'Related')],'None')]
                keywords=0
            else:
                keywords=keyword_list(m[3]) if m[3] else 0
                if keywords is None:return None
        elif m:=re.fullmatch(r'Equipped creature gets ([+][0-9]+)/([+][0-9]+) for each artifact you control and is (?:a|an) ([A-Z][a-z]+) in addition to its other types\.',line):
            if subtype is not None or m[3] not in JOB_TYPES:return None
            a,b=int(m[1]),int(m[2]);subtype=m[3]
            if max(a,b)>32767:return None
            value=stat_value('artifact you control')
            if value is None:return None
            scales=[f'ScalingModifier {{ subject: Subject::Attached, value: {value}, stats: Stats {{ power: {a}, toughness: {b} }} }}']
        elif line=='During your turn, equipped creature has flying.':
            if turn_keywords:return None
            turn_keywords=KEYWORDS['flying']
        elif m:=re.fullmatch(r'(?:[A-Z][A-Za-z\x27 ]* — )?(Equip (?:\{(?:[0-9]+|[WUBRGC])\})+|Equip—Pay 3 life\. Activate only once each turn\.)',line):
            if equip is not None:return None
            equip=m[1]
        else:return None
    if subtype is None or equip is None:return None
    if equip.startswith('Equip—'):
        result=permanent_program_template(dict(card,type_line='Artifact',oracle_text='Pay 3 life: Attach this Equipment to target creature you control. Activate only as a sorcery and only once each turn.'))
        if result is None:return None
        effect,intrinsic,abilities,activations,enters=result
    else:
        result=equipment_template(dict(card,oracle_text=equip))
        if result is None:return None
        effect,intrinsic,abilities,activations,enters,_,_,_=result
    token=token_program('create a 1/1 colorless Hero creature token.')[0]
    token=re.search(r'token: ([^,]+),',token)[1]
    abilities=[('SelfEnters',[f'JobSelect {{ token: {token} }}'],'None'),*abilities,*granted]
    return (effect,intrinsic,abilities,activations,enters,power,toughness,keywords),subtype,turn_keywords,scales

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


def crew_vehicle_body(card):
    """Bind authenticated self-Vehicle clauses to the source, never external nouns.

    The creature-shaped parser is only a local source-subject grammar. Executable
    identity remains the original Artifact, and layer-4 Crew adds Creature later.
    Every residual sentence must compile; this does not erase Vehicle mentions.
    """
    if (card.get('layout') != 'normal' or permanent_type(card) != 'Artifact'
            or 'Vehicle' not in card.get('type_line','').split(' — ')[-1].split()
            or not isinstance(card.get('oracle_text'),str)
            or not spell_cost_options(card.get('mana_cost',''))): return None
    if any(not isinstance(card.get(axis),str) or not (card[axis].isdigit() or card[axis]=='*')
           or (card[axis].isdigit() and int(card[axis])>2**127-1) for axis in ('power','toughness')): return None
    declarations=[];body=[]
    for line in card['oracle_text'].splitlines():
        if line.startswith('Crew '):
            # Authenticate the entire reminder as well as the nonnegative count.
            m=re.fullmatch(r'Crew ([0-9]+)(?: \(Tap any number of (?:other untapped )?creatures you control with total power ([0-9]+) or more: This Vehicle becomes an artifact creature until end of turn\.\))?',line)
            if m is None or int(m[1])>65535 or (m[2] is not None and int(m[2])!=int(m[1])): return None
            declarations.append(int(m[1]))
        else:body.append(line)
    if len(declarations)!=1:return None
    # Exact deictic source phrases only. Target Vehicle/card/other-Vehicle clauses
    # are left untouched and rejected unless the complete grammar supports them.
    text='\n'.join(body).replace('this Vehicle','this creature').replace('This Vehicle','This creature')
    proxy=dict(card,type_line='Artifact Creature',oracle_text=text)
    proxy,dynamic,scaling=extract_stat_rules(proxy)
    # A printed star is not a zero-power approximation: it needs a complete CDA.
    if '*' in card['power']+card['toughness'] and dynamic is None:return None
    parsed=permanent_program_template(proxy)
    if parsed is None:
        basic=creature_template(proxy);trigger=trigger_creature_template(proxy)
        if basic:effect,keywords=basic;abilities=[]
        elif trigger:effect,keywords,abilities=trigger
        else:return None
        activations=[];enters=[0,0,0]
    else:effect,keywords,abilities,activations,enters=parsed
    cost=activation_cost('{0}');cost['mana_component']=False
    activations=[*activations,(cost,'None',[f'Crew {{ minimum: {declarations[0]} }}'],False,False)]
    if len(activations)>64:return None
    return 'Permanent',keywords,abilities,activations,enters,dynamic,scaling


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
        for label in ('Do You Like Squirrels? — ', 'I LOVE Squirrels! — ', 'Fateful Bite — ', 'Blow Up — '):
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
            if m:=re.fullmatch(r'Put (a|one|two|three|four|five|six) (\+1/\+1|-1/-1|stun) counters? on '+re.escape(alias)+r'\.',effect_text):
                effect_text='Put '+m[1]+' '+m[2]+' counter'+('' if m[1] in ('a','one') else 's')+' on this creature.'
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
            if effect_text == 'this permanent deals damage equal to its power to target creature.' and creature and cost['tap'] and cost['sacrifice'] and cost['additional'] is None:
                targets,program='Creature',['DamageScaled(AmountValue::SourcePower)']
            else:
                targeted = targeted_program(effect_text)
                if targeted: targets, program = targeted
                else: targets, program = 'None', statement_program(effect_text)
        if program is None: return None
        if any(instruction.startswith('AddMana') for instruction in program) and (len(program) != 1 or targets != 'None'):
            return None
        # Counterfactual mana search does not consume hand-card resources yet.
        if program[0].startswith('AddMana') and (cost['additional'] is not None or cost['exile_self'] or cost.get('discard',False)): return None
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
    'This land enters tapped unless you control a Plains or an Island.': 'UnlessPlainsOrIsland',
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


def extract_opponent_entry_tapped(card):
    """Exact opposing-permanent entry replacements; retain every residual rule."""
    if (card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str)
            or permanent_type(card) not in ('Creature', 'Artifact Creature', 'Enchantment Creature', 'Artifact', 'Enchantment', 'Planeswalker')):
        return card, 0
    clauses = {
        subject + ' your opponents control enter' + battlefield + ' tapped.': mask
        for subject, mask in [('Creatures', 1), ('Artifacts', 16), ('Artifacts and creatures', 17)]
        for battlefield in ['', ' the battlefield']
    }
    body, mask = [], 0
    for line in card['oracle_text'].split('\n'):
        clean = line.strip()
        # A cosmetic label is removed only together with this whole exact rule.
        if clean.startswith('Venom Blast — '): clean = clean[len('Venom Blast — '):]
        value = clauses.get(clean)
        if value is None: body.append(line)
        else: mask |= value
    return dict(card, oracle_text='\n'.join(body)), mask


def extract_entry_trigger_suppression(card):
    """Closed entry-event prohibitions; admit only with the entire residual body."""
    if (card.get('layout') != 'normal' or not isinstance(card.get('oracle_text'), str)
            or permanent_type(card) not in ('Creature', 'Artifact Creature', 'Enchantment Creature', 'Artifact', 'Enchantment', 'Planeswalker')):
        return card, 0
    clauses = {
        "Creatures entering don't cause abilities to trigger.": 1,
        "Creatures entering the battlefield don't cause abilities to trigger.": 1,
        "Artifacts and creatures entering don't cause abilities to trigger.": 17,
        "Artifacts and creatures entering the battlefield don't cause abilities to trigger.": 17,
    }
    body, mask = [], 0
    for line in card['oracle_text'].split('\n'):
        value = clauses.get(line.strip())
        if value is None: body.append(line)
        else: mask |= value
    return dict(card, oracle_text='\n'.join(body)), mask


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


def endure_body(card):
    """Closed fixed endure bodies; no discarded variable or entering-subject tail."""
    if not isinstance(card.get('oracle_text'), str): return None
    text = re.sub(r'\([^()]*\)', '', card['oracle_text']).strip()
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    base = creature_template(dict(card, oracle_text=''))
    if base is None: return None
    name = card.get('name', '')
    aliases = [name]
    if name and card.get('type_line', '').startswith('Legendary '): aliases.append(name.split(',')[0])
    triggers, residual = [], []
    prefixes = {
        'When this creature enters, ': 'SelfEnters',
        'Whenever this creature attacks, ': 'SelfAttacks',
        'Whenever this creature enters or attacks, ': 'SelfEntersOrAttacks',
        'Whenever another nontoken creature you control dies, ': 'CreatureDiesFiltered { filter: CreatureFilter { relation: 0, other: true, subtype: None, keyword: Keywords(0), attacking: false, tokens: Some(false), artifact: false } }',
    }
    for line in lines:
        prefix = next((prefix for prefix in prefixes if line.startswith(prefix)), None)
        if prefix is None:
            residual.append(line)
            continue
        condition, tail = prefixes[prefix], line[len(prefix):]
        for alias in aliases:
            if alias and tail.startswith(alias + ' endures '): tail = 'this creature endures ' + tail[len(alias) + len(' endures '):]
        self_event = condition in ('SelfEnters', 'SelfAttacks', 'SelfEntersOrAttacks')
        if self_event and tail.startswith('it endures '): tail = 'this creature endures ' + tail[len('it endures '):]
        program = []
        if self_event and (m := re.fullmatch(r'you may pay ((?:\{[0-9WUBRGC]+\})+)\. If you do, it endures ([123])\.', tail)):
            cost = mana_cost_spec(m[1])
            if cost is None or spell_cost_options(m[1]) is None: return None
            program.append(f'MayPayMana {{ cost: {cost}, skip: 1 }}')
            tail = 'this creature endures ' + m[2] + '.'
        if self_event and (m := re.fullmatch(r'you lose 1 life and this creature endures ([123])\.', tail)):
            program.append('LoseLife(PlayerSet::Controller, 1)')
            tail = 'this creature endures ' + m[1] + '.'
        m = re.fullmatch(r'this creature endures ([123])\.', tail)
        if m is None: return None
        amount = int(m[1])
        token = token_program(f'create a {amount}/{amount} white Spirit creature token.')
        if token is None: return None
        marker = re.search(r'token: (__TOKEN_[0-9a-f]+__)', token[0])[1]
        program.append(f'Endure {{ amount: {amount}, token: {marker} }}')
        triggers.append((condition, program, 'None'))
    # Every non-trigger line must be a complete existing intrinsic creature body.
    intrinsic = creature_template(dict(card, oracle_text='\n'.join(residual)))
    if intrinsic is None or not triggers: return None
    return intrinsic[0], intrinsic[1], triggers, [], [0, 0, 0]

def mobilize_body(card):
    """Intrinsic mobilize, fixed or the exact creature-card graveyard expression.
    The complete residual still must compile; no contributor/copy/restriction gates
    are bypassed and unknown dynamic amounts never become a fixed approximation.
    """
    if (not isinstance(card.get('oracle_text'), str) or card.get('layout') != 'normal'
            or spell_cost_options(card.get('mana_cost', '')) is None):
        return None
    lines=[line.strip() for line in re.sub(r'\([^()]*\)','',card['oracle_text']).strip().split('\n')]
    declarations=[line for line in lines if line.startswith('Mobilize ')]
    if len(declarations)!=1:return None
    declaration=declarations[0]
    if m:=re.fullmatch(r'Mobilize ([0-9]+)',declaration):
        if int(m[1])>65535:return None
        amount=f'AmountValue::Fixed({int(m[1])})'
    elif declaration=='Mobilize X, where X is the number of creature cards in your graveyard.':
        amount=amount_value('the number of creature cards in your graveyard')
    else:return None
    proxy=dict(card,oracle_text='\n'.join(line for line in lines if line!=declaration),keywords=[k for k in card.get('keywords',[]) if k!='Mobilize'])
    body=permanent_program_template(proxy)
    if body is None:
        triggered=trigger_creature_template(proxy)
        if triggered is not None:body=(*triggered,[],[0,0,0])
    if body is None:
        base=creature_template(proxy)
        if base is None:return None
        body=(*base,[],[],[0,0,0])
    effect,keywords,abilities,activations,enters=body
    token=token_program('create a 1/1 red Warrior creature token.')
    assert token and len(token)==1
    factory=re.search(r'token: (__TOKEN_[a-f0-9]+__)',token[0])[1]
    trigger=('SelfAttacks',[f'Mobilize {{ token: {factory}, amount: {amount} }}'],'None')
    return effect,keywords,[*abilities,trigger],activations,enters


def saddle_body(card):
    """Closed whole-body Saddle grammar; contribution histories stay gated."""
    if not isinstance(card.get('oracle_text'),str): return None
    text=re.sub(r'\([^()]*\)','',card['oracle_text']).strip()
    lines=[line.strip() for line in text.split('\n')]
    saddles=[line for line in lines if line.startswith('Saddle ')]
    if len(saddles)!=1 or not (m:=re.fullmatch(r'Saddle ([0-9]+)',saddles[0])): return None
    minimum=int(m[1])
    if not 1<=minimum<=65535 or creature_template(dict(card,oracle_text='')) is None:return None
    triggers=[];residual=[]
    prefixes={
        'Whenever this creature attacks while saddled, ':'SelfAttacksWhileSaddled',
        'When this creature enters and whenever it attacks while saddled, ':'SelfEntersOrAttacksWhileSaddled',
        'Whenever this creature becomes saddled for the first time each turn, ':'SelfSaddledFirst',
    }
    for line in lines:
        if line==saddles[0]:continue
        line=line.replace(card.get('name','\0'),'this creature')
        prefix=next((p for p in prefixes if line.startswith(p)),None)
        if prefix is None:residual.append(line);continue
        tail=line[len(prefix):]
        # Only the exact self-event binds these source pronouns. Subsequent
        # arbitrary target/card pronouns are never rewritten.
        tail=re.sub(r'^it (gets|gains|deals) ',r'this creature \1 ',tail)
        if tail=='untap it and put a +1/+1 counter on it.':
            tail='untap this creature. Put a +1/+1 counter on this creature.'
        if tail=='put a +1/+1 counter on it.':tail='put a +1/+1 counter on this creature.'
        targets='None'
        if tail=='this creature gets +X/+X until end of turn, where X is the number of Mounts you control.':
            program=['GrowScaled { subject: Subject::Source, amount: '+amount_value('the number of Mounts you control')+', power: 1, toughness: 1 }']
        elif tail=='mill two cards, then this creature gets +X/+X until end of turn, where X is the number of creature cards in your graveyard.':
            program=[mill_instruction('Controller',2),'GrowScaled { subject: Subject::Source, amount: '+amount_value('the number of creature cards in your graveyard')+', power: 1, toughness: 1 }']
        elif tail=='put a +1/+1 counter on each other creature you control. You gain 1 life for each of those creatures.':
            program=statement_program('put a +1/+1 counter on each other creature you control.')+scaled_life_program('you gain 1 life for each other creature you control.')
        elif tail=='search your library for a basic Plains card, reveal it, put it into your hand, then shuffle.':
            program=[step.replace('Subtype("Forest")','Subtype("Plains")') for step in statement_program(tail.replace('basic Plains','basic Forest'))]
        else:
            targeted=targeted_program(tail)
            if targeted:targets,program=targeted
            else:
                program=statement_program(tail)
                if program is None:
                    first,rest=split_sentence(tail)
                    head=statement_program(first);remaining=statement_program(rest) if rest else None
                    program=head+remaining if head and remaining else None
        if not program:return None
        triggers.append((prefixes[prefix],program,targets))
    proxy=dict(card,oracle_text='\n'.join(residual))
    body=permanent_program_template(proxy)
    if body is None:
        base=creature_template(proxy)
        if base is None:return None
        body=(*base,[],[],[0,0,0])
    effect,keywords,abilities,activations,enters=body
    cost=activation_cost('{0}');cost['mana_component']=False
    return effect,keywords,[*abilities,*triggers],[*activations,(cost,'None',[f'BecomeSaddled {{ minimum: {minimum} }}'],True,False)],enters


def actual_mana_body(card):
    """Complete bounded payment-dependent cast listeners with typed event provenance."""
    if (card.get('layout')!='normal' or creature_template(dict(card,oracle_text='')) is None
            or not isinstance(card.get('oracle_text'),str)):return None
    residual,triggers=[],[]
    increment='Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature\'s power or toughness, put a +1/+1 counter on this creature.)'
    def counter(n):return f'PutCounters {{ subject: Subject::Source, kind: CounterKind::PlusOne, amount: {n} }}'
    def paid_condition(filter_,minimum=0,increment=False):
        return 'ManaSpent { minimum: '+str(minimum)+', increment: '+str(increment).lower()+', event: &TriggerCondition::'+cast_condition(filter_=filter_)+' }'
    def block(program,minimum=None,maximum=None):
        return ['IfManaSpent { minimum: '+('None' if minimum is None else f'Some({minimum})')+', maximum: '+('None' if maximum is None else f'Some({maximum})')+', skip: '+str(len(program))+' }',*program]
    for original in card['oracle_text'].splitlines():
        if original in ('Increment',increment):
            triggers.append((paid_condition('Any',increment=True),[counter(1)],'None'));continue
        if original.startswith('Increment'):return None
        line=original.removesuffix(' (If a permanent with a stun counter would become untapped, remove one from it instead.)')
        for label in ('Opus — ', 'Selfie Shot — '):
            if line.startswith(label):line=line[len(label):]
        match=re.fullmatch(r'Whenever you cast (an instant or sorcery spell|a noncreature spell), (.+)',line)
        if not match:residual.append(original);continue
        filter_='InstantSorcery' if match[1].startswith('an instant') else 'Noncreature'
        body=match[2]
        if not any(phrase in body.casefold() for phrase in ('mana was spent', 'mana spent')):
            residual.append(original);continue
        # Authenticate only the source's complete names before normalization.
        aliases=[card.get('name','')]
        if card.get('type_line','').startswith('Legendary '):aliases.append(card['name'].split(',')[0])
        for alias in aliases:
            if alias and body.startswith(alias+' gets '):body='this creature gets '+body[len(alias+' gets '):]
            if alias:body=body.replace(' on '+alias+'.',' on this creature.')
        body=body.casefold()
        minimum=0
        if m:=re.fullmatch(r'if at least (four|eight) mana was spent to cast it, (.+)',body):
            minimum={'four':4,'eight':8}[m[1]];body=m[2]
        target,program='None',None
        branches={
          'this creature gets +1/+1 until end of turn. if five or more mana was spent to cast that spell, this creature gets +2/+2 until end of turn instead.':('None',[grow_instruction('Source',1,1)],[grow_instruction('Source',2,2)]),
          'target player mills three cards. if five or more mana was spent to cast that spell, that player mills ten cards instead.':('Player',['Mill(PlayerSet::Targeted, MillAmount::Fixed(3))'],['Mill(PlayerSet::Targeted, MillAmount::Fixed(10))']),
          'draw a card. then discard a card unless five or more mana was spent to cast that spell.':('None',['Draw(PlayerSet::Controller, 1)','Discard(PlayerSet::Controller, 1)'],['Draw(PlayerSet::Controller, 1)']),
          'this creature gets +3/+0 until end of turn. if five or more mana was spent to cast that spell, put three +1/+1 counters on this creature instead.':('None',[grow_instruction('Source',3,0)],[counter(3)]),
          'put a +1/+1 counter on this creature. if five or more mana was spent to cast that spell, put two +1/+1 counters on this creature instead.':('None',[counter(1)],[counter(2)]),
          'this creature deals 1 damage to each opponent. if five or more mana was spent to cast that spell, this creature deals 3 damage to each opponent instead.':('None',['DamagePlayers(PlayerSet::Opponent, 1)'],['DamagePlayers(PlayerSet::Opponent, 3)']),
        }
        if body in branches:
            target,low,high=branches[body];program=[*block(low,maximum=4),*block(high,minimum=5)]
        elif body=="this creature gets +1/+1 until end of turn. if five or more mana was spent to cast that spell, this creature also gains double strike until end of turn.":
            program=[grow_instruction('Source',1,1),*block(['GrantKeywords { subject: Subject::Source, keywords: Keywords(1024) }'],minimum=5)]
        elif body in ('this creature gets +x/+0 until end of turn, where x is the amount of mana spent to cast that spell.', 'this creature gets +x/+0 until end of turn, where x is the amount of mana spent to cast that spell. if x is 4 or more, draw a card.'):
            program=['GrowScaled { subject: Subject::Source, amount: AmountValue::ManaSpent, power: 1, toughness: 0 }']
            if 'if x is' in body:program+=block(['Draw(PlayerSet::Controller, 1)'],minimum=4)
        elif body=='create a treasure token.':
            program=predefined_program('create a Treasure token.')
        elif body=='put eight +1/+1 counters on this creature.':
            program=[counter(8)]
        elif body=="put a +1/+1 counter on this creature and it can't be blocked this turn.":
            program=[counter(1),'GrantKeywords { subject: Subject::Source, keywords: Keywords(16384) }']
        else:
            result=targeted_program(body)
            if result:target,program=result
            else:program=statement_program(body)
        if program is None:return None
        triggers.append((paid_condition(filter_,minimum),program,target))
    if not triggers:return None
    proxy=dict(card,oracle_text='\n'.join(residual))
    base=permanent_program_template(proxy)
    if base is None:
        trigger=trigger_creature_template(proxy)
        if trigger:base=(*trigger,[],[0,0,0])
    if base is None:
        intrinsic=creature_template(proxy)
        if intrinsic:base=(*intrinsic,[],[],[0,0,0])
    if base is None:return None
    effect,keywords,abilities,activations,enters=base
    return effect,keywords,[*abilities,*triggers],activations,enters


def rust_activations(abilities):
    rows = []
    for cost, targets, program, sorcery, once in abilities:
        remove = 'None' if cost['remove'] is None else f"Some(({'None' if cost['remove'][0] is None else 'Some(CounterKind::' + cost['remove'][0] + ')'}, {cost['remove'][1]}))"
        alternatives = '&[' + ', '.join(f'ManaCost {{ colored: {mana}, generic: {generic} }}' for mana,generic in cost.get('alternatives', [])) + ']'
        rows.append(f"ActivatedAbility {{ exhaust: {str(cost.get('exhaust',False)).lower()}, guard: ActivationGuard::{cost.get('guard','Always')}, cycling: {str(cost.get('cycling',False)).lower()}, origin: ActivationOrigin::{cost.get('origin','Battlefield')}, cost: ActivationCost {{ colored: {cost['colored']}, generic: {cost['generic']}, alternatives: {alternatives}, additional: {'None' if cost.get('additional') is None else 'Some('+cost['additional']+')'}, mana_component: {str(cost['mana_component']).lower()}, tap: {str(cost['tap']).lower()}, sacrifice: {str(cost['sacrifice']).lower()}, exile_self: {str(cost['exile_self']).lower()}, discard: {str(cost.get('discard',False)).lower()}, life: {cost['life']}, remove: {remove}, loyalty: {'None' if cost.get('loyalty') is None else 'Some('+str(cost['loyalty'])+')'} }}, targets: {rust_targets(targets)}, program: {rust_program(program)}, sorcery: {str(sorcery).lower()}, once_per_turn: {str(once).lower()} }}")
    return '&[' + ', '.join(rows) + ']'


def rust_program(program):
    return '&[' + ', '.join('Instruction::' + instruction for instruction in program) + ']'

def extract_replacements(card):
    """Whole mandatory battlefield replacement sentences, never name dispatch."""
    if (card.get('layout')!='normal' or not isinstance(card.get('oracle_text'),str)
            or fixed_cost(card.get('mana_cost','')) is None
            or spell_cost_options(card.get('mana_cost','')) is None):return card,[]
    def counter(filter_,kind=None,factor=1,add=0,effect=False):
        return ('ReplacementSpec::Counters { filter: ReplacementFilter::'+filter_+
                ', kind: '+('Some(CounterKind::'+kind+')' if kind else 'None')+
                f', factor: {factor}, add: {add}, effects_only: {str(effect).lower()}'+' }')
    fixed={
        'If an effect would create one or more tokens under your control, it creates twice that many of those tokens instead.':'ReplacementSpec::DoubleTokens { effects_only: true }',
        'If one or more tokens would be created under your control, twice that many of those tokens are created instead.':'ReplacementSpec::DoubleTokens { effects_only: false }',
        'If an effect would put one or more counters on a permanent you control, it puts twice that many of those counters on that permanent instead.':counter('Permanent',factor=2,effect=True),
        'If one or more counters would be put on a creature, Spacecraft, or Planet you control, twice that many of each of those kinds of counters are put on it instead.':counter('CreatureSpacecraftPlanet',factor=2),
        'If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on it instead.':counter('Creature','PlusOne',add=1),
        'If one or more +1/+1 counters would be put on a creature or Vehicle you control, that many plus one +1/+1 counters are put on it instead.':counter('CreatureOrVehicle','PlusOne',add=1),
    }
    residual=[];specs=[]
    for line in card['oracle_text'].splitlines():
        if line in fixed:specs.append(fixed[line]);continue
        if line=='If one or more tokens would be created under your control, those tokens plus a 1/1 green Frog creature token are created instead.':
            token=token_program('create a 1/1 green Frog creature token.')[0]
            specs.append('ReplacementSpec::AddToken('+re.search(r'token: (__TOKEN_[0-9a-f]+__)',token)[1]+')');continue
        if line=='If one or more artifact tokens would be created under your control, that many 5/5 red Dragon creature tokens with flying are created instead.':
            token=token_program('create a 5/5 red Dragon creature token with flying.')[0]
            specs.append('ReplacementSpec::ReplaceArtifactTokens('+re.search(r'token: (__TOKEN_[0-9a-f]+__)',token)[1]+')');continue
        residual.append(line)
    return dict(card,oracle_text='\n'.join(residual)),specs


def adventure_faces(card, mechanic="Adventure"):
    """Typed two-part physical card; the reminder is presentation, not its program."""
    faces = card.get('card_faces')
    if (card.get('layout') != 'adventure' or not isinstance(faces, list)
            or len(faces) != 2 or any(not isinstance(f, dict) for f in faces)
            or faces[1].get('type_line') not in ('Instant — '+mechanic, 'Sorcery — '+mechanic)):
        return None
    if (card.get('oracle_text') not in (None, '')
            or card.get('name') != ' // '.join(f.get('name','') for f in faces)
            or card.get('mana_cost') != ' // '.join(f.get('mana_cost','') for f in faces if f.get('mana_cost'))
            or card.get('type_line') != ' // '.join(f.get('type_line','') for f in faces)):
        return None
    result = []
    for position, face in enumerate(faces):
        if not all(isinstance(face.get(k), str) for k in ('name','mana_cost','type_line','oracle_text')):
            return None
        if position and spell_cost_options(face['mana_cost']) is None:
            # Existing scalar programs support one explicit X plus a fixed cost.
            # The complete scalar body must still compile before the pair admits.
            scalar_cost = face['mana_cost']
            if scalar_cost.count('{X}') != 1 or fixed_cost(scalar_cost.replace('{X}', '') or '{0}') is None:
                return None
        text = face['oracle_text']
        if position and mechanic == 'Adventure' and '(Then ' in text and not re.search(r' \(Then exile this card\. You may (?:cast the (?:creature|artifact|enchantment)|play the land) later from exile\.\)$', text):
            return None
        if position and mechanic == 'Omen':
            # Only the exact two published Omen reminders are presentation.
            text = re.sub(r' \(Then shuffle this card into its owner\'s library\.\)$| \(Also shuffle this card\.\)$', '', text)
        if not position and mechanic == 'Omen':
            text = re.sub(r'(?m)^(Ward (\{[0-9]+\})) \(Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays \2\.\)$', r'\1', text)
        if mechanic == 'Omen' and ('(' in text or ')' in text):
            return None
        if position and mechanic == 'Adventure':
            # CR715 supplies this completion for every Adventure, independently of
            # whether Oracle includes reminder text. No residual sentence is erased.
            text = re.sub(r' \(Then exile this card\. You may (?:cast the (?:creature|artifact|enchantment)|play the land) later from exile\.\)', '', text)
        # Only audited reminder text may pass to generic face templates, which
        # historically strip parentheticals. Unknown clauses are not reminders
        # merely because a data source puts them inside parentheses.
        text = role_reminder_text(text)
        checked = text.replace(
            'Food token. (It\'s an artifact with "{2}, {T}, Sacrifice this token: You gain 3 life.")',
            'Food token.')
        checked = checked.replace(
            "Menace (This creature can't be blocked except by two or more creatures.)", 'Menace')
        if '(' in checked or ')' in checked:
            return None
        normal = dict(face, layout='normal', oracle_id=card['oracle_id'],
                      oracle_text=text, keywords=[],
                      colors=[c for c in 'WUBRG' if c in face['mana_cost']])
        if position:
            normal['type_line'] = face['type_line'].split(' — ')[0]
        normal['_adventure_type_line'] = face['type_line']
        normal['_oracle_keywords'] = [k for k in card.get('keywords', []) if k.casefold() in text.casefold()]
        result.append(normal)
    return result


def saga_chapter_program(body):
    """Complete chapter sentences using shared immutable effects, without card-name gates."""
    text=normalize_duration(body[0].lower()+body[1:])
    if m:=re.fullmatch(r"exile the top (two|three|[0-9]+) cards of target opponent's library\. You may play those cards this turn\.",text):
        amount={'two':2,'three':3}.get(m[1])
        if amount is None:amount=int(m[1])
        if not 1<=amount<=65535:return None
        return 'Opponent',[f'ExileTargetLibraryPlay {{ amount: {amount} }}']
    if text=='destroy target artifact an opponent controls.':return 'ArtifactOpponent',['Destroy']
    if m:=re.fullmatch(r'this creature deals ([0-9]+) damage to target tapped creature an opponent controls\.',text):
        return ('CreatureOpponentTapped',[f'Damage({int(m[1])})']) if int(m[1])<=32767 else None
    if text=='each opponent sacrifices a creature of their choice and loses 3 life.':
        parsed=sacrifice_program('each opponent sacrifices a creature of their choice.')
        return 'None',[*parsed[1],'LoseLife(PlayerSet::Opponent, 3)']
    if text=="draw a card if there's a creature or Lesson card in your graveyard.":
        filters=[stat_count_filter('creature cards in your graveyard'),stat_count_filter('Lesson cards in your graveyard')]
        value='StatValue::CountUnion(&['+', '.join(filters)+'])'
        return 'None',['If { guard: StepGuard::Expression { value: '+value+', minimum: Some(1), maximum: None }, skip: 1 }','Draw(PlayerSet::Controller, 1)']
    if text=='put X +1/+1 counters on target creature you control, where X is one plus the number of Foods you control.':
        value='AmountValue::Stat(StatValue::Sum(&[StatValue::Fixed(1), '+stat_value('Foods you control')+']))'
        return 'CreatureYou',['PutCountersScaled { subject: Subject::Targets, kind: CounterKind::PlusOne, amount: '+value+' }']
    if m:=re.fullmatch(r'up to two target creatures each get ([+-][0-9]+)/([+-][0-9]+) and gain (.+) until end of turn\.',text):
        keywords=keyword_list(m[3]);power,toughness=int(m[1]),int(m[2])
        if keywords is None or max(abs(power),abs(toughness))>32767:return None
        return ('Creature',0,2),[grow_instruction('Targets',power,toughness),f'GrantKeywords {{ subject: Subject::Targets, keywords: Keywords({keywords}) }}']
    if text=='double the power and toughness of each creature you control until end of turn.':
        return 'None',['SnapshotDoubleCreatures('+creature_filter('creatures you control')+')']
    if text=='tap target creature an opponent controls. Put a stun counter on it.':
        return 'CreatureOpponent',['Tap',counter_instruction('Targets','a','stun')]
    if text=='draw a card for each tapped creature your opponents control.':
        filt=stat_count_filter('creatures you control').replace('relation: 0','relation: 1')
        return 'None',['DrawScaled(PlayerSet::Controller, AmountValue::Stat(StatValue::TappedCount('+filt+')))']
    parsed=targeted_program(body)
    if parsed:return parsed
    program=statement_program(body)
    return ('None',program) if program else None


def saga_body(card):
    """Whole single-face Saga only: exact reminder, all numbered effects and intrinsic text."""
    if card.get('layout') != 'saga' or 'Saga' not in card.get('type_line','').split(' — ')[-1].split():return None
    if spell_cost_options(card.get('mana_cost','')) is None or '{X}' in card.get('mana_cost',''):return None
    lines=card.get('oracle_text','').splitlines()
    if not lines:return None
    roman={'I':1,'II':2,'III':3,'IV':4,'V':5}
    header=re.fullmatch(r'\(As this Saga enters and after your draw step, add a lore counter\. Sacrifice after (I|II|III|IV|V)\.\)',lines[0])
    if not header:return None
    final=roman[header[1]];abilities=[];seen=set();intrinsic=[]
    for line in lines[1:]:
        match=re.fullmatch(r'([IV, ]+) — (.+)',line)
        if match:
            labels=match[1].split(', ')
            if any(label not in roman for label in labels):return None
            numbers=[roman[label] for label in labels]
            if numbers!=sorted(set(numbers)) or seen.intersection(numbers):return None
            seen.update(numbers)
            body=match[2]
            # A flavor word is part of this one complete chapter, never an executable clause.
            if ' — ' in body:
                flavor,body=body.split(' — ',1)
                if flavor not in {'Aerial Blast','Diamond Dust','Heavenly Strike','Judgment Bolt','Kerplunk','Oblivion','Pain','Slipstream','Stampede!','Wark'}:return None
            # Only remove authenticated reminder sentences from their exact instruction.
            # Unknown parenthesized text is executable residual, never harmless prose.
            reminders = {
                'Create a Treasure token. (It\'s an artifact with "{T}, Sacrifice this token: Add one mana of any color.")': 'Create a Treasure token.',
                'Create a Food token. (It\'s an artifact with "{2}, {T}, Sacrifice this token: You gain 3 life.")': 'Create a Food token.',
                'Mill three cards, then create a Food token. (It\'s an artifact with "{2}, {T}, Sacrifice this token: You gain 3 life.")': 'Mill three cards, then create a Food token.',
                'Tap target creature an opponent controls. Put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.)': 'Tap target creature an opponent controls. Put a stun counter on it.',
            }
            body=reminders.get(body,body)
            if '(' in body or ')' in body:return None
            body=body.replace('This Saga','This creature')
            parsed=saga_chapter_program(body)
            if parsed is None:return None
            targets,program=parsed
            abilities.extend((f'Chapter {{ number: {number} }}',program,targets) for number in numbers)
        else:
            if '(' in line or ')' in line:return None
            intrinsic.append(line)
    if not seen or max(seen)!=final or seen!=set(range(1,final+1)):return None
    types=card.get('type_line','').split(' — ')[0].split()
    if 'Creature'in types:
        residual=dict(card,layout='normal',oracle_text='\n'.join(intrinsic))
        parsed=creature_template(residual)
        if parsed is None:return None
        effect,keywords=parsed
    else:
        if intrinsic:return None
        effect,keywords='Permanent',0
    return effect,keywords,abilities


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
    standard_names = set(cards)
    adventure_pairs = {}
    omen_pairs = {}
    # Reuse the complete-body compiler for each genuine face, then admit only the
    # conjunction. This is not a normal-layout fallback for the physical parent.
    for parent, original in list(cards.items()):
        raw_faces = original.get("card_faces")
        mechanic = "Omen" if isinstance(raw_faces, list) and raw_faces and isinstance(raw_faces[-1], dict) and raw_faces[-1].get("type_line", "").endswith(" — Omen") else "Adventure"
        faces = adventure_faces(original, mechanic)
        if faces is None:
            continue
        payload = json.dumps(faces, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
        _, proof = compile_catalogue(dict(cards=faces, card_count=2,
            cards_sha256=hashlib.sha256(payload).hexdigest()), dict(banned_card_names=[]))
        if not all(next(r['implemented'] for r in proof['cards'] if r['name']==face['name']) for face in faces):
            continue
        # Initial closed face options cannot inherit another unimplemented cast
        # selector merely because each normal synthetic program compiles.
        blocked = ('Plot ', 'Warp ', 'Kicker ', 'Offspring ', 'Convoke', 'Bargain',
                   'Flashback ', 'Cycling ', 'Collect evidence', 'Job select')
        if any(any(line.startswith(blocked) for line in f['oracle_text'].splitlines()) for f in faces):
            continue
        # The first inset preflight supports its printed fixed mana cost. Do not
        # admit a newly parseable mandatory-cost or target-discount combination
        # until that whole choice/funding protocol is audited for the inset.
        if (extract_additional_cost(faces[1])[1] is not None
                or extract_target_cost_reduction(faces[1])[1] is not None
                or extract_cost_reductions(faces[1])[1]):
            continue
        key = ('@omen/' if mechanic == 'Omen' else '@adventure/') + original['oracle_id']
        cards[parent], cards[key] = faces
        (omen_pairs if mechanic == "Omen" else adventure_pairs)[parent] = key
    TOKEN_DEFS.clear()
    EMBLEM_DEFS.clear()
    inset_pairs = adventure_pairs | omen_pairs
    internal_faces = set(inset_pairs.values())
    legacy = {name: (effect, cost, color) for name, effect, cost, color in LEGACY}
    names = list(legacy) + sorted((standard_names | set(target['banned_card_names'])) - set(legacy))
    names += sorted(internal_faces)
    if len(names) > 65536:
        raise ValueError('catalogue exceeds u16 identity space')
    definitions = []
    color_masks = []
    exhaust_rows = []
    uncounterable_rows = []
    entry_suppression_rows = []
    opponent_entry_tapped_rows = []
    loyalty_rows = []
    supported = []
    coverage = []
    static_rows = []
    grant_rows = []
    dynamic_rows = []
    scaling_rows = []
    additional_rows = []
    flashback_rows = []
    plot_rows = []
    warp_rows = []
    vehicle_stats_rows = []
    x_rows = []
    x_entry_rows = []
    kicker_rows = []
    bargain_rows = []
    convoke_rows = []
    evidence_rows = []
    kicked_entry_rows = []
    counter_grant_rows = []
    offspring_rows = []
    offspring_copy_rows = []
    offspring_factories = {}
    hybrid_rows = []
    reduction_rows = []
    cast_timing_rows = []
    target_reduction_rows = []
    entry_counter_rows = []
    equipment_rows = []
    attached_subtype_rows = []
    attached_turn_rows = []
    aura_rows = []
    replacement_rows=[]
    for index, name in enumerate(names):
        original = cards.get(name, {})
        raw_graveyard_body = original.get('oracle_text', '')
        graveyard_route_surface = raw_graveyard_body
        graveyard_route_seen = False
        single_graveyard_seen = False
        while True:
            normalized_graveyard_surface = ' '.join(graveyard_route_surface.casefold().split())
            single_graveyard_seen |= 'single graveyard' in normalized_graveyard_surface
            graveyard_route_seen |= (single_graveyard_seen
                                     or 'exile up to two target cards' in normalized_graveyard_surface)
            stripped = re.sub(r'\([^()]*\)', '', graveyard_route_surface)
            if stripped == graveyard_route_surface:
                break
            graveyard_route_surface = stripped
        if (original.get('layout') == 'normal' and 'Vehicle' in original.get('type_line','').split(' — ')[-1].split()
                and all(original.get(axis, '') == '*' or original.get(axis, '').isdigit() for axis in ('power','toughness'))):
            pt = [0 if original[axis] == '*' else int(original[axis]) for axis in ('power','toughness')]
            if max(pt) <= 2**127-1:
                vehicle_stats_rows.append(f'        {index} => Some(Stats {{ power: {pt[0]}, toughness: {pt[1]} }}),')
        timing_surface = original.get('oracle_text', '')
        timing_seen = False
        scalar_mass_seen = False
        while True:
            # Inspect each erasure stage: an outer reminder can surround a
            # permission whose keywords themselves contain an inner reminder.
            timing_seen |= re.search(r'\b(?:cast|play)\b.*\bas though\b',
                                     ' '.join(timing_surface.split()), flags=re.I) is not None
            scalar_mass_seen |= re.search(r'\beach\s+non-|gets\s+-x/-x', ' '.join(timing_surface.split()), flags=re.I) is not None
            stripped = re.sub(r'\([^()]*\)', '', timing_surface)
            if stripped == timing_surface:break
            timing_surface = stripped
        # Detection never authenticates or changes the raw validation body.
        card, timing_grants = extract_cast_timing_grants(cards.get(name, {}))
        card, uncounterable = extract_uncounterable(card)
        card, entry_suppression = extract_entry_trigger_suppression(card)
        card, opponent_entry_tapped = extract_opponent_entry_tapped(card)
        card, cycling = cycling_abilities(card)
        card, flashback = extract_flashback(card)
        card, offspring = extract_offspring(card)
        card, kicker = extract_kicker(card)
        card, bargain = extract_bargain(card)
        card, convoke = extract_convoke(card)
        plot_seen = any(line.startswith('Plot ') for line in card.get('oracle_text','').splitlines())
        card, plot = extract_plot(card)
        warp_seen = any(line.startswith('Warp') for line in card.get('oracle_text','').splitlines())
        card, warp = extract_warp(card)
        card, replacements = extract_replacements(card)
        card, extra_counter_grants = extract_counter_grants(card) if kicker is None and offspring is None else (card,[])
        card, evidence = extract_evidence(card)
        card, additional = extract_additional_cost(card)
        card, target_reduction = extract_target_cost_reduction(card)
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
        job = job_select_template(card) if (plot is None and warp is None and not cycling and flashback is None and offspring is None and kicker is None and not bargain and not convoke and evidence is None and additional is None and target_reduction is None and not reductions and dynamic is None and not scaling and creature_entry == 'Never' and not entry_counters) else None
        equipment = job[0] if job else equipment_template(card)
        permanent = permanent_program_template(card)
        static = static_permanent_template(card)
        activations, enters = [], [0, 0, 0]
        abilities = []
        entry = 'Never'
        land = land_template(card)
        # Reserved IDs retain absent demonstration fixtures. Supplied facts must
        # compile generically: a familiar name authenticates no rules or costs.
        use_legacy = name in legacy and name not in cards
        if use_legacy:
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
                _, keywords, abilities, activations, enters, enchant_kinds, relation, power, toughness, granted = aura
                target = 'ArtifactYou' if enchant_kinds==16 else {-1:'Creature',0:'CreatureYou',1:'CreatureOpponent'}[relation]
                effect = f'Aura({rust_targets(target)})'
                aura_rows.append(f'        {index} => Some(AuraSpec {{ kinds: {enchant_kinds}, relation: {relation}, bonus: EquipmentSpec {{ stats: Stats {{ power: {power}, toughness: {toughness} }}, keywords: Keywords({granted}) }} }}),')
            elif equipment is not None:
                effect, keywords, abilities, activations, enters, power, toughness, granted = equipment
                equipment_rows.append(f'        {index} => Some(EquipmentSpec {{ stats: Stats {{ power: {power}, toughness: {toughness} }}, keywords: Keywords({granted}) }}),')
            elif permanent is not None:
                effect, keywords, abilities, activations, enters = permanent
            elif static is not None:
                effect, keywords, abilities, activations, enters, modifiers, grants = static
                if modifiers: static_rows.append(f'        {index} => &[{", ".join(modifiers)}],')
                if grants: grant_rows.append(f'        {index} => &[{", ".join(grants)}],')
        scalar=scalar_x_body(card) if not bargain and evidence is None and all(v is None for v in (flashback,offspring,kicker,additional,target_reduction)) and (not reductions or len(reductions)==1 and 'SpellReductionAmount::CreaturePowerCount { minimum: 4 }' in reductions[0]) else None
        if '{X}' in cost:
            if scalar is None: effect='Unsupported'
            else:
                effect,keywords,abilities,spell,x_entry=scalar
                activations=[];enters=[0,0,0]
                cost=cost.replace('{X}','') or '{0}'
        # Offspring copies do not yet clone kicker/counter-grant recipes. Such a
        # combined body must stay gated rather than admit an incomplete copy.
        kicker_body=fixed_kicker_body(card) if kicker is not None and offspring is None else None
        if kicker is not None:
            if kicker_body is None:effect='Unsupported'
            else:
                effect,keywords,abilities,spell,kicked_counters,counter_grants=kicker_body
                activations=[];enters=[0,0,0]
        if bargain:
            bargain_body=fixed_bargain_body(card) if offspring is None and kicker is None and additional is None and flashback is None and target_reduction is None else None
            if bargain_body is None:effect='Unsupported'
            else:
                effect,keywords,abilities,spell,bargain_reduction=bargain_body
                activations=[];enters=[0,0,0]
        evidence_body=fixed_evidence_body(card,evidence) if not (bargain or offspring is not None or kicker is not None or additional is not None or flashback is not None or target_reduction is not None) else None
        if evidence is not None or evidence_body is not None:
            if evidence_body is None:effect='Unsupported'
            else:
                effect,keywords,abilities,spell,activations=evidence_body
                enters=[0,0,0]
        if any(line.startswith('Saddle ') for line in card.get('oracle_text','').split('\n')):
            saddle=saddle_body(card) if evidence is None and not cycling and all(v is None for v in (flashback,offspring,kicker,additional,target_reduction)) and not bargain and not reductions and not dynamic and not scaling else None
            if saddle is None:effect='Unsupported'
            else:effect,keywords,abilities,activations,enters=saddle
        if any('endures ' in line for line in card.get('oracle_text', '').splitlines()):
            endure=endure_body(card) if plot is None and not convoke and not cycling and not bargain and evidence is None and all(v is None for v in (flashback,offspring,kicker,additional,target_reduction)) and not reductions and not dynamic and not scaling else None
            if endure is None:effect='Unsupported'
            else:effect,keywords,abilities,activations,enters=endure

        if any(line.startswith('Mobilize ') for line in card.get('oracle_text','').split('\n')):
            mobilize=mobilize_body(card) if not convoke and evidence is None and not cycling and all(v is None for v in (flashback,offspring,kicker,additional,target_reduction)) and not bargain and not reductions else None
            if mobilize is None:effect='Unsupported'
            else:effect,keywords,abilities,activations,enters=mobilize
        paid_body=actual_mana_body(card) if (plot is None and warp is None and not convoke and not cycling and not bargain and evidence is None and all(v is None for v in (flashback,offspring,kicker,additional,target_reduction)) and not reductions and not dynamic and not scaling) else None
        if paid_body is not None:
            effect,keywords,abilities,activations,enters=paid_body
        # Convoke applies only to complete fixed intrinsic bodies. No selector may
        # silently combine an unimplemented additional/alternative payment recipe.
        if convoke and (bargain or evidence is not None or cycling or any(v is not None for v in (flashback,offspring,kicker,additional,target_reduction)) or '{X}' in card.get('mana_cost','') or any(line.startswith(('Saddle ', 'Mobilize ')) for line in card.get('oracle_text','').splitlines())):
            effect='Unsupported'
        if plot_seen and (plot is None or warp_seen or convoke or evidence is not None or cycling
                or any('endures ' in line for line in card.get('oracle_text','').splitlines())
                or any(line.startswith(('Saddle ', 'Mobilize ')) for line in card.get('oracle_text','').splitlines())
                or any(v is not None for v in (flashback,offspring,kicker,additional,target_reduction,scalar))
                or bargain or reductions
                or spell_cost_options(card.get('mana_cost','')) is None):
            effect='Unsupported'
        if warp_seen and (warp is None or plot_seen or convoke or evidence is not None or cycling
                or any('endures ' in line for line in card.get('oracle_text','').splitlines())
                or any(line.startswith(('Saddle ', 'Mobilize ')) for line in card.get('oracle_text','').splitlines())
                or any(v is not None for v in (flashback,offspring,kicker,additional,target_reduction,scalar))
                or bargain or reductions or spell_cost_options(card.get('mana_cost','')) is None):
            effect='Unsupported'
        crew_seen = any(line.startswith('Crew ') for line in original.get('oracle_text','').splitlines())
        if crew_seen:
            crew = crew_vehicle_body(card) if (not convoke and not bargain and evidence is None
                    and not plot_seen and not warp_seen
                    and all(v is None for v in (flashback,offspring,kicker,additional,target_reduction,scalar))
                    and not reductions and dynamic is None and not scaling and not entry_counters
                    and creature_entry == 'Never') else None
            if crew is None: effect='Unsupported'
            else: effect,keywords,abilities,activations,enters,dynamic,scaling=crew
        if original.get("layout") == "saga":
            saga=saga_body(original)
            if saga is None: effect="Unsupported"
            else:
                effect,keywords,abilities=saga
                activations=[];enters=[0,0,0]
        if not role_consumer_reminders_valid(original):
            effect='Unsupported'
        if attachment_ward_route(original) and not attachment_ward_reminders_valid(original):
            effect='Unsupported'
        # New split-search consumers are whole bodies. The older permanent
        # templates strip generic parentheticals, so reject every unaudited
        # clause before that historical behavior can hide a new instruction.
        raw_split_text = original.get('oracle_text', '')
        if (split_search_route_text(raw_split_text)
                and ('(' in raw_split_text or ')' in raw_split_text)):
            effect = 'Unsupported'
        # Later fixed-cost selectors must never revive a rejected variable-cost body.
        if ('{X}' in card.get('mana_cost', '') or scalar_mass_seen) and scalar is None:
            effect = 'Unsupported'
        if replacements and (plot_seen or convoke or cycling or bargain or evidence is not None
                or any(v is not None for v in (offspring,kicker,additional,flashback,target_reduction,scalar))
                or reductions or dynamic or scaling or creature_entry!='Never' or entry_counters
                or any(line.startswith(('Saddle ','Mobilize ')) or 'endures ' in line for line in card.get('oracle_text','').splitlines())):
            effect='Unsupported'
        if effect != 'Unsupported':
            if creature_entry != 'Never':entry=creature_entry
            activations = [*activations, *cycling]
            if len(activations) > 64: effect = 'Unsupported'; activations = []
        # A later generic selector cannot revive rejected intrinsic reminder text.
        if not adventure_event_reminders_valid(original):
            effect = 'Unsupported'
        if not recovery_reminders_valid(original):
            effect = 'Unsupported'
        if not intrinsic_reminders_valid(original):
            effect = 'Unsupported'
        if not target_exclusion_reminders_valid(original):
            effect='Unsupported'
        if not enchanted_draw_reminders_valid(original):
            effect = 'Unsupported'
        # Rejected timing syntax/source shapes must not be revived by legacy selectors.
        if timing_seen and not timing_grants:
            effect = 'Unsupported'
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
        # Both routes must authenticate their complete unchanged source bodies.
        if (single_graveyard_seen
                and any(char in raw_graveyard_body for char in '()')):
            effect = 'Unsupported'
        if timing_grants and not (effect.startswith(('Creature(', 'Aura(')) or effect in ('Permanent', 'Land')):
            effect = 'Unsupported'
        offspring_key=None
        if offspring is not None and effect!='Unsupported':
            # Fixed copiable values only. The definition below preserves complete
            # immutable abilities; identity is separate from the printed name.
            factory={'name':name,'offspring_source':cards[name]['oracle_id'],'power':1,'toughness':1,'keywords':keywords,'subtypes':card.get('type_line','').split(' — ')[-1],'colors':sum(1 << 'WUBRG'.index(color) for color in card.get('colors',[])),'extra_type':None}
            offspring_key=hashlib.sha256(json.dumps(factory,sort_keys=True).encode()).hexdigest()[:24]
            TOKEN_DEFS[offspring_key]=factory
            abilities=[('SelfOffspring',[f'CreateTokens {{ token: __TOKEN_{offspring_key}__, amount: 1, tapped: false }}'],'None'),*abilities]
        # The compact mana action is an ABI choice, never an admission route.
        # A supplied definition reaches it only after complete generic compilation
        # proves the exact same one-mana tap ability, with no additional ability.
        if name in legacy and name in cards and effect != 'Unsupported':
            old_effect, _, old_color = legacy[name]
            if old_color is not None and effect == old_effect and not abilities and len(activations) == 1:
                activation = activations[0]
                cost_spec, targets, instructions, sorcery_only, once = activation
                expected_cost = dict(activation_cost('{T}'), exhaust=False)
                mana = [int(i == old_color) for i in range(6)]
                if (cost_spec == expected_cost and targets == 'None'
                        and instructions == [f'AddMana(ManaProduction::Fixed({mana}))']
                        and not sorcery_only and not once):
                    color, activations = old_color, []
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
            if timing_grants:cast_timing_rows.append(f'        {index} => &[{", ".join(timing_grants)}],')
            if replacements:replacement_rows.append(f'        {index} => &[{", ".join(replacements)}],')
            if name not in internal_faces: supported.append(index)
            if job is not None:
                attached_subtype_rows.append(f'        {index} => Some({json.dumps(job[1])}),')
                if job[2]:attached_turn_rows.append(f'        {index} => Keywords({job[2]}),')
                if job[3]:scaling_rows.append(f'        {index} => &['+', '.join(job[3])+'],')
            if evidence is not None: evidence_rows.append(f'        {index} => Some(({evidence[0]}, {evidence[1]})),')
            if convoke: convoke_rows.append(str(index))
            if bargain: bargain_rows.append(f'        {index} => Some({bargain_reduction}),')
            if scalar is not None:
                x_rows.append(f'        {index} => 1,')
                if x_entry:x_entry_rows.append(str(index))
            if kicker is not None:
                kicker_rows.append(f'        {index} => Some({kicker}),')
                if kicked_counters:kicked_entry_rows.append(f'        {index} => {kicked_counters},')
                if counter_grants:counter_grant_rows.append(f'        {index} => &[(CounterKind::PlusOne, Keywords(256))],')
            if offspring is not None:
                offspring_rows.append(f'        {index} => Some({offspring}),')
                offspring_copy_rows.append(f'        {index} => Some(__TOKEN_{offspring_key}__),')
            if flashback is not None: flashback_rows.append(f'        {index} => Some({flashback}),')
            if extra_counter_grants: counter_grant_rows.append(f'        {index} => &[{", ".join(extra_counter_grants)}],')
            if plot is not None: plot_rows.append(f'        {index} => Some({plot}),')
            if warp is not None: warp_rows.append(f'        {index} => Some({warp}),')
            if entry_suppression: entry_suppression_rows.append(f'        {index} => {entry_suppression},')
            if opponent_entry_tapped: opponent_entry_tapped_rows.append(f'        {index} => {opponent_entry_tapped},')
            if entry_counters: entry_counter_rows.append(f'        {index} => &[{", ".join(entry_counters)}],')
            if target_reduction: target_reduction_rows.append(f'        {index} => Some(&{target_reduction}),')
            if reductions: reduction_rows.append(f'        {index} => &[{", ".join(reductions)}],')
            if additional is not None: additional_rows.append(f'        {index} => Some({additional}),')
        basic = (not card and name in {'Plains', 'Island', 'Swamp', 'Mountain', 'Forest'}) or card.get('type_line', '').split(' — ')[0] == 'Basic Land'
        costs = spell_cost_options(cost) or [([0] * 6, 0)]
        colored, generic = costs[0]
        if len(costs)>1:
            hybrid_rows.append(f'        {index} => &['+', '.join(f'ManaCost {{ colored: {mana}, generic: {n} }}' for mana,n in costs[1:])+'],')
        deck_limit = 'None' if basic or name in UNLIMITED else 'Some(4)'
        quote = lambda value: json.dumps(value, ensure_ascii=False)
        oracle_id = f"Some({quote(card['oracle_id'])})" if card else 'None'
        type_line = card.get('_adventure_type_line', card.get('type_line', {'Grizzly Bears':'Creature — Bear','Goblin Piker':'Creature — Goblin Warrior','Hill Giant':'Creature — Giant'}.get(name, 'Basic Land' if basic else 'Creature' if kind & 1 else 'Instant' if kind & 8 else 'Sorcery' if kind & 4 else '')))
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
        oracle_keywords = '&[' + ', '.join(quote(word) for word in card.get('_oracle_keywords', card.get('keywords', []))) + ']'
        if effect != 'Unsupported':
            if dynamic is not None:dynamic_rows.append(f'        {index} => Some({dynamic}),')
            if scaling:scaling_rows.append(f'        {index} => &['+', '.join(scaling)+'],')
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {quote(card.get('name', name))}, oracle_id: {oracle_id}, mana_cost: {quote(card.get("mana_cost",cost) if scalar is not None else cost)}, type_line: {quote(type_line)}, oracle_keywords: {oracle_keywords}, standard_legal: {str(name in standard_names).lower()}, legendary: {str(card.get('type_line', '').startswith('Legendary ')).lower()}, implemented: {str(effect != 'Unsupported').lower()}, keywords: Keywords({keywords}), deck_limit: {deck_limit} }}, effect: Effect::{effect}, triggers: {triggers_rust}, activated: {rust_activations(activations)}, enters: Counters {{ plus_one: {enters[0]}, minus_one: {enters[1]}, stun: {enters[2]}, loyalty: 0, lore: 0 }}, entry: EntryTapped::{entry}, kind: {kind}, basic: {str(basic).lower()}, mana_color: {'None' if color is None else f'Some({color})'}, colored: {colored}, generic: {generic} }},")
        if offspring_key is not None:
            offspring_factories[offspring_key]=(index,definitions[-1],color_mask,card.get('type_line','').split(' — ')[-1])
        if uncounterable and effect != 'Unsupported': uncounterable_rows.append(str(index))
        exhaust_mask=sum(1<<i for i,a in enumerate(activations) if a[0].get('exhaust',False))
        if exhaust_mask and effect!='Unsupported':exhaust_rows.append(f'        {index} => {exhaust_mask},')
        coverage.append({'name': name, 'standard_legal': name in standard_names, 'implemented': effect != 'Unsupported',
                         'implementation': 'bargain_template' if bargain and effect != 'Unsupported' else 'explicit' if use_legacy else 'land_template' if land else 'creature_template' if template else 'trigger_template' if trigger_template else 'modal_template' if modal else 'spell_template' if spell else 'aura_template' if aura else 'equipment_template' if equipment else 'permanent_template' if permanent else 'static_template' if static else None})

    # Only factories referenced by wholly accepted cards enter the executable catalogue.
    used_tokens = sorted(set(re.findall(r'__TOKEN_([0-9a-f]+)__', '\n'.join(definitions+replacement_rows))))
    token_base = len(names)
    token_infos = []
    role_rows = []
    for offset, key in enumerate(used_tokens):
        factory = TOKEN_DEFS[key]
        index = token_base + offset
        name = '@token/' + key
        names.append(name)

        if key in offspring_factories:
            source_index,definition,color,subtypes=offspring_factories[key]
            definition=re.sub(r'effect: Effect::Creature\([^)]*\)', 'effect: Effect::Creature(1, 1)',definition,count=1)
            definition=re.sub(r'oracle_id: Some\([^)]*\)', 'oracle_id: None',definition,count=1)
            definition=definition.replace('standard_legal: true','standard_legal: false').replace('deck_limit: Some(4)','deck_limit: None')
            definitions.append(definition)
            token_infos.append(f"    TokenInfo {{ factory_identity: {json.dumps(name)}, name: {json.dumps(factory['name'])}, subtypes: {json.dumps(subtypes)}, colors: {color} }},")
            color_masks.append(color)
            # Clone every compiled property table for the immutable copy; no
            # mutable copy/type-changing engine or runtime card-name dispatch.
            for table in [static_rows,grant_rows,dynamic_rows,scaling_rows,additional_rows,hybrid_rows,reduction_rows,cast_timing_rows,target_reduction_rows,entry_counter_rows,entry_suppression_rows,opponent_entry_tapped_rows,offspring_rows]:
                prefix=f'        {source_index} =>'
                table.extend(row.replace(prefix,f'        {index} =>',1) for row in list(table) if row.startswith(prefix))
            offspring_copy_rows.append(f'        {index} => Some(__TOKEN_{key}__),')
            marker='__TOKEN_'+key+'__'
            definitions=[row.replace(marker,f'Card({index})') for row in definitions]
            offspring_copy_rows=[row.replace(marker,f'Card({index})') for row in offspring_copy_rows]
            continue

        predefined = factory.get('predefined')
        if factory.get('role'):
            role = factory['role']
            effect, token_abilities = 'Permanent', []
            type_line, kind = 'Enchantment — Aura Role', 32
            bonus = 0 if role in ('Cursed','Young Hero') else 1
            keywords = KEYWORDS['trample'] if role == 'Monster' else 0
            aura_rows.append(f'        {index} => Some(AuraSpec {{ kinds: 1, relation: -1, bonus: EquipmentSpec {{ stats: Stats {{ power: {bonus}, toughness: {bonus} }}, keywords: Keywords({keywords}) }} }}),')
            role_rows.append(f'        {index} => Some(RoleKind::{role.replace(" ", "")}),')
        elif predefined:
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
        token_triggers='&[]'
        role = factory.get('role')
        role_triggers = []
        if role == 'Royal': role_triggers = attachment_grants('ward {1}')[1]
        elif role == 'Sorcerer': role_triggers = [('GrantedAttacks',['Scry(1)'],'None')]
        elif role == 'Young Hero': role_triggers = [('InterveningIf { guard: StepGuard::SourceToughnessAtMost(3), event: &TriggerCondition::GrantedAttacks }',['PutCounters { subject: Subject::Source, kind: CounterKind::PlusOne, amount: 1 }'],'None')]
        elif role == 'Wicked': role_triggers = [('SelfDies',['LoseLife(PlayerSet::Opponent, 1)'],'None')]
        if role_triggers:
            token_triggers='&['+', '.join('TriggeredAbility { condition: TriggerCondition::'+condition+', targets: '+rust_targets(rule)+', program: '+rust_program(program)+' }' for condition,program,rule in role_triggers)+']'
        if factory.get('landfall_growth'):
            p,t=factory['landfall_growth']
            token_triggers='&[TriggeredAbility { condition: TriggerCondition::LandEntersYou, targets: TargetPlan::Single(TargetSpec::NONE), program: '+rust_program([grow_instruction('Source',p,t)])+' }]'
        definitions.append(f"    Definition {{ info: CardInfo {{ name: {json.dumps(name)}, oracle_id: None, mana_cost: \"\", type_line: {json.dumps(type_line, ensure_ascii=False)}, oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: Keywords({factory['keywords']}), deck_limit: None }}, effect: Effect::{effect}, triggers: {token_triggers}, activated: {rust_activations(token_abilities)}, enters: Counters {{ plus_one: 0, minus_one: 0, stun: 0, loyalty: 0, lore: 0 }}, entry: EntryTapped::Never, kind: {kind}, basic: false, mana_color: None, colored: [0; 6], generic: 0 }},")
        token_infos.append(f"    TokenInfo {{ factory_identity: {json.dumps(name)}, name: {json.dumps(factory['name'])}, subtypes: {json.dumps(factory['subtypes'])}, colors: {factory['colors']} }},")
        color_masks.append(factory['colors'])
        marker = '__TOKEN_' + key + '__'
        definitions = [definition.replace(marker, f'Card({index})') for definition in definitions]
    token_indices={key:token_base+offset for offset,key in enumerate(used_tokens)}
    def replace_token_markers(row):
        return re.sub(r'__TOKEN_([0-9a-f]+)__',lambda match:f'Card({token_indices[match[1]]})',row)
    definitions=[replace_token_markers(row) for row in definitions]
    offspring_copy_rows=[replace_token_markers(row) for row in offspring_copy_rows]
    replacement_rows=[replace_token_markers(row) for row in replacement_rows]
    used_emblems=sorted(set(re.findall(r'__EMBLEM_([0-9a-f]+)__', '\n'.join(definitions))))
    emblem_base=len(names)
    emblem_infos=[]
    for key in used_emblems:
        index=len(names);name='@emblem/'+key;names.append(name);color_masks.append(0)
        body=EMBLEM_DEFS[key]
        triggers='&[TriggeredAbility { condition: TriggerCondition::LifeGained { first_only: false, own_turn: false }, targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player { opponent_only: true })), program: &[Instruction::LoseGainedLife(PlayerSet::Targeted)] }]'
        definitions.append(f'Definition {{ info: CardInfo {{ name: {json.dumps(name)}, oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: Keywords(0), deck_limit: None }}, effect: Effect::Emblem, triggers: {triggers}, activated: &[], enters: Counters {{ plus_one: 0, minus_one: 0, stun: 0, loyalty: 0, lore: 0 }}, entry: EntryTapped::Never, kind: 0, basic: false, mana_color: None, colored: [0; 6], generic: 0 }},')
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
    aliases = {name:index for index,name in enumerate(names)}
    for parent, face in inset_pairs.items():
        aliases[cards[parent]['name']] = names.index(parent)
        aliases[cards[face]['name']] = names.index(face)
    lines[lines.index(next(v for v in lines if 'static NAME_INDEX:' in v))] = f'pub(super) static NAME_INDEX: [(&str, u16); {len(aliases)}] = ['
    lines += [f'    ({json.dumps(name, ensure_ascii=False)}, {index}),' for name,index in sorted(aliases.items())]
    lines += ['];', f'pub(super) const TOKEN_BASE: usize = {token_base};', f'pub(super) const EMBLEM_BASE: usize = {emblem_base};', '#[rustfmt::skip]', f'pub(super) static EMBLEMS: [EmblemInfo; {len(emblem_infos)}] = [', *emblem_infos, '];',
              '#[rustfmt::skip]', f'pub(super) static TOKENS: [TokenInfo; {len(token_infos)}] = [', *token_infos, '];',
              '#[allow(non_upper_case_globals)]', 'impl Card {', '#[rustfmt::skip]',
              f'    pub const ALL: [Card; {len(supported)}] = [{", ".join(f"Card({i})" for i in supported)}];']
    # Preserve existing Rust constants and JSON tokens. New callers use English names.
    for index, (name, _, _, _) in enumerate(LEGACY):
        lines.append(f'    pub const {name.replace(" ", "")}: Card = Card({index});')
    lines += ['}', '#[rustfmt::skip]', "pub(super) fn static_modifiers(card: Card) -> &'static [StaticModifier] {", '    match card.0 {', *static_rows, '        _ => &[],', '    }', '}', '#[rustfmt::skip]', 'pub(super) fn equipment(card: Card) -> Option<EquipmentSpec> {', '    match card.0 {', *equipment_rows, '        _ => None,', '    }', '}', '#[rustfmt::skip]', 'pub(super) fn aura(card: Card) -> Option<AuraSpec> {', '    match card.0 {', *aura_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn role(card: Card) -> Option<RoleKind> {', '    match card.0 {', *role_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn plot_cost(card: Card) -> Option<ManaCost> {', '    match card.0 {', *plot_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn adventure_identity(card: Card) -> Option<&'static str> {", '    match card.0 {',
              *(f'        {names.index(parent)} => Some({json.dumps(parent,ensure_ascii=False)}),' for parent in inset_pairs), '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn adventure_spell(card: Card) -> Option<Card> {', '    match card.0 {',
              *(f'        {names.index(parent)} => Some(Card({names.index(face)})),' for parent,face in adventure_pairs.items()), '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn adventure_parent(card: Card) -> Option<Card> {', '    match card.0 {',
              *(f'        {names.index(face)} => Some(Card({names.index(parent)})),' for parent,face in adventure_pairs.items()), '        _ => None,', '    }', '}', '']
    for method, pairs in [('omen', omen_pairs)]:
        lines += ['#[rustfmt::skip]', f'pub(super) fn {method}_spell(card: Card) -> Option<Card> {{', '    match card.0 {',
                  *(f'        {names.index(parent)} => Some(Card({names.index(face)})),' for parent,face in pairs.items()), '        _ => None,', '    }', '}', '']
        lines += ['#[rustfmt::skip]', f'pub(super) fn {method}_parent(card: Card) -> Option<Card> {{', '    match card.0 {',
                  *(f'        {names.index(face)} => Some(Card({names.index(parent)})),' for parent,face in pairs.items()), '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn warp_cost(card: Card) -> Option<ManaCost> {', '    match card.0 {', *warp_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn vehicle_stats(card: Card) -> Option<Stats> {', '    match card.0 {', *vehicle_stats_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn replacement_effects(card: Card) -> &'static [ReplacementSpec] {", '    match card.0 {', *replacement_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn attached_subtype(card: Card) -> Option<&'static str> {", '    match card.0 {', *attached_subtype_rows, '        _ => None,', '    }', '}', '#[rustfmt::skip]', 'pub(super) fn attached_turn_keywords(card: Card) -> Keywords {', '    match card.0 {', *attached_turn_rows, '        _ => Keywords::NONE,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn dynamic_stats(card: Card) -> Option<DynamicStats> {', '    match card.0 {', *dynamic_rows, '        _ => None,', '    }', '}', '#[rustfmt::skip]', "pub(super) fn scaling_modifiers(card: Card) -> &'static [ScalingModifier] {", '    match card.0 {', *scaling_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn entry_trigger_suppression(card: Card) -> u16 {', '    match card.0 {', *entry_suppression_rows, '        _ => 0,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn opponent_entry_tapped(card: Card) -> u16 {', '    match card.0 {', *opponent_entry_tapped_rows, '        _ => 0,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn keyword_grants(card: Card) -> &'static [KeywordGrant] {", '    match card.0 {', *grant_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn additional_cost(card: Card) -> Option<AdditionalCost> {', '    match card.0 {', *additional_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn collect_evidence(card: Card) -> Option<(u16, u32)> {', '    match card.0 {', *evidence_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn bargain_reduction(card: Card) -> Option<u32> {', '    match card.0 {', *bargain_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn x_symbols(card: Card) -> u8 {', '    match card.0 {', *x_rows, '        _ => 0,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn x_entry_counters(card: Card) -> bool {', '    '+('matches!(card.0, '+' | '.join(x_entry_rows)+')' if x_entry_rows else 'false'), '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn kicker_cost(card: Card) -> Option<ManaCost> {', '    match card.0 {', *kicker_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn kicked_entry_counters(card: Card) -> u16 {', '    match card.0 {', *kicked_entry_rows, '        _ => 0,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn counter_keyword_grants(card: Card) -> &'static [(CounterKind, Keywords)] {", '    match card.0 {', *counter_grant_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn offspring_cost(card: Card) -> Option<ManaCost> {', '    match card.0 {', *offspring_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn offspring_copy(card: Card) -> Option<Card> {', '    match card.0 {', *offspring_copy_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn flashback_cost(card: Card) -> Option<ManaCost> {', '    match card.0 {', *flashback_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub const CREATURE_TYPES: &[&str] = &[', *('    '+json.dumps(name)+',' for name in creature_types), '];', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn target_cost_reduction(card: Card) -> Option<&'static TargetSpellReduction> {", '    match card.0 {', *target_reduction_rows, '        _ => None,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn cast_timing_grants(card: Card) -> &'static [CastFilter] {", '    match card.0 {', *cast_timing_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn cost_reductions(card: Card) -> &'static [SpellReduction] {", '    match card.0 {', *reduction_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn cost_alternatives(card: Card) -> &'static [ManaCost] {", '    match card.0 {', *hybrid_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', "pub(super) fn entry_counter_rules(card: Card) -> &'static [EntryCounterRule] {", '    match card.0 {', *entry_counter_rows, '        _ => &[],', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn starting_loyalty(card: Card) -> u64 {', '    match card.0 {', *loyalty_rows, '        _ => 0,', '    }', '}', '']
    lines += ['#[rustfmt::skip]', 'pub(super) fn exhaust_mask(card: Card) -> u64 {', '    match card.0 {', *exhaust_rows, '        _ => 0,', '    }', '}', '']
    convoke_match = 'matches!(card.0, ' + ' | '.join(convoke_rows) + ')' if convoke_rows else 'false'
    lines += ['#[rustfmt::skip]', 'pub(super) fn has_convoke(card: Card) -> bool {', '    '+convoke_match, '}', '']
    protection = 'matches!(card.0, ' + ' | '.join(uncounterable_rows) + ')' if uncounterable_rows else 'false'
    lines += ['#[rustfmt::skip]', 'pub(super) fn uncounterable(card: Card) -> bool {', '    ' + protection, '}', '']
    return '\n'.join(lines), {'schema': 1, 'catalogue_version': version, 'standard_legal_cards': len(standard_names),
        'implemented_standard_cards': sum(row['implemented'] and row['standard_legal'] for row in coverage),
        'implemented_total': len(supported), 'token_definitions': len(used_tokens), 'emblem_definitions': len(used_emblems), 'cards': [r for r in coverage if r['name'] not in internal_faces], 'adventure_definitions': len(adventure_pairs), 'omen_definitions': len(omen_pairs)}


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
