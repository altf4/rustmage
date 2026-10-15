# Independent review: life-gain and creature-death events

PR: https://github.com/altf4/rustmage/pull/51
Implementation reviewed: cbc520f against main.

Two independent read-only subagents reviewed this milestone:

- Rules: positive-gain dispatch and history, lifelink grouping and strike timing,
  simultaneous death observers, source LKI, controller/other/nontoken/subtype
  filters, chained triggers, source-name normalization, all 23 admissions and
  complete-text gating.
- Library: owned event history and source indexes, registration/removal/accounting,
  simultaneous death/LKI dispatch, snapshot and continuation replay, schema 21
  privacy, native/Python integration, compiler restrictions and all 23 admissions.

Both reported no actionable findings. Both independently passed 270 Rust tests
(262 unit and eight integration), 50 compiler tests and catalogue consistency.
The library reviewer also passed all 30 release Python tests and 20 native games
with 55 life-gain events, 306 observed triggers and 324 replayed boundaries.
Neither reviewer edited files, mutated Git or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and the full Rust/compiler/release Python suites. One hundred
supported Standard games completed naturally: 50,225 decisions, 347 life-gain
events, 1,586 observed triggers, 109 sacrifices and 1,712 cloned/replayed
boundaries. Native fixtures cover zero gain, late observers, turn resets,
source incarnations, APNAP replay, separate and grouped lifelink sources,
double-strike windows, simultaneous dying observers, token and subtype filters,
departed source lifelink, entry-debuff deaths and sacrifice chains. An admitted-card
fixture verifies source-index registration, removal and restoration for every
executable definition.

Coverage is 634/5,164 Standard identities, six regression cards and 35 token
factories. Wider events and restrictions, variable amounts, replacements/layers,
remaining card rules and full native RL/search integration remain outstanding.
These reviews do not certify full Standard conformance or establish an XMage
performance comparison.
