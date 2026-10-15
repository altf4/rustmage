# Independent review: player-damage triggers and captured amounts

PR: https://github.com/altf4/rustmage/pull/59
Implementation reviewed: d033b28; reviewed fix: 09b2fad.

Two independent read-only subagents reviewed this milestone:

- Rules: damage occurrence grouping, combat/opponent filters, counter/modified/
  tribal predicates, Equipment ownership, captured amounts and source LKI,
  whole-hand discard privacy/sequencing, all 22 admissions and compiler gates.
- Library: owned event/source facts, LKI and live-source lifecycle, validation,
  privacy, snapshots/accounting, continuations, compiler gates and all 22
  admissions.

The rules reviewer found no actionable issues on the original implementation.
The library reviewer found one P2 compiler binding error: a broad replacement of
“on it” in a source-damage event could bind a later target's antecedent to the
source. No frozen admission was affected. Commit 09b2fad restricts this binding
to an exact whole source-counter sentence and adds a target-then-counter negative
regression. Both reviewers rechecked the fix and reported no remaining findings.

Both independently passed 346 Rust tests (338 unit and eight integration), 61
compiler tests and consistency on the implementation; both passed compiler tests
and consistency again on the fix. The library reviewer also passed 37 rebuilt
release Python tests and 20 native games on the fixed head, with 201 replayed
boundaries. Neither reviewer edited files, mutated Git or posted externally.

Author validation passed the full Rust/compiler/rebuilt Python suites, rustfmt,
Clippy with warnings denied and consistency on the fixed implementation. One
hundred supported Standard games completed naturally: 58,554 decisions, 773
observed trigger announcements, 71 aggregate announcements, 266 Equipment
announcements, 233 event-bound discard choices and 1,006 cloned/replayed
boundaries. Maximum initialized state was 12,564 bytes for this workload,
excluding allocator metadata, spare capacity, snapshots and static definitions.
Synthetic source-damage IR is distinguished from complete-card admissions in
damage-triggers.md.

Coverage is 712/5,164 Standard identities, six regression cards and 36 token
factories. Wider event bindings, granted/temporary triggers, replacement and
prevention, control/type layers, remaining card semantics and native RL/search
integration remain incomplete. These reviews do not certify full Standard
conformance or establish an XMage performance comparison.
