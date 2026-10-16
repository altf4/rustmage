# Nonmana Ward payments

Issue: https://github.com/altf4/rustmage/issues/140
Pinned Standard: October 15, 2026. Four complete bodies bring executable coverage
to **1,157/5,164**, leaving **4,007 unsupported**. Six regression identities,
51 token factories and one emblem remain separate.

## Complete bodies and sources

`nonmana-ward-admissions.json` records exact Oracle text and 17 SHA-256 provenance
entries across ten unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All four card classes, WardAbility,
CounterUnlessPaysEffect and the discard/sacrifice/life/Swampcycling classes were read.

| Card | Whole body |
| --- | --- |
| Spectral Snatcher | Discard-one Ward and independent hand-zone Swampcycling {2} |
| Sunset Saboteur | Menace, discard-one Ward and mandatory attack counter on an opposing creature |
| Vein Ripper | Flying, sacrifice-creature Ward and targeted opponent drain for each creature death, including its own |
| Sire of Seven Deaths | Reach, first strike, vigilance, menace, trample, lifelink and pay-seven-life Ward |

Compiler support remains exact: standalone discard-one, sacrifice-one-creature and
fixed unsigned life costs through 65,535. Mixed/random/filtered/scaled costs,
residual clauses, granted Ward, copy/control changes, replacement/prevention and
countering exceptions remain gated. Tragedy Feaster's inverse infusion condition
is still unsupported. Separate repeated Ward instances are preserved.

The two small effect-parser extensions accept whole opposing-creature counter
sentences and whole targeted-opponent loss plus controller gain sentences. No
additional card bodies are admitted incidentally.

## Rules and library behavior

Ward triggers after an opposing spell or ability targets the permanent; it is not
a targeting restriction. The existing target hook captures the exact original
stack ID and controller, independently of source departure or re-entry. A new
`CounterRelated` instruction removes only that captured spell or ability on
nonpayment. It cannot alias a later spell using the same physical card slot.

Object payments reuse the owned `EffectChoice` continuation. They require the full
cost, choose only the payer's hand card or controlled creature, and allow decline.
With insufficient resources, the program follows the unpaid branch without
pretending to make a partial payment. Discards remain private until committed;
sacrifices are public. Existing discard/death events queue their triggers during
the payment and announce them after the resolving Ward finishes.

Life payments reuse `EffectOptional`: a `MayPayOpponentLife` instruction checks
that the payer can afford the complete cost, then offers that player the choice.
Acceptance executes the exact life loss before skipping the counter; declining
or inability to pay executes the counter. Paying zero is allowed independently of
life total. Life paid updates the actual loss history and may be lethal when the
payer's total equals the cost. Capacity failures restore the complete state and
revision, including the still-owned choice.

The program uses the Ward controller's other player as payer: the engine currently
supports two players and gates control-changing effects. Target capture verifies
that the original controller is that opponent. Wider multiplayer/control machinery
must revisit this assumption when admitted.

No mutable state fields or public shapes were added. Observation schema remains
49, C ABI 1 and control protocol 2. Initial logical state remains **19,784 bytes**.
Complete snapshots own every private choice and program counter. Restoring advances
the revision and rejects stale indexed actions. Definitions and indices are local
to catalogue fingerprint
`830ee35e5c8729da30e0f8f55d86cb5181dcfdf9798e9b1d501b388745b4b9ff`.

Rules references: Comprehensive Rules 118.3, 118.12, 119.4, 400.7, 603, 608 and
702.21 from the September 25, 2026 text linked by the
[official rules page](https://magic.wizards.com/en/rules). Vein Ripper's simultaneous
own/other death behavior also follows the
[official MKM release notes](https://media.wizards.com/2024/downloads/MKM_Release_Notes_82nnDTWVBdD/EN_MTGMKM_ReleaseNotes_20240125.pdf).

## Verification and performance

- **877 Rust tests**: 865 unit and 12 integration. Eight new rules methods cover
  both payer seats, payment/decline/inability, private choices, illegal actions,
  exact quantities, source/stack incarnations, overflow rollback, attack targeting,
  simultaneous deaths and Swampcycling. Existing mana-Ward tests also pass.
- **166 compiler/data tests** include exact program branches, numeric bounds,
  full-body rejection and all four pinned admissions. Older negative fixtures now
  use genuinely unsupported wider costs; newly supported bodies are positive cases.
- **120 fresh release Python tests**, including eight native C ABI scenarios
  (four cards, payment and decline), private projections, illegal-action atomicity,
  clone/restore and stale-revision rejection.
- Format, all-target Clippy with warnings denied, catalogue consistency and diff
  checks pass.

`cargo run --release --example standard_nonmana_ward -- 100` uses two legal 60-card
Standard decks and requires all four new entries/payment kinds, both printed
trigger bodies and Swampcycling. The policy retains Snatcher on even seeds and
cycles on odd seeds, prefers opposing Ward targets, and declines on every third
seed. These are workload policy choices; the engine's legal action set is unchanged.

Two runs reproduced **58,264 decisions, 337 owned cost choices, 175 payments,
162 declines, 52,968 body checks, 11,854 clone/restore replay boundaries and
21,196 maximum logical initialized bytes**. Entry counts in table order were
40/240/98/123; payment-choice counts 9/235/31/62. There were 117 attack-counter and
124 death-drain resolutions, plus 102 completed Swampcyclings. The checker verifies
payer/privacy, counter associations, exact life payment/history and printed
keywords; native and Python scenarios cover the remaining exact effects.

An unpinned 1,000-game initial Freeform smoke run recorded 571,049 decisions,
390,461 decisions/s, 683.76 games/s, 110.71 ns clone and 108.19 ns restore. This
includes native decisions/observations and excludes inference, tensors, JSON and
Python. It is a bounded smoke measurement, not Java parity or a full Standard
throughput claim. Byte counts exclude capacities, allocator overhead, immutable
definitions, observations and external snapshots.

Full Standard and broader native RL/search acceptance remain incomplete.
