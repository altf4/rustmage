# Rustmage implementation

The goal is complete Standard gameplay in a fast deterministic in-process library,
with legal actions, hidden information and efficient owned branching/rollback.
Catalogue entries or partial card implementations do not satisfy it.

- Author new card behavior directly in `src/card_definitions/<set>/<slug>.rs`, with its
  paired provenance JSON. Read the complete XMage implementation and Oracle body.
  Reuse typed effects/targets/costs; add generic or bespoke Rust effects as useful.
  Do not require new English-parser productions or a translation framework.
- The English compiler is removed. Preserve working typed behavior. Mark a definition complete only after every
  printed obligation works. No LLM participates during gameplay.
- Use disjoint card files and isolated worktrees for parallel authors. Regenerate
  catalogue/module-link artifacts at integration. Avoid manual generated edits.
- Use GitHub issues and PRs. Require two independent nonauthor reviews of the final
  published batch, excluding reviewers' authored ancestors.
- Test card coverage, not code coverage. Keep one focused suite for shared rules
  and mechanics (flying, first strike, haste, legendary, common costs, targeting,
  tokens, etc.). Do not re-test a shared mechanic for each card that uses it.
- Give each implemented card a small, deterministic sanity case for each distinct
  ability, spell mode, and alternate casting cost. Assert the printed outcome
  using ordinary legal actions. Reuse fixtures and table cases for equivalent
  abilities; metadata, action enumeration, and recorded fingerprints alone do not
  establish card coverage. Smaug, Wicked Worm needs its Treasure entry ability and
  Treasure-funded cast trigger checked; flying and legendary belong in mechanics.
- Maintain `src/testing/coverage/<set>.json` for every admitted physical card.
  Map every printed ability, mode and casting route to a passing outcome case or
  the shared-mechanics registry. Pin the source facts and paragraphs. Table rows
  must report their feature/row ID after their assertions pass; the enclosing
  test name alone cannot establish coverage for a row. Do not mark unresolved
  or inherited gaps covered merely to pass validation.
- Equivalent abilities can share one gameplay outcome. A composed spell-family
  proof requires an Oracle-authored complete configuration, a passing outcome for
  an actual representative, a strict binding of each member's engine-consumed
  targets/program and wrappers, and each member's actual casting-route check.
  Match the whole configuration; reject unknown fields and intrinsic coupling.
  Never infer coverage from a matching instruction somewhere in a program.
- Check legality/privacy, transactional failure, and owned raw/index branch/restore
  once in core-engine tests, not throughout card cases. Bounded full-game tests
  are appropriate for the core engine, not individual cards. Prefer scenario
  helpers over new bots; do not require parser mutation suites for authored cards.
- Deep rule interactions, adversarial matrices, fuzzing, and bug-hunt regressions
  may be useful during implementation or an explicit investigation. Keep those
  exploratory tests outside the repository and ordinary CI. When a bug exposes
  missing basic functionality, add the smallest card or shared-mechanic case that
  demonstrates it, rather than committing the entire investigation.
- Run `python3 scripts/link_catalogue.py` and
  `python3 scripts/validate.py --release`; see `docs/direct-card-authoring.md`.
  Keep execution inputs fixed until consuming jobs actually terminate. Poll the
  same running process after observation timeouts. Never delete active outputs.
- Clean disposable completed build/test files promptly. Preserve every required
  shared library at its exact path/hash and preserve useful uncommitted work.
- Report complete supported cards, shared mechanics, remaining blockers and
  measured implementation/simulation throughput. Keep the full goal open until
  entire-pool behavior, interactions and performance are verified.

- Resource safety: run Cargo through `python3 scripts/resource_guard.py -- cargo ...`.
  Routine validation enters this runner automatically. It enforces one build
  across worktrees, one Cargo job/test thread, and a shared 6 GiB memory cgroup
  (including subprocesses and orphaned units). Raw Cargo compilation from a checkout fails closed. Cargo invoked from an
  unrelated cwd with --manifest-path must still use the runner.
  Do not bypass the wrapper, raise the memory cap or launch unmanaged builds.
  Keep logs/reports under ~/.cache so reboot does not erase failure evidence.

- Do not deliberately trigger the OOM killer in routine tests. Even a tiny contained
  cgroup kill can cause disruptive desktop memory warnings. Verify configured
  limits and fail-closed enforcement without killing an allocation probe.
