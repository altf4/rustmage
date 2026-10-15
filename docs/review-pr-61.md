# Independent review: resolution mana payments and ward

PR: https://github.com/altf4/rustmage/pull/61
Implementation reviewed: 1afd089; reviewed fix: 15219ec.

Two independent read-only subagents reviewed this milestone:

- Rules: ward timing and target binding, captured stack identities, costs,
  counter destinations, paid branches, restricted targets and all 16 admissions.
- Library: owned payment/stack contexts, actors, mana abilities, interrupted
  entry continuations, snapshots, accounting, API privacy and compiler gates.

Both found the same P2 compiler scope error: all sentences after “If they do”
were treated as conditional, incorrectly skipping a later unconditional effect
when payment was declined. No current admission was affected. Commit 15219ec
requires one complete paid sentence, rejects ambiguous tails, adds the negative
regression and regenerates the catalogue fingerprint. Both independently
re-reviewed the fix and reported no remaining findings.

Both reviewers passed 366 Rust tests (358 unit and eight integration), 64 compiler
tests and catalogue consistency on the implementation, then repeated compiler
tests and consistency on the fix. The library reviewer also passed 39 rebuilt
release Python tests and 20 native games with 1,239 replayed boundaries. Neither
reviewer edited files, mutated Git or posted externally.

Author validation passed the full Rust/compiler/rebuilt Python suites, rustfmt,
Clippy with warnings denied and consistency on the fixed implementation. One
hundred supported Standard games completed naturally: 53,266 decisions, 319
payment requests (60 ward), 56 payments, 263 declines, 20 resolution mana
activations and 6,351 cloned/replayed boundaries. Maximum initialized state was
12,412 bytes, excluding allocator metadata, spare capacity, snapshots and static
definitions. Synthetic fixtures are distinguished from printed complete-card
admissions in ward-payments.md.

Coverage is 728/5,164 Standard identities, six regression cards and 36 token
factories. Nonmana/variable/granted ward, uncounterability, broader costs and
layers, remaining card semantics and native RL/search integration remain
incomplete. These reviews do not certify full Standard conformance or establish
an XMage performance comparison.
