# Independent review: conditional activations and exact mana planning

PR: https://github.com/altf4/rustmage/pull/81
Implementation reviewed: 46430a0db72f1e7041074b32935c7011b092cb66.

Two independent read-only subagents reviewed this milestone:

- Rules: all 11 full admissions, activation-start restrictions, mana classification,
  subtype/zone/changeling matching, source departure, graph guard semantics and
  conservative compiler boundaries.
- Library: prerequisite retention/departure, canonical keys, optimistic pruning,
  starting guards versus pending source costs, live/compact parity, snapshot
  ownership, efficiency and all 11 admissions.

Both verified the implementation HEAD and reported no actionable findings. Both
independently passed 518 Rust tests (510 unit plus eight integration), 92 compiler
tests and catalogue consistency. The library reviewer also passed 59 release
Python tests and 20 native games with 1,613 cloned/replayed boundaries. Neither
reviewer edited files, mutated Git or posted externally.

Author validation passed those suites, rustfmt and Clippy with warnings denied.
One hundred supported Standard games using all ten Verges and Mox Jasper completed
naturally: 49,525 decisions, 2,342 direct secondary Verge mana activations, 406
direct Mox activations and 7,720 cloned/replayed boundaries. Maximum initialized
state was 12,628 bytes, excluding allocator metadata, spare capacity, external
snapshots and immutable definitions. Native mana-action enumeration independently
checks compact reachability on small guarded boards. Definition-only nonmana and
pending-source fixtures are distinguished from printed admissions in
activation-guards.md.

Coverage is 889/5,164 Standard identities, six regression cards and 41 token
factories. Wider predicates/costs, remaining card semantics, general layers and
native RL/search integration remain incomplete. These reviews do not certify
full-format conformance or establish an XMage speed comparison.
