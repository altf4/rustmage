"""Emit immutable per-set metadata and the public assembly; never parse card rules."""
from collections import defaultdict
from pathlib import Path
import re
import json


def render(root, metadata, records, version, definition_lines, legacy_catalogue, coverage):
    cards = metadata['cards']
    # These aliases are rule vocabulary already used by the engine. New identities
    # append to the catalogue instead of renumbering compiled intrinsic aliases.
    intrinsic = re.findall(r'pub const (\w+): Card = Card\((\d+)\)',
                           (root/'crates/engine/src/catalogue.rs').read_text())
    for name, index in intrinsic:
        if metadata['constants'].get(name) != int(index):
            raise ValueError('intrinsic Card alias changed; preserve existing catalogue IDs: ' + name)
    grouped = defaultdict(list)
    for card in cards:
        grouped[card['path'].split('/')[0]].append(card)
    outputs = {}
    accessors = ['// Generated shared rule vocabulary/accessors; no card catalogue data.',
                 '#[rustfmt::skip]', 'pub const CREATURE_TYPES: &[&str] = &[',
                 *('    '+json.dumps(name)+',' for name in metadata['creature_types']), '];']
    for name, result in [('back_face', 'Option<Card>'), ('face_parent', 'Option<Card>'),
                         ('physical_identity', "Option<&'static str>"),
                         ('token_info', "Option<&'static TokenInfo>"),
                         ('emblem_info', "Option<&'static EmblemInfo>")]:
        accessors.append(f'pub(super) fn {name}(card: Card) -> {result} {{ (catalogue().{name})(card) }}')
    accessors += [f"pub(super) fn {field}(card: Card) -> {ty} {{ super::authored::rules(card).unwrap().extras.{field} }}"
                  for field, ty in metadata['helper_interface']]
    outputs[root/'crates/engine/src/catalogue_accessors.rs'] = '\n'.join(accessors)+'\n'
    dependencies = []
    definition_refs, rule_refs = {}, {}
    for code, members in sorted(grouped.items()):
        package = 'rustmage-cards-' + code.replace('_', '-')
        alias = package.replace('-', '_')
        directory = root/'crates/cards'/code
        outputs[directory/'Cargo.toml'] = f'''# Generated package linkage; edit src/card_definitions/{code}/.
[package]
name = "{package}"
version = "0.1.0"
edition = "2024"
license = "MIT"
publish = false
[lib]
test = false
doctest = false
[dependencies]
rustmage-core = {{ path = "../../engine", version = "=0.1.0" }}
'''
        if code == 'synthetic':
            outputs[directory/'Cargo.toml'] += '\n[features]\ntest-support = ["rustmage-core/test-support"]\n'
        dependencies.append(f'{package} = {{ path = "crates/cards/{code}", version = "=0.1.0" }}')
        lines = ['// Generated module links and metadata, never rule behavior.',
                 '#![allow(unused_imports)]', 'use rustmage_core::engine;', '#[rustfmt::skip]', 'mod cards {',
                 'pub use rustmage_core::cards::*;',
                 'pub type Card = rustmage_core::Card;',
                 '#[allow(dead_code, non_snake_case)]',
                 'pub const fn Card(index: u16) -> Card { Card::from_catalogue_id(index) }',
                 'use rustmage_core::cards::authored::{CardRules, CardExtras, generic, one, object};',
                 'mod authored {', 'use super::*;']
        for card in members:
            lines += [f'pub(super) mod {card["module"]} {{',
                      'use crate::cards::*;',
                      'use crate::cards::InitialCounters as Counters;',
                      'use super::{CardRules, CardExtras, generic, one, object};',
                      f'include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/{card["path"]}"));',
                      'const _: () = assert!(' +
                      ('matches!(RULES.extras.has_rules_abilities, Some(false))' if records[card['index']].get('has_rules_abilities') is False
                       else '!matches!(RULES.extras.has_rules_abilities, Some(false))') +
                      ', "authored ability-presence fact differs from provenance");', '}']
        lines += ['}', '#[rustfmt::skip]', f'pub static ENTRIES: [CatalogueEntry; {len(members)}] = [']
        for offset, card in enumerate(members):
            definition = definition_lines[card['index']].strip().removesuffix(',')
            lines.append(f'CatalogueEntry {{ definition: {definition}, rules: &authored::{card["module"]}::RULES }},')
            definition_refs[card['index']] = f'&{alias}::ENTRIES[{offset}].definition'
            rule_refs[card['index']] = f'{alias}::ENTRIES[{offset}].rules'
        lines += ['];', '}', 'pub use cards::ENTRIES;']
        outputs[directory/'src/lib.rs'] = '\n'.join(lines) + '\n'
    manifest = root/'Cargo.toml'
    original = manifest.read_text()
    outputs[manifest] = re.sub(r'(# BEGIN GENERATED SET DEPENDENCIES\n).*?(# END GENERATED SET DEPENDENCIES)',
                              lambda m: m[1] + '\n'.join(dependencies) + '\n' + m[2], original, flags=re.S)
    # Face/token/name metadata belongs to the assembly, not the cached engine.
    suffix = legacy_catalogue[legacy_catalogue.index('pub(super) fn back_face'):legacy_catalogue.index('#[allow(non_upper_case_globals)]')]
    suffix = suffix.replace('pub(super)', 'pub(crate)')
    suffix = re.sub(r'\bCard\((\d+)\)', r'Card::from_catalogue_id(\1)', suffix)
    colors = repr(metadata['colors'])
    all_ids = [c['index'] for c in [card for card in cards if card['category']=='physical'] if records[c['index']]['complete']]
    lines = ['// Generated assembly. Per-set behavior is compiled independently.',
             'use rustmage_core::cards::*;', 'use rustmage_core::cards::generated::Catalogue;',
             f'pub const CATALOGUE_VERSION: &str = "{version}";',
             '#[rustfmt::skip]', f'static COLORS: [u8; {len(cards)}] = {colors};',
             '#[rustfmt::skip]', f'static DEFINITIONS: [&Definition; {len(cards)}] = [',
             *(definition_refs[c['index']] + ',' for c in cards), '];',
             '#[rustfmt::skip]', f'static RULES: [&rustmage_core::cards::authored::CardRules; {len(cards)}] = [',
             *(rule_refs[c['index']] + ',' for c in cards), '];',
             '#[rustfmt::skip]', 'static SUPPORTED: &[Card] = &[',
             *(f'Card::from_catalogue_id({i}),' for i in all_ids), '];', suffix,
             '// SAFETY: this exact-version Rust workspace supplies the one immutable',
             '// registry declared by rustmage-core. No mutable state or runtime loading.',
             '#[unsafe(export_name = "__rustmage_catalogue_v1_0_1_0")]',
             'pub static LINKED_CATALOGUE: Catalogue = Catalogue {',
             'version: CATALOGUE_VERSION, colors: &COLORS, definitions: &DEFINITIONS,',
             'names: &NAME_INDEX, rules: &RULES, supported: SUPPORTED,',
             'back_face, face_parent, physical_identity, token_info, emblem_info,',
             'card_names: include_str!("../../data/card-names.txt"),',
             'land_names: include_str!("../../data/land-card-names.txt"),',
             'interchangeable_names: include_str!("../../data/interchangeable-card-names.tsv"),',
             'outside_names: include_str!("../../data/outside-card-names.tsv"),', '};']
    outputs[root/'src/generated/catalogue.rs'] = '\n'.join(lines) + '\n'
    outputs[root/'src/catalogue.rs'] = 'include!("generated/catalogue.rs");\n'
    outputs[root/'data/standard/coverage.json'] = coverage
    return outputs
