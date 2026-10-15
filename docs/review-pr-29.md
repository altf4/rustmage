# Independent review: effective temporary keywords

PR: https://github.com/altf4/rustmage/pull/29
Implementation reviewed: 83f4961 against main.

Two independent read-only subagents reviewed the milestone:

- Rules: all 56 admissions, additive temporary gains, ordinary hexproof and
  indestructible, targeting/revalidation, combat interactions, actual source LKI
  after late gains, cleanup, layer-7 filters, funding keys and replay.
- Library: effective keyword consumers, resets/snapshots, 32-bit flags, compact
  funding canonicalization and haste, invalid-target rejection, LKI across owned
  states, compiler admission and native/Python integration.

Both reported no actionable findings. Both passed 151 Rust tests, 30 script
compiler/importer tests and catalogue consistency. The library reviewer also
rebuilt release, passed all 17 Python tests and completed 20 native games covering
1,651 decisions with hexproof/indestructible. Neither reviewer edited files or
posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 55,454 decisions, 486 activations and 7,299 decisions with
hexproof/indestructible. Native fixtures cover target rejection/revalidation,
destruction versus other deaths, cleanup, haste payments, keyword-sensitive
filters, locked mass sets, source reset, vigilance, both strike steps,
lifelink/deathtouch and late-granted keyword LKI/replay. Python verifies effective
keyword/stat projections through cloning and restore.

Coverage is 335/5,164 Standard identities, six regression cards and 28 token
factories. Static grants, keyword counters, removal/prohibition, dependencies,
protection, qualified hexproof, menace, remaining card/rules semantics and full
native RL/search acceptance remain outstanding. This review does not certify
complete Standard conformance or the full layer system.
