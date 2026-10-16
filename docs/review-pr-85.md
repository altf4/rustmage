# Independent review: mandatory additional spell costs

PR: https://github.com/altf4/rustmage/pull/85
Implementation reviewed: c3258a4, then fix 9fc130ae0e19e9ef32f009d10eb048ef8201f74b.

Two independent read-only subagents reviewed this milestone:

- Rules: all three complete printed admissions, CR601 announcement ordering,
  bypass prevention, exact reservation eligibility, source generations, discard
  and death events, linked returns and paid entry continuations.
- Library: funding reachability and canonical resource roles, prefunded paths,
  retained hand resources, snapshot/clone ownership, observation privacy,
  compiler gates and public native/Python validation.

The library reviewer found a stale payment-request projection while a paid spell
waited for a returned permanent's entry choice. The fix suppresses requests for
paid announcements and requires the caster as viewer. Both viewers' projections
and an opponent-owned returned land now have regression coverage. Both reviewers
verified 9fc130a, independently passed all eight focused tests, and reported no
remaining actionable findings. Neither edited files, mutated Git or posted
externally.

Initial independent full validation passed 540 Rust tests, 96 compiler/data
tests and catalogue consistency; the library reviewer also passed 62 release
Python tests and 20 native Standard games (185 selections/payments, 3,639
cloned/replayed boundaries). Author final validation passed 541 Rust tests
(533 unit plus eight integration), the 96 compiler/data tests, catalogue
consistency, rustfmt and Clippy with warnings denied. Fresh final release
validation passed 62 Python tests and 100 naturally completed supported Standard
games: 112,513 decisions, 991 selections/payments across all three cards and
20,097 cloned/replayed boundaries. Maximum initialized state was 12,300 bytes,
excluding allocator metadata, spare capacity, external snapshots and immutable
definitions. Synthetic funding/link tests are identified in code and documentation;
they do not admit unsupported printed cards.

Coverage is 910/5,164 Standard identities, six regression definitions and 41 token
factories. Wider costs, remaining card semantics and native RL/search integration
remain incomplete. These reviews do not certify full-format conformance or an
XMage speed comparison.
