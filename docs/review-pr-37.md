# Independent review: qualified library search and fetching

PR: https://github.com/altf4/rustmage/pull/37
Implementation reviewed: 2bf1784 against main.

Two independent read-only subagents reviewed the milestone:

- Rules: all 13 admissions, qualified fail-to-find, sacrifice/Lander lifecycle,
  simultaneous arrivals, shuffle/continuation timing, reveal knowledge, privacy
  and illegal-choice rejection.
- Library: owned selections, shuffle/RNG replay, schema 14 privacy and
  revealed-hand lifetimes, source/token generations, legality, funding and
  compiler admission.

Both reported no actionable findings. Both passed 185 Rust tests, 34 script tests
and catalogue consistency. The library reviewer also rebuilt release, passed all
21 Python tests and completed 20 native games covering 354 searches.
Neither reviewer edited files or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 69,261 decisions, 1,702 searches, 1,743 selections, 18,109
decisions with Lander tokens and 16,625 with revealed hand knowledge. Native
fixtures cover rejection, private partial selection, qualified fail-to-find,
empty libraries, exact shuffle and future RNG, simultaneous arrivals, entry
conditions, Wastes/typed nonbasics, reveal records/fresh incarnations, token slot
reuse, continuation order and trigger deferral. Python verifies both destinations,
private selections, public results and clone/restore through the C ABI.

Coverage is 512/5,164 Standard identities, six regression cards and 30 token
factories. Wider searches/qualities, mixed destinations, mandatory unqualified
searches, hidden-information history/belief sampling, remaining cards/rules and
full native RL/search acceptance remain outstanding. These reviews do not certify
full Standard conformance or a complete information-set search implementation.
