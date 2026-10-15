# Independent review: cast events, prowess and spell counts

PR: https://github.com/altf4/rustmage/pull/45
Initial implementation: d85428f. Final implementation: ecaa1b6.

Two independent read-only subagents reviewed this milestone:

- Rules: completed-cast timing across direct/prepaid/target/modal/payment paths,
  independent prowess instances, turn-local counts and resets, source damage/LKI,
  stun-aware untapping, every new admission/token factory and compiler gating.
- Library: event-hook coverage, premature/double-count prevention, source-index
  maintenance, snapshot/replay ownership, accounting, public schema 18 counts,
  trigger ordering/priority and native/Python integration.

The library reviewer identified one P2 compiler issue: generic bare `it deals`
could misattribute damage after a targeted clause. No frozen admission was found
affected. The fix gates both generic damage parsers, including the earlier
targeted-damage parser, to explicit source wording. Only proven own-name spells,
self-event initial pronouns and source-only activation initial pronouns normalize
to source references. Ambiguous other-event and later targeted-clause pronouns are
rejected. Regression cases cover each context and both ambiguous reproductions.

The fix also admits Iron-Fist Pulverizer and Shocking Sharpshooter through their
explicit source wording. Both reviewers examined those complete definitions and
the added targeted second-spell damage/LKI/private-scry and entry-trigger tests.
Both cleared ecaa1b6 with no remaining actionable findings. Neither edited files,
mutated Git or posted externally.

The rules reviewer independently passed the final 230 Rust tests, 41 compiler
tests and catalogue consistency. The library reviewer passed all initial 228 Rust,
40 script and 25 release Python tests and 20 native games (294 replayed casts,
38 second-spell events), then passed the final 41 script tests, catalogue
consistency and 14 focused native cast tests.

Author validation passed all final Rust/script/Python suites, catalogue
consistency, rustfmt and Clippy with warnings denied. One hundred supported
Standard games completed naturally with 47,789 decisions, 1,499 cloned/restored/
replayed completed casts, 215 second spells and 153 trigger-order choices.

Thirty-five complete definitions bring coverage to 582/5,164 Standard identities,
six regression cards and 34 token factories. The 4,582 unsupported identities,
wider costs/conditions/history, copying, granted prowess and remaining rules and
native RL/search integration remain outstanding. These reviews do not certify
full Standard conformance or establish an XMage performance comparison.
