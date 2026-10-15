# Independent review: shared targets and resumable announcements

PR: https://github.com/altf4/rustmage/pull/15
Implementation reviewed: `95ee8b8` against `main`.

Two independent read-only subagents reviewed the implementation:

- Rules: all 26 new admissions against pinned Oracle text, Lesson casting, shared
  predicates, APNAP announcements, required targets for optional effects, target
  counts/payment timing, incarnation binding, partial/all-invalid resolution,
  source departure and reanimation entry triggers. No actionable findings.
- Library: announcement ownership, deferred payment, bound target buffers,
  clone/restore/revisions, schema 6 privacy, nested state accounting, compiler
  exactness and C/Python integration. No confirmed actionable findings.

Both ran 78 Rust tests, 18 compiler/importer tests, and catalogue consistency.
The library reviewer also rebuilt release, ran ten Python tests, and completed
20 native targeting games (92 target choices, 16 multi-target casts). Reviewers
made no edits or GitHub posts.

Author validation additionally passed rustfmt and Clippy with warnings denied.
100 native supported Standard games completed naturally: 52,735 decisions,
545 target choices and 93 multi-target casts. Python tests clone/restore both
spell and trigger announcements; small-board enumeration verifies all complete
one-or-two-creature target sets without duplicates/dead ends. Tests also cover
zero targets versus all selected targets invalid, mandatory targets with “may,”
APNAP resume, private choices, atomic rejection and reanimation identity/triggers.

A single limited Freeform benchmark sanity run at this implementation measured
1.56 million primitive decisions/second including smoke-policy observations,
4,000 initialized state bytes, 162 ns cloning and 136 ns restore. This is not a
full Standard workload, repeated performance study, or XMage comparison.

Reviewed executable coverage: 106/5,164 Standard identities. Multiple distinct
target groups, broader costs/modes/restrictions, counters/tokens, replacements/
layers and other remaining card semantics stay gated. Full acceptance is unproven.
