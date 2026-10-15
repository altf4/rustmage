# Independent review: distinct target groups and creature damage

PR: https://github.com/altf4/rustmage/pull/41
Implementation reviewed: a2d22b3 against main.

Two independent read-only subagents reviewed the milestone:

- Rules: all 14 admissions, independent groups, legal completion/distinctness,
  per-group revalidation/fizzling, instruction focus/rebasing, fight/power-damage
  semantics, owned continuations and compiler gating.
- Library: group ownership, announcement/payment transitions, independent
  revalidation, continuation focus, privacy, compiler operand rebasing and ABI
  integration.

Both reported no actionable findings. Both passed 208 Rust tests, 36 script tests
and catalogue consistency. The library reviewer also passed all 23 release Python
tests and completed 20 native games with 182 target choices. Neither reviewer
edited files or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 51,105 decisions, 918 target choices (515 first/403 second),
168 mode choices and 1,147 decisions with distinct groups on the stack. Native
fixtures include a manual ownership/hexproof mask oracle, declaration rejection
and replay, optional groups, within/across-group distinctness, partial and whole
fizzling, current derived power, simultaneous lethal damage, lifelink/deathtouch/
strike behavior, nonpositive power, self-fight and private-continuation focus.
Python verifies two group declarations and power damage through the C ABI.

Coverage is 537/5,164 Standard identities, six regression cards and 31 token
factories. Cross-group relationships, planeswalker participants, wider source/LKI
instructions, remaining card/rules semantics and full native RL/search acceptance
remain outstanding. These reviews do not certify full Standard conformance.
