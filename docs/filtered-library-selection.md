# Filtered library selection

Issue [#205](https://github.com/altf4/rustmage/issues/205) extends owned private library selection with immutable subtype unions, kind exclusions and conjunctions. Twelve complete Standard bodies become executable; unsupported residual abilities still reject the whole body.

| Card | Complete rules text |
| --- | --- |
| Ashe, Princess of Dalmasca | Whenever Ashe attacks, look at the top five cards of your library. You may reveal an artifact card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Casey Jones, Jury-Rig Justiciar | Haste When Casey Jones enters, look at the top four cards of your library. You may reveal an artifact card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Cowabunga! | Look at the top four cards of your library. You may reveal a Mutant, Ninja, Turtle, or land card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Eclipsed Boggart | When this creature enters, look at the top four cards of your library. You may reveal a Goblin, Swamp, or Mountain card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Eclipsed Elf | When this creature enters, look at the top four cards of your library. You may reveal an Elf, Swamp, or Forest card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Eclipsed Flamekin | When this creature enters, look at the top four cards of your library. You may reveal an Elemental, Island, or Mountain card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Eclipsed Kithkin | When this creature enters, look at the top four cards of your library. You may reveal a Kithkin, Forest, or Plains card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Eclipsed Merrow | When this creature enters, look at the top four cards of your library. You may reveal a Merfolk, Plains, or Island card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Frontier Seeker | When this creature enters, look at the top five cards of your library. You may reveal a Mount creature card or a Plains card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Master Piandao | First strike Whenever Master Piandao attacks, look at the top four cards of your library. You may reveal an Ally, Equipment, or Lesson card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Staunch Crewmate | When this creature enters, look at the top four cards of your library. You may reveal an artifact or Pirate card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |
| Thundertrap Trainer | Offspring {4} (You may pay an additional {4} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.) When this creature enters, look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. |

The closed compiler dictionary accepts only audited noun phrases. Runtime predicates compose `Subtype`, `AnyOf`, `AllOf` and `ExcludingKinds` without mutable filter state or per-candidate allocation. The Eclipsed cycles accept the named creature subtype or either named basic land subtype. Frontier Seeker requires a Mount creature or a Plains card. Master Piandao accepts Ally, Equipment or Lesson; Staunch Crewmate accepts artifact or Pirate; Thundertrap Trainer excludes both creature and land. Changeling supplies creature types even on Kindred noncreature cards, but cannot supply land or noncreature subtypes. Native metadata-only Crib Swap probes verify this without admitting its unsupported spell body.

Selection retains the exact owned continuation, deferred whole-frame commit, actor-only observations, optional and zero-eligible completion, prescribed reveal behavior and deterministic random-bottom ordering described in [private library selection](library-selection.md). Attack-triggered Ashe and Master Piandao remain independent of their departed source. All eighteen hybrid payment equivalents are exercised. Thundertrap Trainer preserves its complete optional offspring cost, immutable 1/1 copy and the copy's own selection ability; neither the parent's choices nor paid status are inherited. Partial implementations such as Dragonologist remain gated.

## Catalogue and memory

On main `092dda1101f9c795bd884986fbf4ca5277cf7262`, support becomes **1,467 / 5,164** Standard cards, with **3,697** gated identities, six non-Standard regression cards, **72 token factories**, one emblem and **5,256 immutable definitions**. The fingerprint is `5186eeda9090b172445c8aa4b4716bd19949922e5e23258dba74db4b5490bf2e`. Observation schema remains **56**, C ABI 1 and control protocol 2.

Canonical comparison changes exactly twelve previously unsupported definitions, preserves **5,243** old definitions and adds one Trainer copy factory. All **71** previous factory recipes and metadata, **442** previous typed helper arms, **5,255** previous color masks and **5,183** ordinary IDs retain their semantics; four helper arms are added. Factory numeric IDs remain fingerprint-scoped. [Provenance](filtered-library-selection-admissions.json) records **122 SHA-256 entries across 29 Java files**, checked against pinned revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7` and the source checkout.

No public observation fields or mutable game-state fields are added. Initialized logical state remains **19,896 bytes**, excluding capacity, allocator overhead and immutable definitions. A 1,000-game native smoke run completed **571,049** decisions at about **412,000 decisions/s**, with clone/restore around **124/129 ns**. These host-dependent measurements include smoke policy and observations, exclude model inference, tensor encoding and JSON, and are not a Java comparison.

## Validation

Integrated checks pass **1,299 Rust tests** (1,287 unit and 12 integration), **248 compiler/data tests**, **160 fresh-release Python tests**, all-target Clippy with warnings denied, formatting, generated catalogue and whitespace checks. Twelve new native tests cover both seats, all full bodies, exact subtype/kind combinations, optional and short libraries, source departure, suppression, paid/unpaid Trainer, independent copy decisions, stale references, capacity rollback and owned RNG/COW snapshots. Three compiler tests retain full-body negative gates. The Python method exercises 26 public scenarios.

The [Standard workload](../examples/standard_filtered_library_selection.rs) completes **100 legal games** and reproduces **71,080 decisions/replays**. Selection-frame counts in manifest order are `[33,26,27,30,21,25,22,21,28,48,24,31]`, split **171/165** between players. It checks **213** selected/revealed cards, **213** deferred partial commits, **61** zero-eligible windows, **142** eligible and **35** rejected Changeling candidates, **107** noncreature checks, **305** land checks, **25** off-color land rejections and **16** plain-artifact rejections. Trainer has **11 unpaid/10 paid casts** and **21 parent/10 copy selection frames**. Ashe/Piandao have **33/48 attacks**, including **2/7** departed-source frames. Maximum initialized logical memory is **20,704 bytes**. The adjacent forage workload retains its **65,729** decisions/replays and all prior event/payment vectors.

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_filtered_library_selection -- 100
cargo run --release --example standard_forage -- 100
```
