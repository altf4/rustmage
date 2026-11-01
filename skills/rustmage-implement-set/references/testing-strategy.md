# Test planning for fast complete-card implementation

Use the active checkout's `docs/testing.md`, `docs/direct-card-authoring.md` and
runner interfaces as the command authority. This reference defines test selection
and cadence; it does not waive repository acceptance checks.

## Plan assertions before implementation

In the batch's existing tracking issue, PR or work notes, make a compact mapping:

| Card or shared rule | Printed obligation / risk | Expected observable outcome | Test case or applicable shared proof |
| --- | --- | --- | --- |
| A targeted spell | A target leaves and returns before resolution | The old target is illegal; the new incarnation is unaffected | Focused response scenario |
| Several counter triggers | Each trigger's event, amount and recipient | The actual event produces the independently expected counters | Small table with a row for each card |
| A new private choice | Opponent visibility and owned pending state | Opponent sees only public facts; branches resolve independently | One affected engine-contract root plus card scenario |

Derive expected results from the full Oracle/XMage evidence and resolved rules,
not from the new Rust definition, generated catalogue or current trace output.
Record uncovered obligations as blockers. Shared keyword tests can prove the
keyword's engine semantics; each changed card must still prove that it has the
right keyword and that its distinct printed effects work. A row in a loading or
cast/entry regression table alone does not establish complete support.

Assign each shared test family one owner. Workers add disjoint scenarios or rows;
the integrator combines them without duplicating the same contract suite per
card. Keep the mapping lightweight and reuse existing evidence formats.

## Write small tests that distinguish correct rules

- Arrange minimal decks, boards and zones with existing scenario helpers, then
  exercise real paid legal actions. Direct state setup is a fixture convenience;
  it must not replace the action or rules event under test. Prefer the small
  helpers in `tests/support/mod.rs`; avoid copying the legacy
  `big_helpers::checked` pattern that branches, serializes and replays every action.
- Assert the actual result: damage/life totals, exact zone and incarnation, token
  characteristics, counter amounts, cost payment, timing, trigger recipients or
  visibility. Include enough context to identify a failing table row.
- For a new rule, pair the success case with a materially different interaction
  or rejected action. Choose boundaries from the printed rule: zero/one/multiple
  events, optional decline, partial/all target invalidation, stale targets,
  unaffordable costs, source departure, controller changes or delayed cleanup.
  Cover each distinct mode and obligation; avoid a Cartesian product of all cases.
- Use both seats when player-relative behavior matters. Check both observations
  for new private information. Test illegal-action rollback at the affected
  failure boundary, including failures after partial work when relevant.
- For new owned pending state, queues, randomness or rollback behavior, extend an
  appropriate representative raw/index/restore root. Cover independent children,
  parent disposal and restored behavior there. Unchanged shared machinery does
  not require repeating those operations at every card action.
- Table together cards that share a real rules contract; keep distinct interactions
  independently understandable. Do not add a per-card bot, random seed hunt,
  exhaustive branch traversal, parser mutation suite or long self-play run to
  demonstrate an outcome a bounded scenario can reach directly.

Use fixed seeds, small fixtures and explicit action/loop ceilings. Prefer
millisecond scenarios. The current native runner caps each scenario at 45 seconds
and suite execution at 240 seconds, separately from compilation; confirm current
limits before relying on them. Split or repair slow scenarios rather than raising
budgets or dropping obligations. Set-level complete games remain a separate,
bounded acceptance workload.

## Use cached focused runs while authoring

The engine, per-set definitions and integration tests compile separately. Complete
the skill's full-set engine audit, fixes and reviews before card authoring. Warm
one guarded target with the engine's final test profile/features, then keep that
baseline stable while writing cards. Avoid cycling profiles, feature sets or
compiler flags. Normal card and test edits should reuse the engine; investigate
unexpected engine recompilation rather than accepting it as routine.

A test filter selects execution within a compiled integration target; it does not
compile just one function. Put new set cases in that set's own integration target,
reuse `tests/support/mod.rs`, and select it with `--test <target>`. Keep the original
suite available as the `legacy` integration target. Do not put card scenarios back
into engine unit tests or depend on the complete legacy harness from a new set
harness. Test helpers use the internal, dev-only fixture feature; production users
continue to declare only the `rustmage` dependency.

Focused `cargo test` does not inherit the native runner's per-test timeouts. Use
the existing process-tree-safe `run_command` helper inside the resource guard for
a wall-clock bound. This example caps compilation plus focused execution together
at 390 seconds; the separate 45-second scenario cap applies only to the native
runner. Explicit action ceilings still belong in the scenarios themselves.

For example, run this existing exact scenario from the active checkout; substitute
the actual test path and a stable cache name unique to the authoring worktree:

```sh
rustmage_author_cache="$HOME/.cache/rustmage-authoring/your-worktree"
mkdir -p "$rustmage_author_cache"
rustmage_run_dir="$(mktemp -d "$rustmage_author_cache/run-XXXXXX")"
rustmage_test_filter='engine::assurance::interactions::ajanis_pridemate_triggers_from_actual_lifelink_damage'
CARGO_TARGET_DIR="$rustmage_author_cache/target" \
  python3 scripts/resource_guard.py -- python3 - \
  "$rustmage_test_filter" "$rustmage_run_dir/focused.log" <<'PY'
import json
import os
from pathlib import Path
import sys

sys.path.insert(0, 'scripts')
from run_native_tests import run_command

command = ['cargo', 'test', '--test', 'legacy', sys.argv[1], '--',
           '--exact', '--include-ignored', '--test-threads=1']
result = run_command(command, env=dict(os.environ), log=Path(sys.argv[2]), timeout=390)
print(json.dumps(result))
raise SystemExit(result['exit'])
PY
```

