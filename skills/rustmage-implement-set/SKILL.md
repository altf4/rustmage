---
name: rustmage-implement-set
description: Implement complete Magic card batches in rustmage using directly authored typed Rust definitions, shared engine mechanics, source provenance and rules tests. Use for adding a set or expanding Standard coverage.
---

Implement whole cards for rustmage's deterministic in-process RL/MCTS library.
Read the checkout's `AGENTS.md` and `docs/direct-card-authoring.md` for current
interfaces and validation commands. Resolve repository paths from the active
checkout. Locate XMage through `RUSTMAGE_XMAGE_SOURCE`, or use a sibling `mage`
checkout; the shared source resolver also handles linked Git worktrees.

## Audit and finish the engine before authoring cards

Use the workspace boundary deliberately: `crates/engine/` contains shared rules
and state; `src/card_definitions/<set>/` compiles through a separate generated set
crate; integration tests link those libraries. Users still depend only on
`rustmage`. Read `docs/build-architecture.md` for the current package and test
commands. On older checkouts, use their documented paths and commands; cache
reuse across this boundary requires the split to be present. Do not move card
behavior or catalogue/name tables into the engine.

Inspect main, active branches, issues and open PRs before starting a set. Read the
whole selected set's Oracle/XMage evidence before beginning card implementation.
Map every printed obligation to existing engine support or a concrete engine
change: typed instructions, costs, targets, modes, keywords, triggers, replacements,
faces/tokens, legality/privacy, queued state, rollback and necessary performance
improvements. Include interactions between the set's mechanics in this audit.
Record the complete dependency list in the set's tracking issue; prioritize shared
work that unlocks multiple cards. A missing engine feature is a blocker, not a
reason to begin a partial card definition.

Implement **all identified engine fixes and improvements first**. Finish their
focused rules/contract tests and required reviews on the set branch before
starting the card-authoring phase. Keep unrelated prepared cards parked. Include
these engine commits in the single completed-set PR. A core rule is finished when
its meaningful interaction tests pass and it is reviewed, not when a happy path
compiles. Catalogue registration alone does not complete a card.

Then prepare the set's metadata/linkage, regenerate it with
`python3 scripts/link_catalogue.py`, and warm the engine with the same profile,
feature set and guarded Cargo target that card tests will use. Freeze that engine
baseline while implementing the set's typed card files and tests. Ordinary card,
provenance, admission and name-metadata changes should rebuild the relevant set,
assembly and test target while reusing the engine. Never add generated per-card
IDs, admission arrays or new card-specific branches to the engine merely to wire
up a definition. Check Cargo freshness records when this reuse is uncertain.

If card authoring reveals a genuinely missed engine dependency, return to the
engine phase: record and finish the shared fix, validate/review it, and rewarm the
cache before resuming dependent cards. Do not interleave speculative engine
changes with large parallel card batches. Keep the all-Standard goal open.

## Select complete-card work

Complete one full set at a time. Foundations (FDN) is the first milestone; finish
it before selecting the next set. Freeze full set membership by Oracle identity,
including reprints and starter products, and count cosmetic printings once. Source
file ownership by set is separate from membership: a reprint shares one behavior.
Use `data/sets/<code>.json` and `python3 -m scripts.set_status <CODE>` to track the
full denominator and gated cards. Do not choose a draft subset or a small bonus
product to make a full set appear complete. Within the selected set, rank batches
by whole cards unlocked per shared mechanic. Keep merged, candidate, gated and
independently verified counts separate. Existing admission flags also require
rules evidence before declaring set acceptance.
Read each card's full Oracle body, faces and authoritative rulings where needed,
plus the complete XMage class and referenced reusable abilities/effects/costs.
Treat XMage as executable evidence; resolve discrepancies using authoritative
rules and Oracle rather than copying a Java bug. Record exact source revisions,
paths/hashes and Oracle facts through the repository's provenance tooling.

Before implementation, enumerate every obligation: modes and additional costs,
optional choices, targets and restrictions, triggers, static effects, replacement
rules, alternate faces and tokens. A card with any unimplemented obligation stays
gated. For a whole set, maintain an issue listing complete cards and concrete
remaining engine blockers; do not equate catalogue metadata with support.

## Author ordinary typed Rust

Author independently editable definitions in `src/card_definitions/<set_code>/` with
their provenance records, following current checked examples. Reuse typed
instructions, targets, costs and shared rules machinery. Add a generic mechanic
when it serves a real shared rule; a small bespoke typed effect is appropriate
for a unique card. No English parser, deterministic translator, new behavior DSL
or LLM participates in gameplay. Do not reintroduce the removed compiler.

Keep immutable card definitions shared and game state owned. Preserve legality,
hidden-information boundaries, deterministic random state, transactional errors,
queued/LKI behavior and complete snapshots/branch/restore. Check affected layer,
controller, zone and object-incarnation semantics rather than assuming a copied
instruction handles them.

## Parallel ownership and integration

