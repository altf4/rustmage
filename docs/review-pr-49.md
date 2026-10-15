# Independent review: resolution-time sacrifices and conditional payments

PR: https://github.com/altf4/rustmage/pull/49
Implementation reviewed: 2519dc9 against main; correction reviewed: 125bf0c.

Two independent read-only subagents reviewed this milestone:

- Rules: sacrifice legality, APNAP selection and simultaneous movement, exact
  optional payments, decline, source LKI, linked returns, deferred state-based
  actions, all twelve admissions, compiler scope and filter gates.
- Library: action rejection and ownership, full versus partial payment,
  continuation ownership and accounting, public sacrifice/private discard
  projection, native/Python integration, rollback, compiler restrictions.

The library reviewer found that qualified unions such as “a nontoken artifact
or creature” and “an artifact or creature with flying” applied their qualifier
across both union arms. No admitted card used these ambiguous filters. Commit
125bf0c rejects these forms and adds regression cases. Both reviewers independently
reviewed the correction, passed all 47 compiler tests and catalogue consistency,
and reported no remaining actionable findings.

Both reviewers passed all 259 Rust tests (251 unit and eight integration). The
library reviewer also passed all 28 release Python tests and 20 native games with
111 sacrifices, 111 discard payments, 26 searches and 497 replayed choice
boundaries. Neither reviewer edited files, mutated Git or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and the full Rust/compiler/release Python suites. One hundred
supported Standard games completed naturally: 64,486 decisions, 473 sacrifices,
495 discard payments, 107 searches and 2,145 cloned/replayed choice boundaries.
Fixtures cover illegal and repeated selections, APNAP batch commitment, partial
cost decline, insufficient resources, private hand choices, returned source
incarnations, token departure, linked returns, source LKI, fizzled spells and
continuations into library search.

Coverage is 611/5,164 Standard identities, six regression cards and 35 token
factories. Broader casting costs, reflexive triggers, unsupported restrictions,
remaining card/rules semantics and full native RL/search integration remain
outstanding. These reviews do not certify full Standard conformance or establish
an XMage performance comparison.
