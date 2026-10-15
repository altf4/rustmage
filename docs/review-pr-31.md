# Independent review: menace and blocker declaration completion

PR: https://github.com/altf4/rustmage/pull/31
Implementation reviewed: 979785a against main.

Two independent read-only subagents reviewed the milestone:

- Rules: all 11 admissions, cumulative evasion/count restrictions, exact
  completion matching, valid partial declarations, FinishBlockers enforcement,
  post-declaration behavior, and targeted versus mass restriction semantics.
- Library: shared scarce blockers, minimum/maximum constraints, complete
  extensions, independent oracle quality, snapshots/privacy, atomic rejection,
  compiler admission and native/Python integration.

Both reported no actionable findings. Both passed 156 Rust tests, 31 script
compiler/importer tests and catalogue consistency. The library reviewer also
rebuilt release, passed all 18 Python tests and completed 20 native games.
Neither reviewer edited files or posted externally.

The library reviewer confirmed the exhaustive oracle is meaningfully independent:
it manually validates complete assignments without production helpers and compares
all reachable partial masks with legal full extensions on six boards. The cases
cover shared flying/reach blockers, optional groups, minimum-three requirements,
contradictory min/max restrictions, token exclusion and unblockable attackers.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 48,861 decisions, 142 activations, 21,937 decisions with
minimum-blocker restrictions and 270 selected blockers. Other regressions cover
compiled restrictions/token creation, post-declaration evasion, atomic premature
finish rejection, private partial observations and native/Python restore.

Coverage is 346/5,164 Standard identities, six regression cards and 29 token
factories. Attack/block requirements, costs, multiple attackers per blocker,
broader restrictions, remaining card/rules semantics and full native RL/search
acceptance remain outstanding. These reviews do not certify full Standard
conformance or general blocker requirement optimization.
