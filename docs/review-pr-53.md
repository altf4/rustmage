# Independent review: Equipment attachments

PR: https://github.com/altf4/rustmage/pull/53
Implementation reviewed: 5e05210 against main.

Two independent read-only subagents reviewed this milestone:

- Rules: exact-incarnation links, equip timing/payment and legal targets,
  independent entry grants, continuous keyword consumers, funding behavior,
  frozen state-based detachments, simultaneous departure LKI, all 22 admissions
  and complete-text compiler gating.
- Library: attachment ownership and source-index lifecycle, effective keyword
  queries, funding ownership/identity edges and memoization, frozen LKI batching,
  snapshot/accounting, schema 22 privacy, native/Python integration and all 22
  admissions.

Both reported no actionable findings. Both independently passed all 287 Rust
tests (279 unit and eight integration), 53 compiler tests and catalogue
consistency. The library reviewer also passed all 32 release Python tests and
20 native games with 172 attachment changes and 458 replayed boundaries.
Neither reviewer edited files, mutated Git or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and the complete Rust/compiler/release Python suites. One hundred
supported Standard games completed naturally: 54,281 decisions, 1,081 attachment
changes, 1,769 paid Equipment activations and 3,063 cloned/replayed boundaries.
Maximum initialized state for that workload was 11,860 bytes, excluding allocator
metadata, spare capacity, snapshots and static definitions. Fixtures cover illegal
equip timing/targets, paid and entry attachment, source/target returns, costs and
fizzling, cleanup, combat keywords, keyword-filtered lords, native/funding haste,
resource edge identities, simultaneous LKI and frozen detachment/legend choices.

Coverage is 656/5,164 Standard identities, six regression cards and 35 token
factories. Auras, granted non-keyword abilities, equipped-creature event subjects,
wider costs and layers, residual card semantics and full native RL/search
integration remain outstanding. These reviews do not certify full Standard
conformance or establish an XMage performance comparison.
