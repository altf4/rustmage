# Restricted counterspell targets

Issue: [#106](https://github.com/altf4/rustmage/issues/106).
The in-process catalogue implements seven additional complete Standard bodies:

| Card | Complete behavior |
| --- | --- |
| Annul | Counter an artifact or enchantment spell |
| Flashfreeze | Counter a red or green spell |
| Precise Redaction | Counter a white or black spell |
| Spell Snare | Counter a spell with mana value exactly two |
| Disdainful Stroke | Counter a spell with mana value at least four |
| Spectral Interference | Counter an artifact or creature spell unless its controller pays four generic mana |
| Voracious Greatshark | Flash, 5/4; entry trigger counters an artifact or creature spell |

Full Oracle bodies and Java source hashes are pinned in
[counter-filter-admissions.json](counter-filter-admissions.json) against XMage
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Coverage becomes 1,040/5,164
Standard identities, with six regression definitions, 48 token factories and one
emblem factory. Whole-body compiler tests reject residual unsupported abilities.

Immutable target rules reuse spell type masks and add a color-intersection mask
and inclusive `u64` mana-value bounds. Action generation, submitted-action
validation and resolution revalidation all use the shared stack matcher. Only
actual spells match; source identity rejects self-targets and bound incarnation
identity prevents a departed/recast card from becoming the old target again.
Hybrid spells retain all printed colors and their printed mana value regardless
of which hybrid allocation paid the cost. Colorless spells match neither color
union; multicolor and multi-type spells match either relevant arm.

These rules follow CR 202.2, 202.3, 608.2b and 701.6 of the
[official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
The supported normal-face fixed/hybrid spell subset has no changing stack
characteristics. X costs, copies, alternative faces and color/type-changing
mechanics remain gated until their stack characteristics are modeled. Static
printed characteristics must not be used to admit those future mechanics.

Spectral Interference reuses the owned resolution-mana continuation: its target's
controller may activate mana abilities, pay exactly four generic mana or decline.
Countering removes the spell without refunding costs or running its body. Paid
spells remain on the stack. Greatshark's trigger survives its source's departure;
without a legal target it leaves no resolving counter ability. No mutable state,
continuation, observation or ABI field is added. Observation 41, ABI 1 and control
protocol 2 remain unchanged, and snapshots include all existing pending choices.

Author checks pass 713 Rust tests (701 unit and twelve integration), 129
compiler/data tests and 90 fresh release Python tests, catalogue consistency,
rustfmt and Clippy with warnings denied. Nine native regressions cover both color
arms, multicolor/colorless, exact mana-value boundaries, each type-union arm,
hybrid payment, forged illegal actions, self-targeting, stale incarnations,
resolution revalidation, tax payment/decline, source departure, no-target entry
and deterministic clone/snapshot replay. Three Python methods cover five direct
counterspells, both four-mana outcomes and Greatshark's flash/entry trigger through
the actual native library with reusable snapshots.

The public `standard_counter_filters` example completes 100 legal Standard games:
66,082 decisions, 2,395 independently checked public facts, 16,556 cloned/replayed
boundaries and 20,468 maximum initialized state bytes. All seven bodies counter
spells and both Spectral payment outcomes occur. The policy holds response mana,
responds once per opposing spell, delays attacks until turn twelve and may cast a
remaining Greatshark without a target after turn twenty. These are workload
choices; engine legality is unchanged. The checker separately reads printed type
lines and mana symbols and checks offered target actions, resolution, payment and
entry facts. Focused tests cover illegal and stale paths beyond this workload.

An unpinned 1,000-game baseline records 571,049 decisions, roughly 392k native
decisions/s, 687 games/s, 118 ns clone and 106 ns restore, with 19,648 initial
logical state bytes. The benchmark includes native policy and observations, and
excludes model inference, tensor encoding and JSON. State byte estimates exclude
capacity, allocator/COW metadata, static definitions and external snapshots.
These timings establish no speed change or Java parity claim. Full Standard and
broader native RL/search acceptance remain incomplete; 4,124 identities remain
unsupported and the full goal stays active.