For a coherent family, use its actual module prefix as the filter and omit
`--exact`. Select changed scenarios plus existing regressions for the affected
rules and their consumers. Verify the intended tests actually executed, with a
positive passing count and no failed or ignored intended cases; Cargo can return
success with zero matching tests. Use the guarded test command with `-- --list`
when discovering names, not as evidence that behavior passed. Keep each run's
logs in a new directory, and inspect failures before broadening.

Use `scripts/run_native_tests.py` for the whole native suite at a shared-rule
checkpoint when
the affected set is broad or uncertain, retaining its bounded execution and timing
report while reusing the same development target:

```sh
python3 scripts/run_native_tests.py \
  --target-dir "$rustmage_author_cache/target" \
  --report "$rustmage_run_dir/report/native.json"
```

The focused example runs Cargo inside the guarded process; the native runner
enters the resource guard automatically. Honor current guard
serialization even across worktrees; parallel authorship is useful, parallel
compilers are not permitted. Keep sources, generated links and fixtures immutable
for every consuming job, and poll the same process through terminal completion.

## Keep acceptance builds cached too

The validator reuses `CARGO_TARGET_DIR` when set, otherwise a stable per-worktree
cache. Use `--target-dir` to select the authoring cache explicitly:

```sh
python3 scripts/link_catalogue.py
python3 scripts/validate.py --release --python smoke \
  --target-dir "$rustmage_author_cache/target"
```

Keep debug/test/release outputs in their normal Cargo profile subdirectories.
Acceptance may need its first release build, but later runs should reuse
compatible artifacts. `--fresh` intentionally creates and cleans a disposable
target; reserve it for an explicit clean-build requirement or investigation.
Targets and `XDG_CACHE_HOME` must be outside the source checkout. The runner rejects
legacy validation targets with `report/result.json` because their original shared
library paths remain pinned. Choose a new mutable cache instead.

Inspect older checkouts before assuming this interface exists. A runner that
creates fresh targets or deletes compiler outputs needs that tooling fixed first;
setting an environment variable cannot override a runner that replaces it.

Keep evidence independent of the cache: every validation run needs a unique
report/log directory, fresh source/input verification and fresh check results.
Retain resource-guard serialization, all required checks, bounded execution,
process-tree termination and exact shared-library path/hash preservation. A cached
artifact is not a passing report. Cache reuse relies on Cargo's normal dependency
and input invalidation, never forcing stale binaries or suppressing changed tests.
Do not let cleanup erase a target still used by authoring or acceptance.

Separate mutable compiler outputs from required library artifacts. Copy newly
published shared libraries into immutable per-run paths, record those hashes,
and run Python/FFI consumers against that snapshot. Later Cargo builds may replace
their mutable outputs without invalidating published evidence. Preserve every
previously required library at its original exact path/hash; do not overwrite a
target that an existing consumer or report still requires to remain immutable.

When changing this runner, use its infrastructure tests for cache reuse, distinct
reports, source drift, terminal cleanup and preservation of published libraries.
Use bounded mocked commands or a tiny isolated fixture where appropriate. Do not
run the entire card suite multiple times just to test cache plumbing, or trigger
OOM probes. Keep production check commands unchanged apart from target/report
plumbing.

## Run broad checks at meaningful checkpoints

Treat small worker chunks as authoring units within a reviewable integration PR.
Group by shared mechanic and dependency risk; do not replace a five-card full-suite
rule with another fixed card-count rule. Keep a missing or failed obligation gated
while proceeding with independent work. A shared core dependency still needs its
own integrated acceptance and two independent nonauthor reviews before expanding
the dependent batch.

The integrator runs `python3 scripts/link_catalogue.py` and the required
`python3 scripts/validate.py --release` with the runner's verified reusable-target
option on the assembled, frozen candidate. Include Python/FFI checks when required
by the checkout or affected interface, and retain required benchmarks. Do not turn off
checks, use metadata-only provenance, or substitute a focused pass for merge
acceptance. Validate the final executable inputs after review fixes; read-only
reviews do not each need a duplicate build. Reuse prior acceptance only when its
recorded inputs match; new executable inputs need fresh acceptance.

Full validation still includes source pins, format checks, infrastructure
contracts, all-target Clippy, native assurance and, with `--release`, the production
build and smoke benchmark. Cache those builds without removing any checks. Avoid
invoking full validation per card, five-card worker chunk, handoff or unchanged
review head.

Record the source revision/changed inputs, selected tests and executed counts,
command/report paths, and elapsed time. Separate resource-guard queue time,
compile time, scenario execution and benchmarks. Use native compile logs and
`native.json`/`native.jsonl`, plus the validator's phase reports, to locate cost.
A slow compile needs a build/cadence diagnosis; a stalled scenario needs its own
bounded reproduction. Do not automatically start another full suite, raise
timeouts or restart a job merely because tool observation timed out.

Keep reusable author targets while the worktree is active. Once jobs terminate
and the worktree is retired, clean disposable outputs while preserving reports,
useful uncommitted work and every required shared library at its exact path/hash.
Report complete cards per elapsed implementation interval, full/focused run
counts and measured validation time. Do not claim a speedup without measurements.
Set-only complete games, all printed obligations, deterministic replay and owned
search checks remain required for full-set acceptance; small smoke workloads do
not certify that milestone or all-Standard gameplay.
