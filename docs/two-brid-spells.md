# Exact two-brid spell costs

Issue: [#108](https://github.com/altf4/rustmage/issues/108).
Normal-layout spell costs now support each monocolored hybrid symbol `{2/W}`,
`{2/U}`, `{2/B}`, `{2/R}` and `{2/G}`, including combinations with ordinary and
two-color hybrid symbols. Each can be paid with one mana of its color or two mana
of any type. The printed spell retains that color and a mana-value contribution
of two regardless of the chosen payment. These rules follow CR 107.4e, 202.3f
and 601.2b in the
[official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

| Card | Complete admitted body |
| --- | --- |
| Kin-Tree Severance | Exile a targeted permanent with mana value at least three |
| Reputable Merchant | Entry/death puts a +1/+1 counter on a targeted owned creature |
| Temur Tawnyback | Entry draws a card, then privately discards a card |

All three have three distinct two-brid symbols: eight unique equivalents whose
payments use three through six mana. Full Oracle bodies and pinned Java hashes
are in [two-brid-admissions.json](two-brid-admissions.json), against XMage
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Coverage becomes 1,043/5,164
Standard identities, six regression definitions, 48 token factories and one
emblem factory. Recognition of a cost never bypasses complete-body admission.
Converge/mana-spent dependencies, other missing two-brid bodies, hybrid ability/
cycling/ward costs, Phyrexian, snow and X costs remain gated.

The compiler deduplicates exact equivalents including each generic component.
Immutable alternative rows now contain `ManaCost`, allowing a different generic
requirement per choice. Colored components sort before generic components: the
canonical row uses the generic side of every two-brid symbol, preserving printed
mana value. Every other row retains its exact payable cost. Components fit `u32`
and at most 65,536 equivalents are represented; definitions exceeding those
bounds fail closed. Ordinary fixed and two-color hybrid equivalents retain their
previous ordering, allocations and printed values.

Staged casting reuses the owned announcement: choose a payable equivalent before
targets/mana abilities, then fund and pay that committed cost. Floating enough mana
for another equivalent does not permit switching after commitment. Prefunded
shortcuts enumerate exact payments over all equivalents and emit duplicates once.
The fixed-cost fast path remains allocation-free. Existing funding, target, mode,
additional-cost, rollback and private-choice machinery is reused. No mutable state,
observation, ABI or control field is added: schema 41, ABI 1 and protocol 2 remain
unchanged. Cost alternatives are shared static definitions, outside snapshots.

Author verification passes 719 Rust tests (707 unit and twelve integration), 131
compiler/data tests, 92 fresh release Python tests, consistency, rustfmt and
Clippy with warnings denied. Six new native tests compare all payments against
independent exhaustive assignments over 270 small pools, exercise all eight
committed costs, enforce affordable choices and fixed commitments, check printed
mana value against counter targeting, resolve Merchant's entry/death after source
departure, and restore Tawnyback's private discard without repeating its draw.
Existing hybrid invariants now distinguish payable equivalents from printed mana
value; every supported Standard printed cost/value still matches frozen Oracle.
Compiler tests cover mixed symbols, repeated-symbol deduplication, bounds,
unsupported symbols/ability costs and complete-body gates. Two Python methods
exercise both three- and six-mana exile paths, entry counters and private loot
through the actual native library with reusable snapshots.

The `standard_two_brid` public example completes 100 legal Standard games:
86,710 decisions, 602 committed choices, 4,022 independently checked public facts,
30,593 cloned/replayed boundaries and 20,484 maximum initialized state bytes.
All three bodies, all four generic components (zero/two/four/six) and all four
payment totals (three/four/five/six) are sampled. The policy prefers different
available costs by seed, holds the generic-six path until six mana is available,
and delays attacks until turn sixteen; blocks remain available. These policy
choices preserve engine legality. The checker separately validates cost/color
structure, spent mana, exile/counter facts and draw-before-discard continuation.
Focused fixtures cover stale revisions and illegal/hidden-state paths beyond the
workload.

An unpinned 1,000-game baseline measures 571,049 decisions, about 406k native
decisions/s, 711 games/s, 113 ns clone and 110 ns restore. Initial logical state
remains 19,648 bytes. The benchmark includes native policy/observations and excludes
model inference, tensor encoding and JSON; byte estimates exclude capacity,
allocator/COW metadata, static definitions and external snapshots. These timings
establish no speed change or Java parity claim. Full Standard and broader native
RL/search acceptance remain incomplete, with 4,121 unsupported identities; the
full goal stays active.
