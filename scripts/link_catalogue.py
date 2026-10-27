#!/usr/bin/env python3
"""Assemble pinned metadata and direct Rust modules. Never interpret rules text."""
import argparse, hashlib, json, re
from collections import Counter, defaultdict
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def canonical(value):return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def quote(value):return json.dumps(value,ensure_ascii=False)
def boolean(value):return str(value).lower()
def physical_digest(metadata,card):
    return hashlib.sha256(canonical(dict(card=card,intrinsic_colors=metadata['colors'][card['index']]))).hexdigest()
def validate_simple_mana_components(metadata, records):
    """Check fixed printed components, skipping hybrid and other special symbols."""
    checked = 0
    for card in metadata['cards']:
        if card['category'] not in ('physical', 'back_face') or not records[card['index']]['complete']:
            continue
        card = dict(card, **card.get('front_face', {}))
        cost = card['mana_cost']
        if not re.fullmatch(r'(?:\{(?:[0-9]+|[WUBRGCX])\})*', cost):
            continue
        colored, generic = [0] * 6, 0
        for symbol in re.findall(r'\{([^}]+)\}', cost):
            if symbol.isdecimal():
                generic += int(symbol)
            elif symbol != 'X':
                colored['WUBRGC'.index(symbol)] += 1
        if card['colored'] != colored or card['generic'] != generic:
            raise ValueError('fixed printed mana components changed: ' + card['name'])
        checked += 1
    return checked

def load(root=ROOT):
    metadata=json.loads((root/'data/catalogue-identities.json').read_text())
    facts={c['oracle_id']:c for c in json.loads((root/'data/standard/cards.json').read_text())['cards']}
    by_id={c['index']:c for c in metadata['cards']}
    by_oracle={c['oracle_id']:c for c in metadata['cards'][:metadata['physical_count']] if c['oracle_id']}
    records={};fingerprint=bytearray(canonical(metadata))
    directory=root/'src/card_definitions'
    for path in sorted(directory.rglob('*.json')):
        record=json.loads(path.read_text())
        index=record.get('catalogue_id')
        card=by_id[index] if index is not None else by_oracle[record['oracle_id']]
        index=card['index']
        if index in records:raise ValueError('duplicate card definition: '+card['name'])
        if record['name']!=card['name'] or record.get('oracle_id')!=card['oracle_id']:raise ValueError('card identity changed: '+str(path))
        expected=directory/card['path']
        if path!=expected.with_suffix('.json'):raise ValueError('card must use pinned canonical set/path: '+card['path'])
        if card['facts_sha256']:
            if record.get('facts_sha256')!=card['facts_sha256'] or hashlib.sha256(canonical(facts[card['oracle_id']])).hexdigest()!=card['facts_sha256']:raise ValueError('pinned facts changed: '+card['name'])
        if record.get('physical_metadata_sha256')!=physical_digest(metadata,card):raise ValueError('pinned physical metadata changed: '+card['name'])
        source=path.with_suffix('.rs')
        if not source.is_file():raise ValueError('missing typed Rust: '+str(source))
        if type(record.get('complete')) is not bool:raise ValueError('missing complete declaration: '+str(path))
        records[index]=dict(record,source=source,relative=source.relative_to(root).as_posix())
        fingerprint.extend(canonical(record));fingerprint.extend(source.read_bytes())
    if records.keys()!=by_id.keys():raise ValueError('missing card definitions: '+str(sorted(by_id.keys()-records.keys())))
    for card in metadata['cards']:
        if 'front_face' in card:
            oracle = facts[card['oracle_id']]
            if oracle['layout'] not in ('modal_dfc', 'transform', 'adventure', 'prepare'): raise ValueError('face metadata requires modal DFC, adventure or prepare')
            face = oracle['card_faces'][0]
            if any(card['front_face'][key] != face[key] for key in ['name','mana_cost','type_line']): raise ValueError('front face facts changed')
            if oracle['layout'] in ('modal_dfc', 'transform'):
                back = by_id[card['back_face']]
                if back['face_parent'] != card['index'] or any(back[key] != oracle['card_faces'][1][key] for key in ['name','mana_cost','type_line']): raise ValueError('back face facts changed')
        face_pin = records[card['index']].get('oracle_face')
        if face_pin:
            face = facts[face_pin['oracle_id']]['card_faces'][face_pin['index']]
            if hashlib.sha256(canonical(face)).hexdigest() != face_pin['sha256']: raise ValueError('Oracle face pin changed')
    validate_simple_mana_components(metadata, records)
    fingerprint.extend((directory/'mod.rs').read_bytes())
    return metadata,records,hashlib.sha256(fingerprint).hexdigest()
