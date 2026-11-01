#!/usr/bin/env python3
"""Materialize the isolated 186-card crate-split experiment; never edit the input.

Input must be the archived matched new-source snapshot, which contains the exact
91 selected scenarios and benchmark identity instrumentation. This is a prototype,
not a production migration: it deliberately exposes internals in an unpublished
engine crate so the original test bodies can run unchanged across crate boundaries.
"""
import pathlib,re,shutil,sys,json

TOKEN=re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|r(?P<raw>\#*)"[\s\S]*?"(?P=raw)|b?"(?:\\[\s\S]|[^"\\])*"|b?\'(?:\\[\s\S]|[^\'\\])\'|[A-Za-z_][A-Za-z_0-9]*|[^\s]',re.M)
def tokens(text):
 return [m for m in TOKEN.finditer(text) if not m[0].startswith(('//','/*'))]
def pair(ts,index):
 opening=ts[index][0]; stack=[{'(':')','[':']','{':'}'}[opening]]
 for i in range(index+1,len(ts)):
  c=ts[i][0]
  if c in ('(','[','{'):stack.append({'(':')','[':']','{':'}'}[c])
  elif c in (')',']','}'):
   assert c==stack.pop(),(c,ts[i].start())
   if not stack:return i
 raise ValueError('unclosed')
def test_functions(text):
 ts=tokens(text)
 for i,t in enumerate(ts):
  if t[0]!='#' or i+3>=len(ts):continue
  if [x[0] for x in ts[i:i+4]]!=['#','[','test',']']:continue
  j=i+4
  while ts[j][0]=='#':j=pair(ts,j+1)+1
  if ts[j][0]!='fn':continue
  name=ts[j+1][0]
  while ts[j][0]!='{':j+=1
  end=pair(ts,j)
  yield t.start(),ts[end].end(),name,text[ts[j].start():ts[end].end()]

import argparse
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--source', required=True, type=pathlib.Path)
parser.add_argument('--output', required=True, type=pathlib.Path)
args = parser.parse_args()
SRC=args.source.resolve()
DST=args.output.resolve()
if SRC == DST or SRC in DST.parents:
    parser.error('output must be outside the source tree')
if 'mod matched_card_benchmark' not in (SRC/'src/lib.rs').read_text():
    parser.error('source must be the instrumented matched benchmark snapshot')
if DST.exists():raise SystemExit('output already exists')
shutil.copytree(SRC,DST,ignore=shutil.ignore_patterns('.git','target','__pycache__'))
base=(SRC/'Cargo.toml').read_text()
core=DST/'crates/engine';(core/'src').mkdir(parents=True)
# Source snapshots remain immutable once a build starts. Only a disposable copy is changed.
for p in (SRC/'src').glob('*.rs'):shutil.copy2(p,core/'src'/p.name)
(core/'src/generated').mkdir()
(core/'src/card_definitions').mkdir()
shutil.copytree(SRC/'data', core/'data')
# Export internals ONLY in this isolated experiment, to let unchanged unit scenarios
# live in another crate. Production migration must use a narrow test-support API.
def expose(text):
 text=re.sub(r'pub\((?:[^()]|\([^()]*\))*\)', 'pub', text)
 ts=tokens(text);trait_ranges=[];struct_ranges=[]
 for i,t in enumerate(ts):
  if t[0] in ('impl','trait','struct'):
   j=i+1
   while j<len(ts) and ts[j][0] not in ('{',';'):j+=1
   if j==len(ts) or ts[j][0]!='{':continue
   end=pair(ts,j)
   if t[0]=='trait' or (t[0]=='impl' and any(x[0]=='for' for x in ts[i:j])):
    trait_ranges.append((ts[j].start(),ts[end].end()))
   elif t[0]=='struct':struct_ranges.append((j,end))
 edits=[]
 # Fields of named structs (skip nested generic types/delimiters).
 for a,b in struct_ranges:
  j=a+1
  while j<b:
   if ts[j][0]=='#':j=pair(ts,j+1)+1;continue
   if ts[j][0]=='pub':j+=1
   elif j+1<b and ts[j+1][0]==':':edits.append((ts[j].start(),'pub '))
   # jump to next outer comma without confusing generics or tuples
   depth=0
   while j<b:
    tok=ts[j][0]
    if tok in ('(','[','{'):j=pair(ts,j)+1;continue
    if tok=='<':depth+=1
    elif tok=='>':depth=max(0,depth-1)
    elif tok==',' and depth==0:j+=1;break
    j+=1
 for m in re.finditer(r'(?m)^(\s*)(?=(?:(?:async|unsafe|const) )?(?:fn|struct|enum|type|trait|mod|static|const)\b)',text):
  pos=m.end()
  if not any(a<pos<b for a,b in trait_ranges):edits.append((pos,'pub '))
 for pos,addition in sorted(set(edits),reverse=True):text=text[:pos]+addition+text[pos:]
 text=text.replace('pub struct Card(u16);','pub struct Card(pub u16);')
 return text
for p in (core/'src').glob('*.rs'):
 p.write_text(expose(p.read_text()))
# Engine's card rules are types, not per-card implementations.
rules=(SRC/'src/card_definitions/mod.rs').read_text().split('include!("../generated/authored_cards.rs");')[0]
rules=expose(rules)
rules+='''\npub fn rules(card: Card) -> Option<&'static CardRules> { generated::catalogue().rules.get(card.index()).copied() }
pub fn mode_costs(card: Card) -> Option<&'static [ManaCost]> { rules(card).and_then(|r| r.mode_costs) }
'''
(core/'src/card_definitions/mod.rs').write_text(rules)
cards=(core/'src/cards.rs').read_text()
cards=cards[:cards.index('#[cfg(test)]\n#[path = "testing/bindings.rs"]')]
cards=cards.replace('authored::RULES[self.index()]','authored::rules(self).unwrap()')
cards=cards.replace('generated::DEFINITIONS','generated::catalogue().definitions').replace('generated::COLORS','generated::catalogue().colors').replace('generated::NAME_INDEX','generated::catalogue().names')
cards=cards.replace('pub const _: () = assert!(generated::catalogue().definitions.len() < u16::MAX as usize);\n','')
# Named Definition fields need crossing the internal crate boundary too.
cards=expose(cards)
(core/'src/cards.rs').write_text(cards)
generated=(SRC/'src/generated/catalogue.rs').read_text()
constants=generated[generated.index('#[allow(non_upper_case_globals)]'):generated.index('#[rustfmt::skip]\npub const CREATURE_TYPES')]
creatures=generated[generated.index('#[rustfmt::skip]\npub const CREATURE_TYPES'):generated.index('pub(super) fn static_modifiers')]
getters=generated[generated.index('pub(super) fn static_modifiers'):].replace('pub(super)','pub').replace('authored::RULES[card.index()]','authored::rules(card).unwrap()')
proxy='''use std::ops::Deref;
use super::authored::CardRules;
pub struct Catalogue {
    pub version: &'static str,
    pub colors: &'static [u8],
    pub definitions: &'static [Definition],
    pub names: &'static [(&'static str, u16)],
    pub rules: &'static [&'static CardRules],
    pub back_face: fn(Card) -> Option<Card>,
    pub face_parent: fn(Card) -> Option<Card>,
    pub physical_identity: fn(Card) -> Option<&'static str>,
    pub token_info: fn(Card) -> Option<&'static TokenInfo>,
    pub emblem_info: fn(Card) -> Option<&'static EmblemInfo>,
}
// One immutable Rust-only link symbol, supplied by the assembly crate. This is
// statically linked into the ordinary rustmage rlib/cdylib, not a runtime plugin.
unsafe extern "Rust" { pub static RUSTMAGE_CATALOGUE: Catalogue; }
pub fn catalogue() -> &'static Catalogue { unsafe { &RUSTMAGE_CATALOGUE } }
pub fn back_face(card: Card) -> Option<Card> { (catalogue().back_face)(card) }
pub fn face_parent(card: Card) -> Option<Card> { (catalogue().face_parent)(card) }
pub fn physical_identity(card: Card) -> Option<&'static str> { (catalogue().physical_identity)(card) }
pub fn token_info(card: Card) -> Option<&'static TokenInfo> { (catalogue().token_info)(card) }
pub fn emblem_info(card: Card) -> Option<&'static EmblemInfo> { (catalogue().emblem_info)(card) }
'''
version=re.search(r'pub const CATALOGUE_VERSION[^\n]+',generated)[0]
(core/'src/generated/catalogue.rs').write_text(version+'\n'+proxy+constants+creatures+getters)
# Production rlib excludes FFI and tests. Tracing remains identical in both variants.
lib=(core/'src/lib.rs').read_text().replace('pub mod ffi;','')
lib=lib.replace('#[cfg(test)]\npub mod matched_card_benchmark','pub mod matched_card_benchmark')
(core/'src/lib.rs').write_text('#![allow(dead_code, unused_imports, private_interfaces)]\n'+lib)
engine=(core/'src/engine.rs').read_text()
engine=engine[:engine.index('#[cfg(test)]\n#[path = "testing/mod.rs"]')]
engine=engine.replace('#[cfg(test)]\n        crate::matched_card_benchmark', 'crate::matched_card_benchmark')
(core/'src/engine.rs').write_text(engine)
(core/'Cargo.toml').write_text('''[package]
name = "rustmage-engine"
version = "0.1.0"
edition = "2024"
publish = false
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["arbitrary_precision"] }
''')
# Keep every catalogue entry/definition; split one development set from the rest.
authored=(SRC/'src/generated/authored_cards.rs').read_text()
modules=re.findall(r'pub\(super\) mod (\w+) \{.*?include!\("\.\./card_definitions/([^\"]+)"\);\n\}',authored,re.S)
assert len(modules)==5487,len(modules)
sets={'msh':[],'other':[]}
for num,path in modules:sets['msh' if path.startswith('msh/') else 'other'].append((num,path))
for name,rows in sets.items():
 pack=DST/'crates'/('cards-'+name);(pack/'src/cards').mkdir(parents=True)
 (pack/'Cargo.toml').write_text(f'''[package]
name = "rustmage-cards-{name}"
version = "0.1.0"
edition = "2024"
publish = false
[dependencies]
rustmage-engine = {{ path = "../engine" }}
''')
 out=['#![allow(dead_code, unused_imports)]','pub use rustmage_engine::engine;','pub mod cards { pub use rustmage_engine::cards::*; }','use crate::cards::*;','use crate::cards::authored::*;']
 for num,path in rows:
  dest=pack/'src/cards'/path;dest.parent.mkdir(exist_ok=True,parents=True)
  dest.write_text(expose((SRC/'src/card_definitions'/path).read_text()))
  out.append(f'''pub mod {num} {{
use crate::cards::*;
use rustmage_engine::cards::authored::{{CardRules,CardExtras,generic,one,object}};
use crate::cards::InitialCounters as Counters;
include!("cards/{path}");
}}''')
 (pack/'src/lib.rs').write_text('\n'.join(out)+'\n')
# Public facade remains the package rustmage; one dependency and one cdylib.
for p in (DST/'src').glob('*.rs'):p.unlink()
facade='''#![allow(dead_code,unused_imports)]
pub use rustmage_engine::*;
mod ffi;
mod catalogue {
use rustmage_engine::cards::*;
mod authored {
'''
for name,rows in sets.items():
 facade+='pub use rustmage_cards_'+name+'::{'+','.join(n for n,_ in rows)+'};\n'
facade+='pub static RULES: [&CardRules; 5487] = ['+','.join('&'+n+'::RULES' for n in re.findall(r'&([A-Za-z_0-9]+)::RULES',authored))+'];\nuse rustmage_engine::cards::authored::CardRules;\n}\n'
# No inherent impl of another crate's Card here; aliases and types are in the core.
assembly=generated[:generated.index('#[allow(non_upper_case_globals)]')]
assembly=assembly.replace('pub(super)','pub')
facade+=assembly
facade+='''
#[unsafe(no_mangle)]
pub static RUSTMAGE_CATALOGUE: rustmage_engine::cards::generated::Catalogue = rustmage_engine::cards::generated::Catalogue {
 version: CATALOGUE_VERSION, colors: &COLORS, definitions: &DEFINITIONS,
 names: &NAME_INDEX, rules: &authored::RULES,
 back_face, face_parent, physical_identity, token_info, emblem_info,
};
}
'''
(DST/'src/lib.rs').write_text(facade)
shutil.copy2(SRC/'src/ffi.rs',DST/'src/ffi.rs')
# Private module paths needed by existing FFI (these aren't consumer imports).
(DST/'src/lib.rs').write_text(facade+'\nuse rustmage_engine::{cards,engine,capacity};\n')
# Test target separate from all production crate compilation.
tests=DST/'crates/card-tests';(tests/'src').mkdir(parents=True)
shutil.copytree(SRC/'src/testing',tests/'src/testing')
(tests/'Cargo.toml').write_text('''[package]
name = "rustmage-card-tests"
version = "0.1.0"
edition = "2024"
publish = false
[dependencies]
rustmage = { path = "../.." }
rustmage-engine = { path = "../engine" }
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["arbitrary_precision"] }
''')
(tests/'src/lib.rs').write_text('''#![allow(dead_code, unused_imports)]
use rustmage::*;
pub use rustmage_engine::matched_card_benchmark;
mod cards {
 pub use rustmage_engine::cards::*;
 #[path = "../testing/bindings.rs"] pub mod coverage_bindings;
 #[path = "../testing/creature_bindings.rs"] pub mod creature_coverage_bindings;
 #[path = "../testing/land_bindings.rs"] pub mod land_coverage_bindings;
}
mod engine {
 use rustmage_engine::engine::*;
 use rustmage_engine::rng::Rng;
 use rustmage_engine::cards::*;
 #[path = "../testing/mod.rs"] pub mod testing;
}
''')
# Rust resolves #[path] beneath the inline-module directory, which must exist.
(tests/'src/cards').mkdir();(tests/'src/engine').mkdir()
profiles=base[base.index('# Keep repeated'):]
(DST/'Cargo.toml').write_text('''[workspace]
members = ["crates/engine", "crates/cards-msh", "crates/cards-other", "crates/card-tests"]
resolver = "3"
[package]
name = "rustmage"
version = "0.1.0"
edition = "2024"
[lib]
crate-type = ["rlib", "cdylib"]
[dependencies]
rustmage-engine = { path = "crates/engine" }
rustmage-cards-msh = { path = "crates/cards-msh" }
rustmage-cards-other = { path = "crates/cards-other" }
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["arbitrary_precision"] }
'''+profiles)
print(json.dumps({'output':str(DST),'cards':{k:len(v) for k,v in sets.items()}}))

# The original binding helpers now query the engine's imported immutable registry.
for name in ('bindings.rs', 'creature_bindings.rs', 'land_bindings.rs'):
    path = tests/'src/testing'/name
    path.write_text(path.read_text().replace('generated::DEFINITIONS', 'generated::catalogue().definitions'))
# Tracing must be active in a dependency build too, including stats-only cases.
for path in (core/'src').rglob('*.rs'):
    text = path.read_text()
    text = re.sub(r'#\[cfg\(test\)\]\s*(crate::matched_card_benchmark::)', r'\1', text)
    path.write_text(text)
