# Independent review: casting and activation payment windows

PR: https://github.com/altf4/rustmage/pull/23
Implementation reviewed: 92c6096 against main, plus the query-local funding cache
follow-up recorded in this PR.

Two independent read-only subagents reviewed the implementation:

- Rules: payment timing, target binding, reserved source obligations, captured
  costs after source departure, pending mana ability recursion, exact funding
  reachability and deferred state-based actions and trigger placement.
- Library: compact resource keys and optimistic bounds, termination gates,
  equivalent-source canonicalization, actor ownership and privacy, atomic
  rejection, complete snapshot/replay and the read-only spell-cost cache.

Both reported no actionable findings. Both passed all 120 Rust tests, 27
compiler/importer tests and catalogue consistency. The library reviewer also
rebuilt release, passed all 14 Python tests and completed 20 native games with
970 payment windows. Both inspected the cache follow-up; the library reviewer
reran all nine funding tests, including the exhaustive 54-cost comparison.
Neither reviewer changed files or posted external comments.

Author validation passed rustfmt, Clippy with warnings denied, native and Python
regressions, and 100 supported Standard games completing naturally: 109,902
decisions, 6,509 activations and 5,226 payment windows. A freshly built limited
Freeform benchmark completed 1,000 games and 571,049 decisions; this is a smoke
workload, not full Standard acceptance or a Java speed comparison.

Coverage remains 212/5,164 Standard definitions. Broader costs, modes, triggered
or restricted mana, infinite-resource handling, layers, remaining card semantics
and full RL/search acceptance remain outstanding. These reviews validate this
milestone's documented scope; they do not certify the complete rules engine.
