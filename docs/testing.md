# Card coverage

Tests cover printed card features, rather than lines of Rust or an ever-growing
collection of interactions. Ordinary CI should make implementing another card
cheap: small deterministic fixtures, one legal execution, concrete outcomes.

- `src/testing/mechanics.rs` checks shared mechanics using representative cards.
  Flying, first strike, haste, legendary, targeting, combat, token rules and common
  costs belong here. A card with flying does not need its own flying rules test.
- `src/testing/cards.rs` and `src/testing/card_cases/` check card-specific behavior.
  Each distinct ability, spell mode and alternate casting cost needs a normal
  success case. Equivalent cards can share a table with literal expectations.
- `src/testing/core.rs` checks hidden information, transactional failures,
  independent owned branches, raw/index actions and restore once. A bounded full
  game is appropriate here. Python checks cover the binding contract, rather
  than replaying card scenarios through a second language.

For Smaug, Wicked Worm, test that entry creates the right number of tapped
Treasures and that a Treasure-funded cast draws a card and loses 1 life. Flying
and legendary are covered by the shared mechanics suite. A mode or conditional
ability may need a small contrasting case to establish its ordinary behavior;
this does not justify an interaction matrix across other cards, both players,
source departures, copy effects, replacements and restore at every action.

Use printed Oracle outcomes as expectations. A successful cast, catalogue entry,
list of legal actions or recorded observation digest alone does not establish
ability coverage. Test through checked actions; setup may directly arrange a
small state. Card helpers must not silently add cloning, restore, invalid-action
or privacy checks. All loops need small explicit bounds and fixed inputs.

Deep corner cases, fuzzing, large randomized games, exhaustive branching and bug
hunts are useful during implementation or a dedicated investigation. Keep those
experiments outside the repository and regular CI. If an investigation uncovers
missing basic functionality, add its smallest card or shared-mechanic example.
Do not retain a second legacy or ignored exploratory suite.

## Local iteration

```sh
python3 scripts/link_catalogue.py
python3 scripts/validate.py --python smoke
```

The validator reuses compiler outputs and retains separate reports under
`~/.cache/rustmage-validation-*/report/`. Native scenarios run serially in a single
process with a 30-second execution cap, independent of compilation. The report
separates build and execution times, accounts for every selected test, and rejects
zero or skipped suites. `native-execution.log` names the running case and retains
failure details. The test profile prioritizes compilation instead of optimization.

Run a focused group with the same reusable target:

```sh
python3 scripts/run_native_tests.py \
  --target-dir "$HOME/.cache/rustmage-tests/target" \
  --report "$HOME/.cache/rustmage-tests/smaug/native.json" \
  --filter smaug
```

Omit `--filter` for the entire native suite. CI always runs the entire suite.
Python checks use a debug library during normal validation. Production acceptance
also builds the optimized library and measures a bounded simulation sample:

```sh
python3 scripts/validate.py --release --python smoke \
  --target-dir "$HOME/.cache/rustmage-tests/target"
```

Every Cargo command must run through `scripts/resource_guard.py`; validation and
the native runner enter it automatically. It enforces one build, one Cargo job,
one test thread and the shared 6 GiB memory cap. Tool contracts verify configured
limits and fail-closed behavior without provoking an OOM kill.

Targets and `XDG_CACHE_HOME` must be outside the checkout. Set `CARGO_TARGET_DIR`
or use `--target-dir` to select a reusable target. Each run copies any consumed
shared library into its report and records the immutable path and hash; later
builds can reuse their target without modifying published copies. Old targets
containing `report/result.json` remain pinned and are rejected for reuse.

Use `--fresh` only for an intentional clean-build measurement. It removes its
owned target only after all consuming processes terminate, preserving reports
and published libraries. Never edit build inputs or delete active outputs while
a consuming command is running. Poll the same process after an observation
timeout. Source-pin checks remain mandatory; `check_sources.py --metadata-only`
is a local convenience and does not replace acceptance or CI source verification.
