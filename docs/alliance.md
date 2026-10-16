# Alliance complete-body admissions

Issue: https://github.com/altf4/rustmage/issues/176

Six complete bodies bring executable coverage to **1,321/5,164** pinned Standard
cards, with **3,843 unsupported**. Six non-Standard regression cards, 57 token
factories and one emblem remain separate. The full Standard migration remains
in progress; the target is pinned to October 15, 2026.

| Card | Complete body |
| --- | --- |
| EPF Point Squad | 2/1 Human Soldier with three exact hybrid payment options; another owned creature entering adds a +1/+1 counter to this source |
| East Wind Avatar | 2/4 flying, vigilance; another owned creature entering gives this source +1/+0 until end of turn |
| Mighty Mutanimals | 2/1; entry creates a 2/2 red Mutant; another owned creature entering adds a +1/+1 counter to a targeted owned creature |
| Mutant Town Musicians | 2/4 trample; another owned creature entering gives this source +1/+0 until end of turn |
| Raphael, Tough Turtle | Legendary 1/3; another owned creature entering deals one damage to target opponent |
| Slash, Reptile Rampager | Legendary 7/5; another owned creature entering deals two damage to each opponent; attacking creates a 2/2 red Mutant |

## Compiler and engine

Alliance is a cosmetic ability word, without independent rules meaning
([CR207.2c](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)).
The compiler recognizes only the exact `Alliance — ` label, then requires the
entire underlying event, effect and every other clause to compile. The label
does not substitute an event or erase an unsupported clause. Unsupported event
subjects, entering-object pronouns, unknown effects and residual abilities
continue to gate the complete card.

All six use the existing `OtherCreatureEntersYou` observer. Self entry, opposing
creatures, lands and noncreature artifacts do not trigger it; each creature in
a token or simultaneous-entry batch does. Mighty Mutanimals' own token enters
after its source is present and triggers Alliance. Slash's attack-created token
also produces a separate Alliance trigger.

Existing APNAP placement, legal target selection, exact-incarnation
revalidation, source independence and state-based actions apply. A departed
source's targeted/damage ability can resolve, but old source growth/counters
cannot affect a new incarnation. Temporary growth expires; counters and
intrinsic abilities remain. Existing token recipes are reused. Production
runtime is unchanged; only the compiler label and generated immutable card
definitions change. State layout, observation schema **51**, C ABI **1** and
control protocol **2** are unchanged.

`alliance-admissions.json` records all six full Oracle bodies and **49 SHA-256
entries across 25 unique Java files**, pinned to
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All six constructors were read in
full; source hashes match the pinned blobs and read-only checkout. The manifest
includes Alliance and inherited entry observers, the shared filter definitions,
effects, targets, intrinsic abilities and existing red Mutant factory.

## Validation

Ten native tests exercise actual full casts for both seats, all three hybrid
payment options and insufficient payment rejection, intrinsic keywords,
self/opponent/noncreature exclusions, each token in a batch, simultaneous new
sources, Mighty token recursion without self triggering, APNAP with two owned
triggers per player, source reentry, illegal targets and exact target
revalidation, temporary expiry, atomic counter overflow and Slash's attack
trigger before combat damage. Semantic actions compare full cloned states and
restore/replay, including stale revision rejection.

Two compiler tests check exact label equivalence, malformed labels, unsupported
events/effects/pronouns, all six full admissions and unknown residual gating.
One fresh release Python test casts all six and independently checks the
resolved Alliance effects, Mighty/Slash token effects and native replay.

Two deterministic Standard100 runs use legal 60-card/four-copy decks containing
all six bodies, Helpful Hunter, Inspiring Overseer, Shock and 24 basic lands.
Both reproduce:

- **47,932 decisions**, casts `[90,72,96,160,211,114]` in table order.
- Triggers `[68,91,320,399,556,338]`.
- Independently verified Alliance effects `[68,91,223,399,547,221]`.
- Independently verified Mighty/Slash token programs: **96 / 94**.
- **160,425 public stat/ability pairs**, **12,065 replay boundaries** and
  **22,244 maximum initialized logical state bytes**.

The witness uses public before/after facts, manually models source/target
counter changes, growth, damage and tokens, and keeps a separate
exact-incarnation/turn growth ledger. Public stats and intrinsic abilities are
checked for every creature in both viewers' observations. It does not consume
the engine's private instruction definitions. Terminal-producing effects are
excluded from the resolved-effect counts.

Complete checks: **1,096 Rust tests** (1,084 unit and 12 integration), **209
compiler/data tests**, **143 fresh release Python tests**, all-target Clippy
with warnings denied, formatting, catalogue consistency and diff checks.

The separate 1,000-game native benchmark completed 571,049 decisions at about
312,638 decisions/s and 547 games/s; clone 133 ns, restore 130 ns. Initial
logical state remains **19,784 bytes**. These host-dependent timings include
policy and observation construction, exclude inference/tensors/JSON, and are
not an XMage parity comparison. Logical accounting excludes capacity/allocator
overhead, COW sharing, immutable definitions, observations and external
snapshots.
