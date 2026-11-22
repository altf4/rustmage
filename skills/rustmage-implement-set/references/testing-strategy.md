# Test planning for fast complete-card implementation

Use the active checkout's `AGENTS.md` and runner interfaces for validation commands.
Follow the repository's acceptance checks and use this reference to select tests.

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
  helpers in `tests/support/mod.rs` and exercise branching or replay at the
  decision points relevant to the assertion.
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

### Register scenarios once

Find the set or rules-family suite in the root `Cargo.toml`. For example,
`dft_rules_scenarios` links `tests/suite_dft_rules_scenarios.rs`. To add a new
scenario file there, put it beside the suite entry point and declare it with a
plain `mod <file_stem>;` line. Directory-style suites such as
`tests/trk_rules_a_e/main.rs` use the same sibling convention. The suite
declares shared fixtures once. Scenario files can import them from `crate::support`.

Keep existing scenario source paths and names used by provenance and focused
test selectors. Leave `autotests = false` in place. When a new set needs its
first suite, register one `[[test]]` with `name = "<set>_scenarios"` and
`path = "tests/suite_<set>_scenarios.rs"`, then add ordinary cases as modules.
Preserve independently run workloads such as complete games. The infrastructure
test `test_every_scenario_file_has_one_cargo_target` detects missing and duplicate
registrations. All modules in a suite share one test executable and link step.

### Select the cases to run

The native runner's `--test-target` accepts a scenario module name or glob, or
an exact suite name. A case filter narrows execution within the compiled suite.
For example, the command below builds the DFT rules suite and runs one crew case.

The native runner enters the resource guard automatically. Use the same target
as the validator and keep the report for each run in a separate directory.

```sh
rustmage_target="$(python3 -c 'from scripts.build_platform import cargo_environment; print(cargo_environment()["CARGO_TARGET_DIR"])')"
rustmage_reports="${XDG_CACHE_HOME:-$HOME/.cache}/rustmage-authoring"
mkdir -p "$rustmage_reports"
rustmage_run_dir="$(mktemp -d "$rustmage_reports/run-XXXXXX")"
python3 scripts/run_native_tests.py \
  --target-dir "$rustmage_target" \
  --report "$rustmage_run_dir/native.json" \
  --test-target dft_crew \
  --filter crew_choice_raw_index_and_restore_branches_keep_owned_pending_and_events
```

For a coherent family, filter by a shared test-name prefix. Verify the intended
cases executed and passed. Keep logs in a new directory for each run. Omit the
target and filter for a shared-rule checkpoint with broad impact, using the same
Cargo target. Keep inputs fixed until the consuming process terminates.

## Choose validation artifacts by workload

**GitHub CI.** Run `python3 scripts/validate.py --test-artifacts --python smoke`.
Native assurance builds the benchmark and shared library in the test profile.
The validator checks both against that run's Cargo artifact records, runs the
native simulation and Standard smoke with the benchmark, and loads the
snapshotted library for Python. Keep all three checks on these artifacts.
Omitting `--test-artifacts` from the Python smoke command requests a release
library and compiles the card pool a second time.

**Local completed-set acceptance.** Use the production profile, including its
ThinLTO setting. Build only the release library and benchmark needed for this
workload. The validator reuses `CARGO_TARGET_DIR` when set, otherwise a stable
per-worktree cache. To use the authoring target explicitly, run

```sh
python3 scripts/link_catalogue.py
python3 scripts/validate.py --release --python smoke \
  --target-dir "$rustmage_target"
```

Keep debug/test/release outputs in their normal Cargo profile subdirectories.
Acceptance may need its first release build, but later runs should reuse
compatible artifacts. `--fresh` intentionally creates and cleans a disposable
target; reserve it for an explicit clean-build requirement or investigation.
Targets and `XDG_CACHE_HOME` must be outside the source checkout. The runner rejects
legacy validation targets with `report/result.json` because their original shared
library paths remain pinned. Choose a new mutable cache instead.

On older branches, check the suite targets in `Cargo.toml` and the flags in
`scripts/validate.py` before using these examples. A runner that
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

The integrator runs the local completed-set acceptance commands above on the
assembled, frozen candidate. Retain the Python/FFI checks and benchmarks. Do not turn off
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

For a proposed build-time optimization, compare before and after on the same
host, toolchain and guard settings. Give each revision its own empty target
outside the checkout. Time the same full workload, record the Cargo profile and
targets, and report compilation and linking separately from queue and test time.
For changes to action generation, also run each revision's optimized benchmark
with the same decks, seeds and game count. Include Freeform and a representative
Standard deck workload, compare decision counts and gameplay throughput over
repeated runs, and verify raw/index/restore behavior. Keep these cold trials
separate from routine card authoring.

Keep reusable author targets while the worktree is active. Once jobs terminate
and the worktree is retired, clean disposable outputs while preserving reports,
useful uncommitted work and every required shared library at its exact path/hash.
Report complete cards per elapsed implementation interval, full/focused run
counts and measured validation time. Do not claim a speedup without measurements.
Set-only complete games, all printed obligations, deterministic replay and owned
search checks remain required for full-set acceptance; small smoke workloads do
not certify that milestone or all-Standard gameplay.
