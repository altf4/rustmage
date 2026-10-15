# Independent review: nonbasic lands and pre-entry conditions

PR: https://github.com/altf4/rustmage/pull/35
Implementation reviewed: abbc8e8 against main.

Two independent read-only subagents reviewed the milestone:

- Rules: all 135 admissions, entry predicates and simultaneous timing, paired
  returns, intrinsic mana/type handling, Wastes and copy limits, ETBs/activations,
  funding and legality enforcement.
- Library: pre-entry conditions, simultaneous and linked-return batches,
  funding/source costs, snapshot replay, privacy and complete-text admission.

Both reported no actionable findings. Both passed 175 Rust tests, 33 script tests
and catalogue consistency. The library reviewer also rebuilt release, passed all
20 Python tests and completed 20 native games covering 384 nonbasic land plays.
Neither reviewer edited files or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 138,933 decisions, 1,979 nonbasic land plays (1,621 tapped),
28,875 mana choices and 812 library-choice decisions. Native tests cover entry
thresholds/ownership, Wastes, simultaneous arrivals/returns, stack triggers,
private library choices, exact colors/funding, self-sacrifice and replay. Python
covers tapped play, surveil privacy, untap, mana projections and clone/restore.

Coverage is 499/5,164 Standard identities, six regression cards and 29 token
factories. General replacement choices/ordering, restricted mana, fetch/search,
land animation, legendary lands, remaining cards/rules and full native RL/search
acceptance remain outstanding. These reviews do not certify full Standard
conformance.
