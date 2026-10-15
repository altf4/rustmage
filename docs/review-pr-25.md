# Independent review: private scry and surveil

PR: https://github.com/altf4/rustmage/pull/25
Reviewed implementation: dad1bfc against main, followed by fix be6020a.

Two independent read-only subagents reviewed the milestone:

- Rules: all 27 admissions, scry/surveil partition and ordering completeness,
  private observations, deferred movement/SBAs, source departure, instruction
  sequencing and rejection of illegal choices.
- Library: owned continuations, snapshot/restore, schema 12, complete action
  masks, invalid-action atomicity, ordering, state accounting, performance and
  compiler coverage.

The library reviewer found that selected cards were projected only as IDs;
policies using the current observation lost their known characteristics after
selection. Fix be6020a projects full ObjectViews in every destination group,
with native/Python regressions retaining the actor-only boundary. Both reviewers
rechecked the fix and reported no remaining actionable findings.

Both passed 129 Rust tests, 28 compiler/importer tests and catalogue consistency
on the base implementation. Both passed all nine native library tests after the
fix. The library reviewer also built release, passed all 15 Python tests after
the fix and completed 20 native games covering 787 library selections. Neither
reviewer edited files or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 80,614 decisions, 3,782 library selections, 470 bottomed and
1,421 buried. Native tests cover exhaustive two-card outcomes, both destination
orders, hidden partial observations, invalid actions, source departure, deferred
SBAs, short/empty/zero looks, subsequent draws and replay. The Python fixture
checks full selected-card views, privacy, cloning and partial-choice restore.

Coverage is 239/5,164 Standard definitions, with six legacy regression cards and
28 referenced token factories. Scry/surveil triggers, replacements, variable look
counts, wider rules, remaining card semantics and complete RL/search acceptance
remain outstanding. This review does not certify complete Standard conformance.
