# Independent review: object incarnations and Standard removal

PR: https://github.com/altf4/rustmage/pull/12
Base: `engine/damage-events`. Initial implementation: `d762630`.
Review fixes: `837a0c4`.

Two independent read-only subagents reviewed the PR:

- Rules reviewer: generation-bound targets, centralized zone transitions, resets,
  library indexing, combat membership, source identity and four exact removal
  definitions against the September 25, 2026 Comprehensive Rules.
- Library reviewer: snapshot/restore, schema 4/privacy, C/Python integration,
  catalogue admission/fingerprint, state accounting and performance concerns.

The initial rules review found two issues. Removing the sole declared attacker
lost the history needed to decide whether remaining combat steps could be skipped
(CR508.8). Removing an attacker also lost surviving blockers' independent combat
membership, potentially suppressing the first-strike step (CR506.4/510.4).

Both were fixed with declaration history and a separate blocking-creature list,
including snapshot/observation/accounting changes. Regression tests include actual
Unsummon gameplay using only the public API, first-strike orphan blockers and
restore/replay. Both reviewers re-reviewed `837a0c4` and reported no remaining
actionable findings. Reviewers made no repository edits or GitHub posts.

Final verification: 50 Rust tests, 12 importer/compiler tests, eight Python tests
against the rebuilt release library, generated catalogue check, rustfmt, Clippy
with warnings denied, and release build. Native supported-Standard self-play also
completed 100 games and 49,750 decisions. Coverage is 47/5,164 Standard identities;
this review does not establish full Standard support or trigger-engine completion.
