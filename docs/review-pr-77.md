# Independent review: continuous keyword grants and exact mana planning

PR: https://github.com/altf4/rustmage/pull/77
Implementation reviewed: 093d376.

Two independent read-only subagents reviewed this milestone:

- Rules: continuous recipient scopes and conditions, layer ordering, tribal and
  changeling interactions, strike-step transitions, departing-source last-known
  information, funding behavior, all 12 complete admissions and compiler gates.
- Library: exact funding reachability and canonical keys, shared predicates,
  source departure and last-known information, source indexing, snapshot
  ownership and all 12 admissions.

Both reported no actionable findings. Both independently passed 498 Rust tests
(490 unit and eight integration), 88 compiler tests and catalogue consistency.
The library reviewer also passed 55 release Python tests and 20 native games,
including two haste-enabled sick mana activations and 1,166 replayed boundaries.
Neither reviewer edited files, mutated Git or posted externally.

Author validation passed those suites, rustfmt and Clippy with warnings denied.
One hundred supported Standard games completed naturally: 51,234 decisions,
77 observed entries with continuously granted haste, seven sick mana activations
and 6,774 cloned/replayed boundaries. Maximum initialized state was 12,708 bytes,
excluding allocator metadata, spare capacity, external snapshots and static
card definitions. Native mana-action enumeration independently checks the compact
funding model on small boards with multiple costs, source owners, active players
and source presence. See continuous-keywords.md for admission and test scope.

Coverage is 858/5,164 Standard identities, six regression cards and 41 token
factories. General layers/dependencies, remaining card semantics and native
RL/search integration remain incomplete. These reviews do not certify full
Standard conformance or establish an XMage speed comparison.
