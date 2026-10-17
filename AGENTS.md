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
- Test actual rules interactions, legality/privacy, transactional failures and
  owned raw/index branch/restore. Prefer existing scenario helpers over new bots;
  do not require parser mutation suites for authored Rust cards.
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
