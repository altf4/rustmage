---
name: rustmage-implement-set
description: Implement every card in a chosen Magic Standard set in Rustmage, from a fresh checkout or a saved per-set checkpoint. Use for starting, continuing, or completing a set with typed Rust behavior, source provenance, rules tests, independent reviews, and full-game acceptance.
---

# Implement one complete Rustmage set

Take one user-selected set from its current coverage to complete, verified
behavior in Rustmage, a deterministic in-process Magic engine for reinforcement
learning and game-tree search. This is an implementation task: keep selecting,
authoring, testing and reviewing batches until that set passes acceptance. A
catalogue row, a `complete` flag, or a successful smoke game is not completion.

Accept a set name or code, an optional repository/fork, and an optional frozen
Standard date. Resolve names to codes from repository/source data. If the user
asks you to choose, select one eligible unfinished full set using current
coverage and reusable mechanics, state the choice, and stay with it. If no set
or permission to choose is supplied, ask only for the set. There is no mandatory
Foundations-first sequence. Stop at the selected set; another set is a separate
milestone unless the user requested more.

**Open exactly one implementation PR for the selected set, only after every
card is fully implemented and all set-level validation below passes.** Keep
intermediate card batches, shared mechanics and source-intake work on local
branches/worktrees, integrating them into the set branch. Do not open draft,
partial-set, per-batch or prerequisite-mechanic PRs. The tracking issue and
per-set ledger carry progress until the complete set is ready to publish.

## Start or resume

1. Locate the active Rustmage checkout; read `AGENTS.md` and
   `docs/direct-card-authoring.md`. All repository paths below are relative to
   that checkout, **not** the installed skill directory. On a new computer or
   when sources/build prerequisites are missing, follow
   [references/setup.md](references/setup.md). No prior chat, local cache, other
   installed skill, or original developer's home directory is required.
2. Inspect Git status, the default branch, worktrees, and existing GitHub issues
   and PRs before making changes. Reuse relevant work and preserve other authors'
   changes. Create or resume one set tracking issue and
   `docs/sets/<lowercase-code>.md` using
   [references/set-progress.md](references/set-progress.md). Recover source pins,
   exact commits, ownership, running jobs and next steps from those records.
3. Resolve the requested Standard scope. For “current Standard,” verify today's
   tabletop legality and bans using Wizards' official sources, then compare them
   with the checked-in target. An explicit historical snapshot stays frozen.
   Do not silently use an old snapshot as today's format, rotate a running
   project, or substitute Arena legality. Prepare a reviewed source update when
   the requested set/date is absent; see setup's source-intake section.
4. Authenticate an existing `data/sets/<code>.json` with
   `python3 -m scripts.set_status CODE --check`. If absent, reconcile complete
   source membership first, then run `python3 -m scripts.set_status CODE --freeze`
   once and check it. Plain `python3 -m scripts.set_status CODE` reports current
   admission/gating. Keep its unresolved entries visible; do not shrink the
   denominator or repeatedly freeze away discrepancies.

Set membership counts physical Oracle identities once, including reprints and
starter products; cosmetic printings do not multiply it. Alternate faces,
tokens and emblems are obligations of those physical cards. Include cards banned
from Standard in the set ledger and implement their behavior, while retaining
correct deck-legality rejection. A missing source is an explicit unresolved
member, not permission to omit the card. The current inventory tools may need
extending to reconcile such members; never equate their partial output with a
complete set. Directory ownership is separate: find an existing canonical
reprint definition through `data/catalogue-identities.json` and reuse it.

## Repeat until the selected set is accepted

### Choose a bounded, complete-card batch

Read the ledger and select the next unowned cards, ranking useful batches by
whole cards unlocked per shared mechanic. Enumerate **every** printed obligation
before coding: faces, costs, modes, optional choices, target restrictions,
triggers, static/replacement effects, durations, tokens and linked abilities.
Map each obligation to an existing tested primitive or a concrete engine change.
Treat already admitted cards as needing evidence, not automatically certified.

Finish and test the batch's shared engine dependencies before marking consumers
complete. Implement the missing mechanics rather than repeatedly choosing only
easy cards. Keep unrelated prepared work parked. Missing external sources or
permissions can block a card; ordinary engineering difficulty is work to do.
Proceed with independent unblocked work while recording genuine blockers.

### Read sources and author typed Rust

Read the full Oracle body and all faces from the pinned facts, the complete
XMage class, and every referenced reusable ability/effect/cost needed to
understand its behavior. Consult official rules and rulings for ambiguous
interactions. XMage is executable evidence; resolve discrepancies against Oracle
and the rules. If a required XMage implementation is missing, record the gap and
seek a reviewed source revision; do not invent source evidence.

Edit `src/card_definitions/<canonical-set>/<slug>.rs` and its paired `.json`.
Follow current checked definitions and `scripts/link_catalogue.py` for the schema.
Preserve stable identities and migration history; record Oracle/physical hashes,
Java revision/path/SHA-256 and reusable dependency pins for newly authored work.
A hash must describe bytes actually read. Keep `complete: false` until every
obligation is implemented and tested, including referenced synthetic definitions.

