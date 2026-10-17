---
name: rustmage-implement-set
description: Implement complete Magic card batches in rustmage using directly authored typed Rust definitions, shared engine mechanics, source provenance and rules tests. Use for adding a set or expanding Standard coverage.
---

Implement whole cards for rustmage's deterministic in-process RL/MCTS library.
Read the checkout's `AGENTS.md` and `docs/direct-card-authoring.md` for current
interfaces and validation commands. Resolve repository paths from the active
checkout; the usual primary checkout is `/home/altf4/Code/rustmage` and XMage is
`/home/altf4/Code/mage`.

## Finish engine dependencies first

Inspect main, active branches, issues and open PRs before starting a batch.
Preserve useful work. Identify shared rules missing from the selected cards and
finish the relevant core implementations and their reviews before expanding the
batch. Keep unrelated prepared card work parked in its own worktree. A core rule
is finished when its meaningful interaction tests pass and its reviewed change
is integrated, not when its happy path compiles. Keep the all-Standard goal open.

## Select complete-card work

Use the pinned Standard pool and coverage report to choose a set or coherent
batch. Rank batches by complete cards unlocked per shared mechanic, not number
of text patterns matched. Keep merged, candidate and gated counts separate.
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

Author independently editable card definitions in `src/card_definitions/` with
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

Use GitHub issues and PRs and isolated worktrees. Assign agents disjoint card
files and test modules; give a shared mechanic one owner. State each batch's
base revision, source pins, dependencies and definition ownership. Let the
integrator regenerate derived module links and catalogue artifacts instead of
having every author edit a central generator. Keep the linker metadata-only.
Use current centralized schema expectations; change a wire schema only for an
actual wire change.

## Prove the complete behavior

Use existing scenario helpers for meaningful paid legal actions and adversarial
interactions. Cover every printed obligation, invalid/partially invalid targets,
optional choices, timing and relevant controller/source departure cases. Exercise
both seats, public/private observations, illegal-action atomicity, owned raw and
indexed branches, parent disposal and restore where affected. Choose tests for
rules risks, not one bespoke bot per card or parser mutation coverage.

Run the repository's routine validator and focused new rules tests with fixed
source inputs. Poll running jobs to terminal completion; an observation timeout
is not a reason to restart them. Use isolated build targets and promptly remove
terminal disposable artifacts. Preserve active work and required shared libraries
at their exact paths and hashes. Reuse automated provenance/validation reports.

Obtain two independent nonauthor subagent reviews of the final published PR
head, covering complete translations and affected shared rules. A reviewer must
not count its own authored changes as independently reviewed. Address findings,
refresh reviews when substantive changes invalidate them, then integrate.

Report merged complete supported cards, shared mechanics unlocked, remaining
blockers, and measured card implementation throughput with elapsed interval and
scope. Label simulation benchmarks by workload; do not extrapolate a small
Freeform smoke benchmark into all-Standard performance. Apply the skill to the
next ready batch once its core dependencies are integrated.
