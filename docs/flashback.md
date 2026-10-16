# Flashback casting

Issue [#197](https://github.com/altf4/rustmage/issues/197) adds nine complete Standard bodies on base `70439f52b87fea0dcda24e894b10d387e62c4226`. This branch implements 1,407 of the 5,164 pinned legal cards, with six additional regression cards; 3,757 Standard bodies remain unsupported. Its catalogue fingerprint is `453a317d38d171228793eafca13e9a0b91dc76cf6a67f8d3837f29bf63e1af01`.

| Card | Printed cost | Flashback cost | Entire effect |
| --- | --- | --- | --- |
| Auron's Inspiration | {2}{W} | {2}{W}{W} | All attacking creatures get +2/+0 this turn |
| Bestial Incursion | {3}{G} | {5}{G} | Create a 4/4 green Beast with trample |
| Dig Site Inventory | {W} | {W} | Counter on an owned target creature; vigilance this turn |
| Dreams of Laguna | {1}{U} | {3}{U} | Private surveil 1, then draw |
| Laughing Mad | {2}{R} | {3}{R} | Discard another hand card as a cost, then draw two |
| Predictive Preparations | {1}{W} | {3}{W} | Counter on each of one or two target creatures |
| Revenge of the Rats | {2}{B}{B} | {2}{B}{B} | Tapped 1/1 black Rats equal to owned creature cards in the graveyard |
| Think Twice | {1}{U} | {2}{U} | Draw one |
| Tome Blast | {1}{R} | {4}{R} | Two damage to any target |

The compiler extracts one exact fixed-mana flashback line from a normal instant or sorcery. It then compiles every residual effect, qualifier and mandatory cost through the existing closed grammar. Hybrid, variable, nonmana flashback costs, extra flashback qualifications, conditional origin effects and unsupported residual sentences remain gated. No partial body becomes executable. Every original oracle byte and the compiler source participate in the catalogue fingerprint; changing only a flashback cost changes the fingerprint.

The [pinned Java manifest](flashback-admissions.json) records the complete constructors, their direct implementation dependencies and critical shared flashback, target, discard and graveyard-count logic: 109 SHA-256 entries across 40 files, verified against both revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7` and the local Java checkout. The rules authority is [Comprehensive Rules, effective September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf), particularly 601.2a–c/f–h, 202.3 and 702.34a. Flashback changes payment and permission; printed colors, type and mana value remain the same. The pinned Java replacement also binds the spell's zone-change counter and replaces every departure from the stack.

## Library and RL contract

Observation schema **53** adds `SpellView.cast: Option<CastInfo>`. Actual spells carry `CastInfo { origin: Zone, method: CastMethod }`, where the methods are `Normal` and `Flashback`; activated and triggered packets carry `None`. The record remains available through announcement, stack priority and suspended resolution. The C ABI remains version 1 and the control protocol remains schema 2. Current executable consumers use the new observation schema; older milestone documents retain their historical schema and measurements.

`Card::flashback_cost()` exposes immutable fixed alternative mana requirements. `Action::AnnounceFlashback { card }` is offered only for owned graveyard cards with that permission, correct ordinary timing, available targets and a fundable complete cost. Existing direct normal cast actions remain hand-only. Graveyard preflight excludes the departing spell itself from future target candidates. Targets are bound after it moves to the stack, before additional costs and mana payment; the cast event occurs only after payment completes.

The selected base cost is reduced before payment is locked. Zero-generic alternatives skip generic-reducer scans. Reducer eligibility still sees the original card characteristics. Removing the old printed-generic early return permits generic-heavy alternatives on printed-zero-generic cards. Tests include actual Tome Blast plus Stormcatch Mentor, a synthetic selected generic cost on Dig Site Inventory, and a six-generic Defibrillating Current payment reduced to five while retaining printed mana value six and its cast trigger. An actual flashback Tome Blast remains mana value two and does not trigger Lurking Lizards.

A boolean on the exact current Object incarnation records whether its stack departure must be replaced with exile. The central zone transition applies it for **every** destination, reports the actual destination once, and clears it on departure. It works when `resolve_top` has already popped the packet and when the interpreter temporarily owns a private resolution frame. Resolving, countered, all-targets-illegal, bounced or otherwise removed flashback spells are exiled. A later same-ID incarnation does not inherit the marker. No stack scan, new mutable registry, event-history allocation or clone of target lists is required. Existing COW arena/snapshot semantics cover the flag and the owned cast packet; the initial logical state remains **19,880 bytes**.

Two new immutable token recipes describe the Beast and ordinary Rat. All 58 old recipes remain unchanged. Factory IDs are scoped to the catalogue fingerprint and can shift when sorted recipe hashes gain entries. Normalizing `Card(n)` references to immutable factory identities proves exactly nine changed old Definition rows, 5,233 unchanged rows, and no regressions. All 5,183 ordinary card IDs remain stable. Every previous immutable helper row is preserved; the only additional rows are the nine flashback costs and Laughing Mad's mandatory discard. A preexisting cannot-block Rat test now explicitly selects the correct recipe rather than the first shared printed token name.

## Validation and reproduction

```sh
cargo test
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_flashback -- 100
cargo run --release --bin benchmark -- 1000
```

The complete checks pass 1,232 Rust tests (1,220 unit and 12 integration), 235 compiler tests and 153 fresh-release Python tests. Formatting, all-target Clippy with warnings denied, generated-catalogue verification and whitespace checks also pass.

Native boundaries exercise both origins for all nine complete effects, private surveil, mandatory discard and mana funding, ownership, sorcery and response timing, illegal direct-cast bypasses, all-targets-illegal and partial resolution, an actual Counterspell, every requested stack destination with its packet removed, marker reset on same-ID reentry, printed mana values and reductions, self-target preflight, capacity failures, clone/restore and stale action indices. Python directly loads the release library and independently checks each complete effect from both origins, including private decisions and replay.

The 100-game workload uses only public APIs and Standard-legal 60-card/four-copy decks. Its independent witnesses compute counters, temporary combat boosts, keywords, token characteristics/counts, life changes, draws, paid discards and actual spell destinations. It checks both origins separately and exercises private suspended resolution and stale-index rejection. It completes naturally; its fixture nontermination guard does not limit the engine's action space.

Two runs produce 47,896 decisions, normal/flashback casts `[[7,2],[20,17],[15,13],[31,29],[28,28],[12,9],[5,2],[26,24],[22,19]]`, and verified effects `[[7,2],[20,17],[15,13],[31,29],[28,28],[12,9],[5,2],[26,24],[21,19]]`. The missing final normal Tome Blast ends its game before an effect witness is collected. There are 330 independently checked effect units, 56 paid discards, 220 private decisions and 7,988 replay boundaries; maximum initialized logical state is 20,900 bytes.

A native 1,000-game smoke run executes 571,049 decisions at roughly 410,000 decisions/second, with 128.77 ns clone and 124.71 ns restore. These host-dependent measurements include the existing smoke policy/observation construction, exclude neural inference and JSON, and do not claim XMage parity or a Java speed comparison.
