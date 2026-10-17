"""Source-preserving continuous-effect groups for the characteristic evaluator.

This projection does not authorize text or admit cards. Callers must compile and
raw-authenticate the COMPLETE body first; fragments retain the original line.
The current module is not yet connected to catalogue emission.
"""
from dataclasses import dataclass
from enum import IntEnum
import re


class Layer(IntEnum):
    TYPE = 4
    COLOR = 5
    ABILITY = 6
    CDA = 70
    BASE = 71
    MODIFY = 72
    RULE = 100


@dataclass(frozen=True)
class Fragment:
    kind: str
    layer: Layer
    payload: object


@dataclass(frozen=True)
class StaticGroup:
    line: int
    raw: str
    fragments: tuple[Fragment, ...]

    @property
    def first_layer(self):
        return min(f.layer for f in self.fragments)


def continuous_groups(card, parser):
    """Project immutable groups without changing the caller's factory registry."""
    tokens, emblems = dict(parser.TOKEN_DEFS), dict(parser.EMBLEM_DEFS)
    try:
        return _continuous_groups(card, parser)
    finally:
        parser.TOKEN_DEFS.clear()
        parser.TOKEN_DEFS.update(tokens)
        parser.EMBLEM_DEFS.clear()
        parser.EMBLEM_DEFS.update(emblems)