def validate_synthetic_aliases(metadata):
    aliases = dict(metadata['aliases'])
    if len(aliases) != len(metadata['aliases']) or metadata['aliases'] != sorted(metadata['aliases']):
        raise ValueError('catalogue aliases must be unique and sorted')
    token_cards = [c for c in metadata['cards'] if c['category'] == 'token']
    if len(token_cards) != len(metadata['tokens']):
        raise ValueError('token metadata count mismatch')
    identities = [(token['factory_identity'], card['index'])
                  for card, token in zip(token_cards, metadata['tokens'])]
    identities.extend((card['name'], card['index']) for card in metadata['cards']
                      if card['category'] == 'emblem')
    for identity, index in identities:
        if aliases.get(identity) != index:
            raise ValueError('missing or incorrect synthetic factory alias: ' + identity)

def render(metadata,records,version):
    validate_synthetic_aliases(metadata)
    cards=metadata['cards'];n=len(cards)
    registry=['// Module links only. Edit the independent per-set Rust files.']
    for c in cards:
        name=c['module'];path=c['path']
        registry.extend([f'pub(super) mod {name} {{','    use crate::cards::*;', '    #[allow(unused_imports)]', '    use crate::cards::InitialCounters as Counters;', '    // Import only shared helpers: a sibling glob creates quadratic name resolution.', '    #[allow(unused_imports)]', '    use super::{CardRules, CardExtras, generic, one, object};',f'    include!("../card_definitions/{path}");','}'])
    registry.extend(['#[rustfmt::skip]',f'pub(super) static RULES: [&CardRules; {n}] = ['])
    registry.extend(f'    &{c["module"]}::RULES,'for c in cards)
    registry.extend(['];',"pub(super) fn mode_costs(card: Card) -> Option<&'static [ManaCost]> { RULES[card.index()].mode_costs }"])
    registry.append("pub(super) fn rules(card: Card) -> Option<&'static CardRules> { RULES.get(card.index()).copied() }")
    lines=['// Metadata/module linkage only; card behavior lives in src/card_definitions/.',f'pub const CATALOGUE_VERSION: &str = "{version}";','#[rustfmt::skip]',f'pub(super) static COLORS: [u8; {n}] = {metadata["colors"]!r};','#[rustfmt::skip]',f'pub(super) static DEFINITIONS: [Definition; {n}] = [']
    for c in cards:
        r='authored::'+c['module']+'::RULES';complete=records[c['index']]['complete']
        c = dict(c, **c.get('front_face', {}))
        info='CardInfo { '+', '.join([f'name: {quote(c["name"])}',f'oracle_id: '+('Some('+quote(c['oracle_id'])+')'if c['oracle_id'] else 'None'),f'mana_cost: {quote(c["mana_cost"])}',f'type_line: {quote(c["type_line"])}','oracle_keywords: &['+', '.join(map(quote,c['oracle_keywords']))+']',f'standard_legal: {boolean(c["standard_legal"])}',f'legendary: {boolean(c["legendary"])}',f'implemented: {boolean(complete)}',f'keywords: {r}.keywords',f'deck_limit: {r}.deck_limit'])+' }'
        expr=[f'info: {info}',f'effect: {r}.checked_effect()'if complete else f'effect: {r}.effect',*[f'{f}: {r}.{f}'for f in ['triggers','activated','enters','entry']],f'kind: {c["kind"]}',f'basic: {boolean(c["basic"])}','mana_color: '+('None'if c['mana_color'] is None else f'Some({c["mana_color"]})'),f'colored: {c["colored"]!r}',f'generic: {c["generic"]}']
        lines.append('    Definition { '+', '.join(expr)+' },')
    lines+= ['];']
    for key in ['back_face','face_parent']:
        lines.append(f'pub(super) fn {key}(card: Card) -> Option<Card> {{ match card.index() {{')
        lines.extend(f"{c['index']} => Some(Card({c[key]}))," for c in cards if key in c)
        lines.append('_ => None, } }')
    lines.append("pub(super) fn physical_identity(card: Card) -> Option<&'static str> { match card.index() {")
    lines.extend(f"{c['index']} => Some({quote(c['name'])})," for c in cards if 'front_face' in c)
    lines.append('_ => None, } }')
    lines+= ['#[rustfmt::skip]' ,f'pub(super) static NAME_INDEX: [(&str,u16); {len(metadata["aliases"])}] = [']
    lines.extend(f'    ({quote(name)}, {index}),'for name,index in metadata['aliases']);lines.append('];')
    for k in ['token_base','emblem_base']:lines.append(f'#[allow(dead_code)] pub(super) const {k.upper()}: usize = {metadata[k]};')
    for key,ty in [('tokens','TokenInfo'),('emblems','EmblemInfo')]:
        lines+=['#[rustfmt::skip]',f'pub(super) static {key.upper()}: [{ty}; {len(metadata[key])}] = [']
        for obj in metadata[key]:lines.append('    '+ty+' { '+', '.join(k+': '+(quote(v)if isinstance(v,str)else str(v))for k,v in obj.items())+' },')
        lines.append('];')
    token_cards=[c for c in cards if c['category']=='token']
    if len(token_cards)!=len(metadata['tokens']):raise ValueError('token metadata count mismatch')
    for card,token in zip(token_cards,metadata['tokens']):
        if card['name'] not in (token['factory_identity'],token['name']):raise ValueError('token metadata identity mismatch: '+card['name'])
    lines.append("pub(super) fn token_info(card: Card) -> Option<&'static TokenInfo> { match card.index() {")
    lines.extend(f"    {c['index']} => Some(&TOKENS[{offset}])," for offset,c in enumerate(token_cards))
    lines.append("    _ => None, } }")
    emblem_cards=[c for c in cards if c['category']=='emblem']
    if len(emblem_cards)!=len(metadata['emblems']):raise ValueError('emblem metadata count mismatch')
    lines.append("pub(super) fn emblem_info(card: Card) -> Option<&'static EmblemInfo> { match card.index() {")
    lines.extend(f"    {c['index']} => Some(&EMBLEMS[{offset}])," for offset,c in enumerate(emblem_cards))
    lines.append("    _ => None, } }")
    all_ids=[c['index']for c in cards[:metadata['physical_count']]if records[c['index']]['complete']]
    lines+=['#[allow(non_upper_case_globals)]','impl Card {','#[rustfmt::skip]',f'    pub const ALL: [Card; {len(all_ids)}] = ['+', '.join(f'Card({i})'for i in all_ids)+'];']
    lines.extend(f'    pub const {name}: Card = Card({index});'for name,index in metadata['constants'].items());lines.append('}')
    lines+=['#[rustfmt::skip]','pub const CREATURE_TYPES: &[&str] = &[',*[f'    {quote(t)},'for t in metadata['creature_types']],'];']
    for field,ty in metadata['helper_interface']:
        lines.append(f'pub(super) fn {field}(card: Card) -> {ty} {{ authored::RULES[card.index()].extras.{field} }}')
    coverage=[];sets=defaultdict(Counter)
    for c in cards[:metadata['physical_count']]:
        complete=records[c['index']]['complete'];row=dict(name=c['name'],standard_legal=c['standard_legal'],implemented=complete,implementation='typed_rust'if complete else None,definition=records[c['index']]['relative'],set=c['set'])
        coverage.append(row)
        if c['standard_legal']:
            sets[c['set']]['total']+=1;sets[c['set']]['complete'if complete else 'gated']+=1
    report=dict(schema=1,catalogue_version=version,standard_legal_cards=sum(c['standard_legal']for c in coverage),implemented_standard_cards=sum(c['standard_legal']and c['implemented']for c in coverage),implemented_total=sum(c['implemented']for c in coverage),token_definitions=len(metadata['tokens']),emblem_definitions=len(metadata['emblems']),adventure_definitions=sum(c['category']=='adventure'for c in cards),omen_definitions=sum(c['category']=='omen'for c in cards),definition_ownership={s:dict(v)for s,v in sorted(sets.items())},cards=coverage)
    return {ROOT/'src/generated/catalogue.rs':'\n'.join(lines)+'\n',ROOT/'src/generated/authored_cards.rs':'\n'.join(registry)+'\n',ROOT/'data/standard/coverage.json':json.dumps(report,ensure_ascii=False,indent=2)+'\n'}
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');args=p.parse_args()
    metadata,records,version=load();outputs=render(metadata,records,version)
    stale=[str(path.relative_to(ROOT))for path,value in outputs.items()if not path.is_file()or path.read_text()!=value]
    if args.check and stale:raise SystemExit('stale metadata/module links: '+', '.join(stale))
    if not args.check:
        for path,value in outputs.items():path.write_text(value)
    report=json.loads(outputs[ROOT/'data/standard/coverage.json']);print(json.dumps({k:v for k,v in report.items()if k not in ['cards','definition_ownership']}))
if __name__=='__main__':main()
