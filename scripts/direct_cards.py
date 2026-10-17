"""Link hand-authored Rust behavior to pinned identities. This never translates rules."""
import hashlib,json,re

def digest(value):
    return hashlib.sha256(json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()

def load(root):
    directory=root/'src/card_definitions'
    facts={c['oracle_id']:c for c in json.loads((root/'data/standard/cards.json').read_text())['cards']}
    cards={};seen=set();fingerprint=bytearray()
    for path in sorted(directory.glob('*.json')):
        record=json.loads(path.read_text());slug=path.stem
        if not re.fullmatch('[a-z][a-z0-9_]*',slug):raise ValueError('invalid authored module: '+slug)
        card=facts[record['oracle_id']]
        if digest(card)!=record['facts_sha256']:raise ValueError('authored facts changed: '+card['name'])
        source=path.with_suffix('.rs')
        if not source.is_file():raise ValueError('missing authored Rust: '+slug)
        if record['oracle_id'] in seen:raise ValueError('duplicate authored identity')
        seen.add(record['oracle_id'])
        if record.get('complete') is True:cards[record['oracle_id']]=dict(record,module=slug)
        fingerprint.extend(path.read_bytes());fingerprint.extend(source.read_bytes())
    if (directory/'mod.rs').exists():fingerprint.extend((directory/'mod.rs').read_bytes())
    return cards,bytes(fingerprint)

def registry(records,indices):
    lines=['// Generated module links only; behavior lives in independent Rust files.']
    for record in records.values():
        slug=record['module'];lines.extend([f'pub(super) mod {slug} {{', '    use super::*;', f'    include!("../card_definitions/{slug}.rs");','}'])
    lines.extend(["pub(super) fn mode_costs(card:Card)->Option<&'static [ManaCost]> {",'    match card.0 {'])
    for identity,record in records.items():
        if identity in indices:lines.append(f'        {indices[identity]} => {record["module"]}::RULES.mode_costs,')
    lines.extend(['        _=>None,','    }','}'])
    lines.extend(["pub(super) fn rules(card:Card)->Option<&'static CardRules> {", '    match card.0 {'])
    for identity,record in records.items():
        if identity in indices:lines.append(f'        {indices[identity]} => Some(&{record["module"]}::RULES),')
    lines.extend(['        _=>None,','    }','}'])
    return '\n'.join(lines)+'\n' 