def _continuous_groups(card, parser):
    """Preserve clause identity while existing extractors split its fragments.

    Every payload is the existing immutable executable recipe. The projection
    neither erases raw reminders for authentication nor changes admission.
    """
    if isinstance(card.get('card_faces'), list):
        faces = card['card_faces']
        mechanic = 'Omen' if faces and faces[-1].get('type_line', '').endswith(' — Omen') else 'Adventure'
        parsed = parser.adventure_faces(card, mechanic)
        return continuous_groups(parsed[0], parser) if parsed is not None else ()
    raw = card.get('oracle_text')
    if not isinstance(raw, str):
        return ()
    lines = raw.splitlines()
    if len(lines) > 65535:
        raise ValueError('static source line identity exhausted')
    equipment = 'Equipment' in card.get('type_line', '').split(' — ')[-1].split()
    aura = card.get('type_line', '').split(' — ')[-1] == 'Aura'
    # Use the card's actual context/costs, never a fabricated admission body.
    equips = [s for s in lines if re.search(r'(?:^| — )Equip(?: |—)', s)]
    enchant = [s for s in lines if s.startswith('Enchant ')]
    job_header = lines[:1] if lines[:1] and lines[0] in ('Job select', parser.JOB_REMINDER) else []
    vehicle = 'Vehicle' in card.get('type_line', '').split(' — ')[-1].split()
    kicked, kicker = parser.extract_kicker(card)
    kicker_body = parser.fixed_kicker_body(kicked) if kicker is not None else None
    counter_reminder = " (It can deal excess combat damage to the player or planeswalker it's attacking.)"
    groups = []
    for index, original in enumerate(lines):
        fragments = []
        one = dict(card, oracle_text=original)
        if vehicle and original.startswith(("This Vehicle's power", "This Vehicle's toughness")):
            # Exact deictic conversion shared with the whole Crew parser. This
            # is a characteristic clause, never a fabricated creature object.
            one = dict(one, type_line='Artifact Creature', oracle_text=original.replace('This Vehicle', 'This creature'))
        if (kicker_body is not None and kicker_body[-1]
                and original.startswith('Each creature you control with a +1/+1 counter on it has ')
                and original.endswith(counter_reminder)):
            # The existing COMPLETE kicker body proves this exact counter-grant
            # reminder slot. Keep its unmodified raw line as the group identity.
            one = dict(one, oracle_text=original[:-len(counter_reminder)])
        # Changeling contributes types before ability removal. Keep that CDA
        # distinct from separate later static abilities on this same card.
        surface = original
        if '(' in surface or ')' in surface:
            surface = surface.split(' (', 1)[0] if parser.intrinsic_reminder_line_valid(surface) else ''
        words = {word.strip().casefold() for word in surface.split(',') if word.strip()}
        if words and words <= parser.INTRINSIC_KEYWORDS.keys():
            mask = sum(parser.INTRINSIC_KEYWORDS[word] for word in words)
            changeling = parser.INTRINSIC_KEYWORDS['changeling']
            if mask & changeling:
                fragments.append(Fragment('changeling', Layer.TYPE, changeling))
            if mask & ~changeling:
                fragments.append(Fragment('intrinsic_keywords', Layer.ABILITY, mask & ~changeling))
        # These extractors may rewrite a combined stats/keyword clause. Pass the
        # residual onward WITH the same origin, never assign a new synthetic ID.
        residual, live = parser.extract_live_static_stats(one)
        for effect in live:
            if re.search(r'(?:power|toughness): Some\(', effect):
                fragments.append(Fragment('live_static_base', Layer.BASE, effect))
            if any(int(value) for value in re.findall(r'bonus_(?:power|toughness): (-?[0-9]+)', effect)):
                fragments.append(Fragment('live_static_bonus', Layer.MODIFY, effect))
            if int(re.search(r'keywords: Keywords\(([0-9]+)\)', effect)[1]):
                fragments.append(Fragment('live_static_keywords', Layer.ABILITY, effect))
            if 'cannot_block_subtype: Some(' in effect:
                fragments.append(Fragment('live_static_block_restriction', Layer.RULE, effect))
        residual, dynamic, scaling = parser.extract_stat_rules(residual)
        if dynamic is not None:
            fragments.append(Fragment('dynamic_stats', Layer.CDA, dynamic))
        fragments.extend(Fragment('scaling_modifiers', Layer.MODIFY, s) for s in scaling)
        residual, base = parser.extract_base_stat_setter(residual)
        if base is not None:
            fragments.append(Fragment('base_stat_setter', Layer.BASE, base))
        residual, enchanted = parser.extract_enchanted_modifiers(residual)
        fragments.extend(Fragment('enchanted_modifiers', Layer.MODIFY, s) for s in enchanted)
        residual, counter_grants = parser.extract_counter_grants(residual)
        fragments.extend(Fragment('counter_keyword_grants', Layer.ABILITY, s) for s in counter_grants)
        static = parser.static_permanent_template(residual)
        if static is not None:
            fragments.extend(Fragment('static_modifiers', Layer.MODIFY, s) for s in static[-2])
            fragments.extend(Fragment('keyword_grants', Layer.ABILITY, s) for s in static[-1])
        # Rule-modifying static providers do not start in an earlier
        # characteristic layer. Preserve their independent raw ability group.
        for extract, kind, plural in (
                (parser.extract_replacements, 'replacement_effects', True),
                (parser.extract_cast_timing_grants, 'cast_timing_grants', True),
                (parser.extract_cost_reductions, 'cost_reductions', True),
                (parser.extract_target_cost_reduction, 'target_cost_reduction', False),
                (parser.extract_combat_restriction, 'combat_restriction', False),
                (parser.extract_entry_trigger_suppression, 'entry_trigger_suppression', False),
                (parser.extract_opponent_entry_tapped, 'opponent_entry_tapped', False)):
            _, value = extract(one)
            if value is not None and value != 0:
                fragments.extend(Fragment(kind, Layer.RULE, row) for row in (value if plural else [value]))
        if job_header and original == 'During your turn, equipped creature has flying.':
            whole_job = parser.job_select_template(card)
            if whole_job is not None and whole_job[2]:
                fragments.append(Fragment('attached_turn_keywords', Layer.ABILITY, whole_job[2]))
        if equipment and original not in equips and original not in job_header:
            if job_header and 'in addition to its other types' in original:
                # Job parsing allocates immutable token factories. Projection
                # must not change the caller's factory registry or its order.
                tokens = dict(parser.TOKEN_DEFS)
                try:
                    job = parser.job_select_template(dict(card, oracle_text='\n'.join([*job_header, original, *equips])))
                finally:
                    parser.TOKEN_DEFS.clear()
                    parser.TOKEN_DEFS.update(tokens)
                if job is not None:
                    body, subtype, _, scales = job
                    fragments.append(Fragment('attached_subtype', Layer.TYPE, subtype))
                    if body[5] or body[6]:
                        fragments.append(Fragment('attachment_stats', Layer.MODIFY, (body[5], body[6])))
                    if body[7]:
                        fragments.append(Fragment('attachment_keywords', Layer.ABILITY, body[7]))
                    for condition, program, targets in body[2]:
                        if condition.startswith('Granted('):
                            fragments.append(Fragment('attachment_trigger', Layer.ABILITY, (condition, tuple(program), targets)))
                    fragments.extend(Fragment('scaling_modifiers', Layer.MODIFY, s) for s in scales)
            else:
                body = parser.equipment_template(dict(card, oracle_text='\n'.join([residual.get('oracle_text', ''), *equips])))
                if body is not None:
                    if body[5] or body[6]:
                        fragments.append(Fragment('attachment_stats', Layer.MODIFY, (body[5], body[6])))
                    if body[7]:
                        fragments.append(Fragment('attachment_keywords', Layer.ABILITY, body[7]))
                    for condition, program, targets in body[2]:
                        if condition in ('GrantedAttacks',) or condition.startswith('Granted('):
                            fragments.append(Fragment('attachment_trigger', Layer.ABILITY, (condition, tuple(program), targets)))
        elif aura and original not in enchant:
            body = parser.aura_template(dict(card, oracle_text='\n'.join([*enchant, residual.get('oracle_text', '')])))
            if body is not None:
                if body[7] or body[8]:
                    fragments.append(Fragment('attachment_stats', Layer.MODIFY, (body[7], body[8])))
                if body[9]:
                    fragments.append(Fragment('attachment_keywords', Layer.ABILITY, body[9]))
                for condition, program, targets in body[2]:
                    if condition.startswith('Granted('):
                        fragments.append(Fragment('attachment_trigger', Layer.ABILITY, (condition, tuple(program), targets)))
        if fragments:
            groups.append(StaticGroup(index, original, tuple(fragments)))
    return tuple(groups)