Use a GitHub tracking issue and isolated worktrees during implementation. Create
exactly one PR for the selected set, only after the entire set passes full-set
acceptance. Do not open draft, partial-batch, or intermediate shared-mechanic PRs
for that set work. The completed-set PR must include an update to the card
progress tracker in `README.md`, reflecting the final verified supported-card
counts and set completion status.

Assign agents disjoint card
files and integration-test targets; give a shared mechanic one owner. Begin
parallel card authorship only after the engine checkpoint above. State each batch's
base revision, source pins, dependencies and definition ownership. Let the
integrator regenerate derived module links and catalogue artifacts instead of
having every author edit a central generator. Keep the linker metadata-only.
Use current centralized schema expectations; change a wire schema only for an
actual wire change.

## Plan tests and separate authoring from acceptance

Before writing tests or selecting validation commands, read
[references/testing-strategy.md](references/testing-strategy.md). Map each printed
obligation to an independent outcome assertion or an applicable existing shared
proof. Use small deterministic scenarios, table cases for genuinely shared rules,
and representative engine-contract roots. Every changed card still needs evidence
that its own definition wires the printed behavior correctly. Exercise legality,
privacy, transactional failure and owned raw/index branching/restore where affected;
do not repeat every shared contract at every action of every card.

Reuse compiler outputs through full acceptance. The validator uses
`CARGO_TARGET_DIR` or a stable worktree cache by default; pass `--target-dir` to
share the focused authoring target. Every run still gets separate reports and
immutable library copies. On older checkouts, inspect target/cleanup behavior
and fix forced fresh targets before routine set work. Do not run `cargo clean`
between card batches. `--fresh` requires a concrete investigation or explicit
clean-build requirement, not a progress checkpoint.

Choose validation by changed behavior and integration state, not card count:

- **Author loop:** after a coherent card family or mechanic change, run its focused
  rules tests and relevant existing regressions through the resource guard, using
  one reusable development target per worktree under `~/.cache`. Select the
  relevant integration test target so unrelated card tests need not recompile. A five-card work
  chunk, agent handoff or progress update does not trigger full validation.
- **Shared-rule checkpoint:** test the mechanic and its affected consumers/contracts
  before expanding dependent card work. Broaden to native assurance when impact is
  wide or uncertain. Finish the core implementation's acceptance and reviews on
  the set branch before expanding dependent work, as required above.
- **Completed-set PR acceptance:** assemble the entire completed set, including
  its shared mechanics and README card progress tracker update, regenerate links,
  freeze its inputs, then run the full release validator using
  the verified reusable-target mode, with any additional checks required by the
  checkout. Complete full-set acceptance before opening the single set PR.
  Every merge candidate needs passing acceptance for its final
  executable inputs. Repair failures with focused
  tests first; repeat acceptance after substantive fixes are assembled. Reuse a
  successful report for the same frozen inputs instead of rerunning it for each
  worker, reviewer or status report.
- **Set acceptance:** retain the complete-set game and interaction requirements
  below. Focused author checks never establish full-set acceptance.

Coordinate one validation owner; other agents can author in separate worktrees
while a frozen integration checkout is tested. All Cargo commands must use
`python3 scripts/resource_guard.py -- cargo ...`; the repository's validation
runners enter it automatically. Preserve its single build/test concurrency and
6 GiB limit. Keep logs/reports under `~/.cache`. Distinguish build-slot wait,
compilation, test execution and benchmark time before choosing broader checks.
Do not launch a fresh full validator to diagnose a slow focused test.

Keep execution inputs fixed until their consuming job terminates. Poll the same
running process after observation timeouts. Reuse the worktree's development cache
through authoring, review fixes and acceptance; clean terminal disposable outputs
when the worktree is retired. Keep newly published libraries in immutable per-run
paths, separate from mutable compiler outputs. Never delete active outputs or
overwrite/delete required shared libraries at their exact paths/hashes. Preserve
useful work and reuse provenance/validation reports.

Obtain two independent nonauthor subagent reviews of the final published PR
head, covering complete translations and affected shared rules. A reviewer must
not count its own authored changes as independently reviewed. Address findings,
refresh reviews when substantive changes invalidate them, then integrate.

Report merged complete supported cards, shared mechanics unlocked, remaining
blockers, and measured card implementation throughput with elapsed interval and
scope. Include focused/full validation counts and measured wait/build/test time
so repeated acceptance builds are visible. Label simulation benchmarks by workload;
do not extrapolate a small Freeform smoke benchmark into all-Standard performance.
For set acceptance, require every printed obligation of every member card to be
implemented and tested, plus complete games with set-only legal decks. Exercise
all colors and the set's mechanics, interactions, legal choices and terminal
outcomes; retain deterministic replay and owned search checks. A smoke deck or
coverage manifest alone cannot certify a set. Record remaining obligations and
game coverage in its tracking issue. Only after full-set acceptance, use this
skill again to select the next full set by existing coverage and reusable
mechanics. Keep the all-Standard goal active throughout those milestones.
