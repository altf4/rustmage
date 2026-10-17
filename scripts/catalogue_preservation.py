"""Compare generated facts by physical oracle identity, never numeric Card offsets.

Callers must explicitly declare every changed definition, new definition and new
helper arm. This module does not authorize incidental admissions.
"""
from collections import Counter
import json
import re

_STRING = r'"(?:[^"\\]|\\.)*"'

def canonical(text, physical):
    rows = [line for line in text.splitlines() if line.lstrip().startswith('Definition {')]
    occurrences = Counter()
    keys = []
    for row in rows:
        name = json.loads(re.search(r'name: (' + _STRING + ')', row)[1])
        oracle = re.search(r'oracle_id: Some\("([^"]+)"\)', row)
        if oracle and 'standard_legal: true' in row:
            name = physical[oracle[1]]
        keys.append(name + '#' + str(occurrences[name]))
        occurrences[name] += 1
    assert len(set(keys)) == len(keys)
    def norm(value):
        return re.sub(r'Card\((\d+)\)', lambda m: 'Card(' + json.dumps(keys[int(m[1])]) + ')', value)
    helpers, flags, aliases, tokens = {}, {}, {}, {}
    function = None
    in_aliases = False
    emblem_rows = []
    all_cards = None
    for line in text.splitlines():
        if line.startswith('pub(super) fn '):
            function = line.split('fn ', 1)[1].split('(', 1)[0]
        if match := re.match(r'        (\d+) => (.*)', line):
            key = (function, keys[int(match[1])])
            assert key not in helpers
            helpers[key] = norm(match[2])
        if match := re.search(r'matches!\(card\.0, ([\d |]+)\)', line):
            flags[function] = frozenset(keys[int(i)] for i in match[1].split('|'))
        if 'static NAME_INDEX:' in line:
            in_aliases = True
        elif in_aliases and line == '];':
            in_aliases = False
        elif in_aliases:
            match = re.fullmatch(r'    \((' + _STRING + r'), (\d+)\),', line)
            assert match, line
            name = json.loads(match[1])
            assert name not in aliases
            aliases[name] = keys[int(match[2])]
        if 'TokenInfo { factory_identity:' in line:
            identity = json.loads(re.search(r'factory_identity: (' + _STRING + ')', line)[1])
            assert identity not in tokens
            tokens[identity] = norm(line.strip())
        if line.startswith('EmblemInfo {'):
            emblem_rows.append(norm(line))
        if 'pub const ALL:' in line:
            all_cards = frozenset(keys[int(i)] for i in re.findall(r'Card\((\d+)\)', line))
    emblem_keys = [key for key in keys if key.startswith('@emblem/')]
    assert len(emblem_rows) == len(emblem_keys)
    colors = json.loads(re.search(r'static COLORS: \[u8; \d+\] = (\[[^;]+\]);', text)[1])
    assert len(colors) == len(keys) and all_cards is not None
    return dict(definitions=dict(zip(keys, map(norm, rows))), helpers=helpers,
                colors=dict(zip(keys, colors)), flags=flags, aliases=aliases,
                tokens=tokens, emblems=dict(zip(emblem_keys, emblem_rows)), all=all_cards)


def assert_union(before, after, *, changed, added, helpers, aliases):
    """Exact declared union; old metadata and every unrelated body stay intact."""
    old, new = before['definitions'], after['definitions']
    assert set(old) <= set(new), set(old) - set(new)
    assert {key for key in old if old[key] != new[key]} == changed
    assert set(new) - set(old) == added
    for key in changed:
        assert 'effect: Effect::Unsupported' in old[key]
        assert 'implemented: true' in new[key] and 'Effect::Unsupported' not in new[key]
    for section in ['helpers', 'colors', 'aliases']:
        assert all(after[section].get(key) == value for key, value in before[section].items()), section
    assert set(after['helpers']) - set(before['helpers']) == helpers
    assert set(after['colors']) - set(before['colors']) == added
    assert set(after['aliases']) - set(before['aliases']) == set(aliases)
    assert {key: after['aliases'][key] for key in aliases} == aliases
    for section in ['flags', 'tokens', 'emblems']:
        assert before[section] == after[section], section
    assert after['all'] - before['all'] == changed
    assert before['all'] <= after['all']
    return dict(prior_definitions=len(old), current_definitions=len(new),
                changed_definitions=sorted(changed), added_definitions=sorted(added),
                preserved_definitions=len(old)-len(changed),
                prior_helpers=len(before['helpers']), new_helpers=sorted(helpers),
                prior_colors=len(before['colors']), inline_flag_tables=len(before['flags']),
                token_recipes=len(before['tokens']), emblem_recipes=len(before['emblems']),
                prior_aliases=len(before['aliases']), result='PASS')
