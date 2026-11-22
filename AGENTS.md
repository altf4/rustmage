# Rustmage implementation

The goal is complete Standard gameplay in a fast deterministic in-process library,
with legal actions, hidden information and efficient owned branching/rollback.
Catalogue entries or partial card implementations do not satisfy it.

## Everyday bug fixes and features

- Search the existing tests for the behavior and the nearest shared fixture.
  If a case already exposes the bug, use it. If existing assertions miss it,
  extend a related assertion or table row, or add a small deterministic
  case that fails for the reported reason. For a feature, cover its new observable
  behavior. Add branch, privacy or rollback cases when those contracts change.
- Put ordinary Rust regressions in the owning rules or set suite. Use
  `bug_regressions` for cross-set bugs without a closer home. A new `#[test]`
  in an existing module shares its executable. If a new scenario file is needed,
  declare it once with `mod <file_stem>;` in the suite entry point. Reserve new
  Cargo test targets for distinct suites or independently run workloads.
- During a fix, select **both** the suite and case. Confirm the reproducer fails
  on the original code and passes with the fix. An existing failure report for
  the same case and inputs can supply the first result. Confirm the named test ran.
  This example selects one draw-replacement regression.

  ```sh
  python3 scripts/resource_guard.py -- cargo test --test bug_regressions bug_hunt_archaic_draw::archaic_discards_after_its_draw_replacement_choice -- --exact
  ```

  A case filter alone can still build every integration target. Keep the same
  worktree, Cargo target directory, test profile and features for both runs.
  Let the focused test command build what it needs. A separate preliminary
  `cargo build` or `cargo check` adds another pass. After the fix, drop the case
  filter to run the rest of the already linked suite. For shared-engine changes,
  run the affected suites together. Batch related edits before broad validation
  and let CI cover the full test profile on the PR.
  Use release locally when the change needs production behavior or speed
  measurements. For script-only or documentation changes, start with the
  corresponding script tests or documentation checks.

## Card implementation and repository checks

- Author new card behavior directly in `src/card_definitions/<set>/<slug>.rs`, with its
  paired provenance JSON. Read the complete XMage implementation and Oracle body.
  Reuse typed effects/targets/costs; add generic or bespoke Rust effects as useful.
  Do not require new English-parser productions or a translation framework.
- The English compiler is removed. Preserve working typed behavior. Mark a definition complete only after every
  printed obligation works. No LLM participates during gameplay.
- Use disjoint card files and isolated worktrees for parallel authors. Regenerate
  catalogue/module-link artifacts at integration. Avoid manual generated edits.
- Keep the Cargo boundaries: shared rules/state in `crates/engine/`, authored
  card files in `src/card_definitions/<set>/` compiled by generated per-set crates,
  and scenario modules grouped by set or rules family. Register each new scenario
  file once in its suite and keep Cargo test autodiscovery disabled. Consumers declare only
  `rustmage`; ship one native library. Never add card definitions or mutable
  catalogue/name/admission data to the engine's compilation.
- Extend the shared legal-action generator in `Game::visit_unfiltered_actions`.
  Keep its callback non-generic so listing, legality checks and indexed lookup
  share one compiled traversal. Preserve emission order and early exit.
- Before implementing a set, audit all its cards and identify every engine fix,
  shared mechanic and necessary engine improvement. Implement, test and review
  those changes first; then regenerate linkage, warm a stable guarded test cache,
  and implement cards against that engine. Return to the engine phase when a
  missed dependency appears, then revalidate and rewarm before dependent cards.
- Use GitHub issues and PRs. Require two independent nonauthor reviews of the final
  published batch, excluding reviewers' authored ancestors.
- Test actual rules interactions, legality/privacy, transactional failures and
  owned raw/index branch/restore. Prefer existing scenario helpers over new bots;
  do not require parser mutation suites for authored Rust cards.
- For local set acceptance, run `python3 scripts/link_catalogue.py` and
  `python3 scripts/validate.py --release --python smoke`. GitHub CI uses
  `python3 scripts/validate.py --test-artifacts --python smoke` so simulation,
  Standard gameplay and Python reuse the native test build.
  Keep execution inputs fixed until consuming jobs actually terminate. Poll the
  same running process after observation timeouts. Never delete active outputs.
- Clean disposable completed build/test files promptly. Preserve every required
  shared library at its exact path/hash and preserve useful uncommitted work.
- Report complete supported cards, shared mechanics, remaining blockers and
  measured implementation/simulation throughput. Keep the full goal open until
  entire-pool behavior, interactions and performance are verified.

- Reuse warm Cargo outputs; do not run clean builds during routine work.
- Resource safety: run Cargo through `python3 scripts/resource_guard.py -- cargo ...`.
  Routine validation enters this runner automatically. It enforces one build
  across worktrees and one Cargo job/test thread. Linux with a user systemd manager
  uses a shared hard 6 GiB memory cgroup (including orphaned units). macOS, Windows,
  and Linux without systemd use a supervised 6 GiB aggregate RSS threshold;
  this sampled budget can overshoot and is not a kernel allocation cap. Install
  `scripts/requirements-build.txt` for that backend. Reports identify the backend
  and limit semantics. Raw Cargo compilation from a checkout fails closed. Cargo invoked from an
  unrelated cwd with --manifest-path must still use the runner.
  Do not bypass the wrapper, raise the memory cap or launch unmanaged builds.
  Keep logs/reports under ~/.cache so reboot does not erase failure evidence.

- Do not deliberately trigger the OOM killer in routine tests. Even a tiny contained
  cgroup kill can cause disruptive desktop memory warnings. Verify configured
  limits and fail-closed enforcement without killing an allocation probe.
