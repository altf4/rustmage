# Effective-power blocking restrictions

Issue: https://github.com/altf4/rustmage/issues/132

Four complete Standard bodies are admitted:

| Card | Complete implemented behavior |
| --- | --- |
| April O'Neil, Kunoichi Trainee | {1}{W}, legendary 2/2 Human Ninja; entry scry 2; cannot be blocked by creatures with power 3 or greater |
| Cavern Stomper | {4}{G}{G}, 7/7 Dinosaur; entry scry 2; {3}{G} prevents creatures with power 2 or less blocking it this turn |
| Prehistoric Pet | {W}, 1/2 Dinosaur Ninja; cannot be blocked by creatures with greater power; {1}{W}, tap to return another target controlled creature to its owner's hand, only during its controller's turn |
| Verdant Outrider | {2}{G}, 4/2 Human Knight; {1}{G} prevents creatures with power 2 or less blocking it this turn |

Full Oracle text and four pinned Java hashes are in
[power-blocking-admissions.json](power-blocking-admissions.json), at XMage revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Catalogue fingerprint:
`7009f10fbc93483fa7817fb51017f1809b225a80e1f05fbe246f73ffadb02498`.
Coverage is **1,122/5,164** Standard identities, six regression definitions,
49 token factories and one emblem; **4,042** Standard identities remain unsupported.

Block-pair legality now evaluates live signed derived power when an attacker
has a relevant restriction. Strict greater-than compares blocker and attacker;
the fixed boundaries reject blockers at power <=2 or >=3, respectively.
Counters, equipment, continuous and temporary modifiers use the existing derived
stats path. Equality remains legal for greater-power evasion. Negative power is
compared directly rather than clamped. Unrestricted pairs skip power evaluation;
attacker keywords are cached within the predicate. Flight/reach, token exclusion,
blocker eligibility and menace/minimum/maximum completion checks remain shared.
A power/evasion change after declaration does not undo an existing block.

Four new masks use the existing `u32` flags:

| Constant | Mask | Meaning |
| --- | --- | --- |
| SKULK | 1,048,576 | Formal skulk keyword |
| NO_POWER_LE_TWO_BLOCKERS | 2,097,152 | Blocker power <=2 prohibited |
| NO_POWER_GE_THREE_BLOCKERS | 4,194,304 | Blocker power >=3 prohibited |
| NO_GREATER_POWER_BLOCKERS | 8,388,608 | Equivalent greater-power evasion text without granting skulk |

Prehistoric Pet does **not** have skulk: its functional rule uses the last mask,
so keyword-based effects can distinguish them. The formal keyword shares the
same block comparison and is covered through explicit IR fixtures. No complete
Standard skulk-keyword body is newly admitted: Ingenious Prodigy's remaining X
cost/entry/upkeep machinery is still gated. This-turn text grants use existing
owned temporary flags and expire at cleanup; intrinsic restrictions remain.

The compiler accepts only these exact power clauses and this-turn source grants,
authenticates named static clauses against the source card's existing full/short
legendary names, and adds the exact mandatory another-controlled-creature return
target. Arbitrary thresholds, longer durations and unknown residual clauses stay
gated. Old Fat Spider's additional opponent-target trigger is not implemented by
this slice. Azure Beastbinder, Delney, Gate Colossus, Foggy Swamp Vinebender,
Bristlebane Outrider, Rust-Shield Rampager and Sandman's other dependencies also
remain gated; evasion alone never makes their bodies executable.

Rules references: CR201.5, 509.1b/c, 509.1h and 702.118 in the pinned
[Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Observation schema **48** versions the added flag vocabulary. ABI 1 and control
protocol 2 remain unchanged. No persistent state field is added; initial logical
state remains **19,784 bytes**. Logical initialized accounting excludes capacity,
allocator/COW metadata, statics, observations and external snapshots.

Validation passes **840 Rust tests** (828 unit and 12 integration), **157
compiler/data tests** and **113 rebuilt-release Python tests**, formatting,
Clippy with warnings denied, catalogue and diff checks. Seven new native tests
cover equality, signed/wide boundaries, formal versus textual evasion, counter/
equipment/temporary changes, cleanup, flight/reach/token restrictions, exact
menace completion, illegal actions, post-declaration changes, private scry and
Prehistoric Pet's whole activated body. The Python suite plays all four cards
to a blocker decision, independently checks live-power action sets, rejects a
forged block and branches/restores the valid block. Pet's return ability also
runs through the existing native binding recovery test.

`cargo run --release --example standard_power_blocking -- 100` plays 100 complete
games with legal identical 60-card Standard decks. The independent pair checker
combines observed signed powers with the public evasion masks and compares every
eligible pair with the offered actions; these decks have no minimum/maximum
blocker requirements. Results: **49,001 decisions, 2,819 pair checks, 74 rejected
illegal blocks, 332 resolved restriction grants, 62 Pet returns, 298 private scry
choices, 21,381 clone/restore/replays and 20,848 maximum initialized bytes**.
Every new card enters; each restriction has both allowed and forbidden pair
outcomes. The three activated bodies resolve. An external per-turn set limits
policy repetition without constraining engine legality. Native tests separately
cover menace completion and formal skulk.

A fresh 1,000-game Freeform baseline yields 571,049 decisions at 407,769.57
decisions/s and 714.07 games/s, clone 166.56 ns and restore 114.34 ns. This
unpinned timing sample includes smoke policy and observation construction and
excludes model inference, tensor encoding and JSON. It establishes neither a
performance change nor XMage parity.

Full Standard and native RL/search acceptance remain incomplete and active.
