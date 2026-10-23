# Rustmage

Rustmage is a Magic: The Gathering rules engine written in Rust for reinforcement
learning and game-tree search. It runs in the same process as your training or
search code, providing a simulation environment for developing and evaluating
bots that play Magic.

A program creates a game, reads a player's observation, and chooses an action.
Rustmage enforces the rules, resolves the consequences, and advances to the next
decision. Each player's observation contains the information available to that
player. A fixed random seed and the same sequence of decisions reproduce the
same game.

Search algorithms can save a position, create independent branches, explore
possible continuations, and restore an earlier state. Branches share memory
where possible while keeping their changes isolated. This makes branching and
rollback central parts of the engine, supporting workloads such as Monte Carlo
tree search.

The engine is available through a native Rust API and in-process Python bindings.
Your application supplies the bot, learning algorithm, and reward function;
Rustmage supplies the game rules and state transitions.

Three full sets are implemented: **Foundations**, **Lorwyn Eclipsed**, and
**Teenage Mutant Ninja Turtles**, with rules tests and complete-game acceptance.
The long-term goal is complete Standard coverage; implementation of the wider
card pool is still in progress. Standard games reject cards whose rules have not
been implemented.

Card behavior is authored directly in Rust, organized by set, and built on shared
abilities, effects, targets, and costs. XMage's implementations and authoritative
Oracle text guide that work. Gameplay executes deterministic Rust code.

To explore the project:

- [Rust examples](examples/) show how to drive games and branch game state.
- [Python bindings](python/rustmage.py) expose the engine to Python applications.
- [Card implementation guide](docs/direct-card-authoring.md) explains how to add card behavior.
- [Foundations validation](docs/foundations-acceptance.md) describes the coverage and testing process.

## Standard set tracker

**3 / 20 sets complete · 15%**

`███░░░░░░░░░░░░░░░░░`

Scope: the repository's frozen **2026-10-15 tabletop/MTGO Standard snapshot**
([inventory](data/standard/xmage-inventory.json)), rather than live format legality.
“Complete” means whole-set implementation with rules and full-game acceptance;
partial admission or catalogue entries do not establish set completion. Upstream
review and merge status are tracked separately in the linked records.

<!-- Refresh supported counts from scripts/set_status.py freeze/status using the
     pinned XMage tables, Oracle facts and data/standard/coverage.json. Reconcile
     accepted sets with data/sets/<code>.json. Keep summary and rows in sync when
     a set passes acceptance; definition-directory counts omit shared reprints. -->

| Set | Code | Supported cards | Implementation status | Acceptance / tracking |
| --- | --- | ---: | --- | --- |
| Foundations | FDN | 517/517 (100%) | ✅ Complete | [Acceptance](docs/foundations-acceptance.md) |
| Lorwyn Eclipsed | ECL | 273/273 (100%) | ✅ Complete | [Set record](docs/sets/ecl.md) |
| Teenage Mutant Ninja Turtles | TMT | 195/195 (100%) | ✅ Complete | [Set record](docs/sets/tmt.md) |
| Aetherdrift | DFT | 110/276 (40%) | ⏳ Partial | — |
| Avatar: The Last Airbender | TLA | 97/286 (34%) | ⏳ Partial | — |
| Bloomburrow | BLB | 127/279 (46%) | ⏳ Partial | — |
| Duskmourn: House of Horror | DSK | 85/276 (31%) | ⏳ Partial | — |
| Edge of Eternities | EOE | 110/266 (41%) | ⏳ Partial | — |
| Final Fantasy | FIN | 133/312 (43%) | ⏳ Partial | — |
| Marvel Super Heroes | MSH | 84/281 (30%) | ⏳ Partial | — |
| Marvel's Spider-Man | SPM | 76/193 (39%) | ⏳ Partial | — |
| Murders at Karlov Manor | MKM | 81/279 (29%) | ⏳ Partial | — |
| Outlaws of Thunder Junction | OTJ | 101/276 (37%) | ⏳ Partial | — |
| Reality Fracture | FRA | 89/285 (31%) | ⏳ Partial | — |
| Secrets of Strixhaven | SOS | 105/271 (39%) | ⏳ Partial | — |
| Tarkir: Dragonstorm | TDM | 127/276 (46%) | ⏳ Partial | — |
| The Big Score | BIG | 5/30 (17%) | ⏳ Partial | — |
| The Hobbit | HOB | 69/193 (36%) | ⏳ Partial | — |
| The Lost Caverns of Ixalan | LCI | 105/291 (36%) | ⏳ Partial | — |
| Wilds of Eldraine | WOE | 150/281 (53%) | ⏳ Partial | — |

Supported counts use the current [coverage report](data/standard/coverage.json)
and full physical set membership from pinned XMage tables and Oracle facts,
including reprints and starter products. Banned members remain in the totals and
count as unsupported when gated; cosmetic printings count once. Partial-set
counts are admission flags, not full-set acceptance.

Refresh the counts when card support changes; update the status, evidence link
and summary when a set passes full acceptance. When refreshing the Standard
snapshot, reconcile this list against `set_tables` in the pinned inventory. Reprints may belong to multiple sets, so
set card counts must not be added to estimate unique Standard coverage.

## Implement a set with Codex

The portable [rustmage-implement-set skill](skills/rustmage-implement-set/SKILL.md)
lets a new contributor choose a Standard set and work through its remaining
cards, shared mechanics, tests and reviews until full-set acceptance. Its
[fresh-machine setup](skills/rustmage-implement-set/references/setup.md) covers
source pins, Linux build requirements and source intake for newly added sets.
Progress is saved by set in `docs/sets/<code>.md` and a GitHub tracking issue.
Implementation batches stay local. Open one PR only after the entire set is
implemented and passes set-level validation, then obtain the final reviews.

From a checkout on a new machine, install the skill:

```sh
mkdir -p "$HOME/.agents/skills"
cp -R skills/rustmage-implement-set "$HOME/.agents/skills/"
```

Inspect an existing installation before replacing it. Open the checkout in Codex
and give it this prompt, replacing `SET_CODE` with the desired set:

```text
Use $rustmage-implement-set to implement every card in SET_CODE in this Rustmage
checkout. Verify its Standard scope, set up the pinned sources, and resume or
create its per-set progress record and tracking issue. Keep implementing shared
mechanics and complete card batches locally until every card is implemented and
all set-level validation passes. Only then open one PR for the complete set and
obtain two independent nonauthor reviews. Do not open draft, partial-set, batch
or prerequisite PRs. Use isolated worktrees for parallel authors and the required
resource guard for all builds. Checkpoint
progress if interrupted, and stop after this set.
```

For a later session: `Use $rustmage-implement-set to resume SET_CODE from its
per-set progress record and tracking issue.` The workflow is resumable; it does
not create a background scheduler or override Codex runtime limits.