Reuse `CardRules`, `CardExtras`, typed instructions, targets and costs. Use named
keyword constants. Extend a shared mechanic for shared rules; a bespoke typed
effect can serve a unique card. Do not restore the removed English compiler,
introduce a translation framework, or use text interpretation or an LLM during
play. Keep immutable definitions shared and mutable game state owned. Preserve
legality, deterministic random state, hidden information, transactional failures,
last-known information, layers, control, zones and exact object incarnations.

### Test and integrate locally

Use existing scenario helpers and actual paid legal actions. Cover each printed
obligation and meaningful adversarial interactions: invalid/partly invalid
targets, timing, optional branches, source departure and control changes as
applicable. Check both players' observations, private choices, rejected-action
atomicity, deterministic replay, owned raw/indexed branches, parent disposal and
restore for affected behavior. Prefer focused rules scenarios to new per-card
bots. Parser mutation suites are irrelevant to directly authored Rust.

Use the GitHub tracking issue while work is in progress. Parallel authors need
isolated worktrees, disjoint card/test files and one owner per shared mechanic. Record base commits,
source pins and dependencies. The integrator alone regenerates catalogue/module
links; never hand-edit generated artifacts or let the linker interpret wording.
Integrate batches into the local set branch. For a ready local integration, run:

```sh
python3 scripts/link_catalogue.py
python3 -m scripts.set_status CODE --check
python3 scripts/validate.py --release
```

Replace `CODE` with the selected code. Run focused tests and additional binding
checks needed by the change. **Every Cargo invocation** goes through
`python3 scripts/resource_guard.py -- cargo ...`; the validator enters the guard
automatically. Keep its shared 6 GiB cgroup, one Cargo job and one test thread.
Never bypass the guard or use allocation/OOM probes. Keep logs/reports under
`~/.cache`. Freeze source and generated inputs until consuming jobs terminate;
poll the same process after observation timeouts. Clean only completed disposable
outputs, retaining required shared libraries at their exact paths/hashes.

Update the set ledger and issue after each batch. Keep admitted, locally tested,
locally reviewed and upstream-merged counts distinct. Local reviews can catch
problems early but do not replace the final published-head reviews. Select the
next batch immediately; do not stop merely because one local batch is finished.

## Finish the entire set before opening its PR

Before opening any implementation PR, require all of the following on the
combined local set branch:

- The complete frozen membership is reconciled, with zero unresolved identities
  and no unimplemented printed obligations. All required tokens/faces work and
  preexisting implementations have adequate rules evidence.
- Meaningful rules/interaction tests, provenance/link checks and release
  validation pass on the integrated local revision. New metadata alone
  does not establish gameplay support.
- Complete games with legal set-only decks cover every color and the set's
  mechanics, decisions and terminal outcomes. Use `tests/foundations_games.rs`
  and `docs/foundations-acceptance.md` as patterns, not proof for this set. Track
  actual mechanic execution; drawing or including a card in a deck does not
  test its rules. Use targeted scenarios for cards unavailable in legal decks.
- Deterministic replay, privacy, legal actions and owned branching/restore are
  verified across these workloads. Measure simulation throughput on declared
  workloads with seeds, build/revision and hardware context. The validator's
  Freeform smoke benchmark is only a regression measurement.

Only then publish one PR containing the entire set implementation, its shared
mechanics, source updates, tests and acceptance evidence. Obtain two independent
nonauthor reviews of that final published PR head. Use independent reviewer
subagents when available, or independent human reviewers; a review must exclude
the reviewer's authored ancestors. Give reviewers the Oracle/XMage sources,
entire diff, obligation ledger and test evidence. Record reviewer identities and
exact reviewed SHA. Fix findings in the same PR, rerun affected checks and refresh
invalidated reviews. No reviewer availability means “review pending,” never a
waiver or two self-reviews.

Final set acceptance additionally requires those reviews and resolution of all
findings. Merge under the user's repository permissions; absent merge permission,
leave the completed, reviewed set in its PR and report that boundary. An
interruption or blocker before the implementation and validation gates pass
requires a checkpoint, not an early PR.

## Report and hand off

Record supported cards/total, shared mechanics, remaining blockers, reviews,
validation reports and measurements in `docs/sets/<code>.md` and the issue.
Measure implementation throughput as newly verified whole cards over an actual
elapsed interval; distinguish inherited coverage, reviewed work and merged work.
Do not claim full Standard completion from one accepted set.

If interrupted, budget-limited, or blocked on all remaining work, checkpoint the
exact next action and necessary evidence using the progress reference. Report
incomplete status and the specific missing input. Resume from that checkpoint
when invoked again. This skill supplies a work loop, not a scheduler or a promise
of unlimited runtime; do not create background automations without a request.
