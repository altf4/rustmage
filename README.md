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

Seven full sets are implemented across accepted branches: **Foundations**,
**Lorwyn Eclipsed**, **Teenage Mutant Ninja Turtles**, **The Big Score**,
**Secrets of Strixhaven**, **Marvel Super Heroes**, and **The Hobbit**, with
accepted rules and interaction assurance.
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

**8 / 20 sets complete · 40%**

`████████░░░░░░░░░░░░`

**3,027 / 5,164 unique Standard cards implemented across accepted branches · 58%**

`████████████░░░░░░░░`

Scope: the repository's frozen **2026-10-15 tabletop/MTGO Standard snapshot**
([inventory](data/standard/xmage-inventory.json)), rather than live format legality.
“Complete” means an accepted whole-set implementation, with validation scope
recorded in its published evidence; partial admission or catalogue entries do not
establish set completion. Set completion is separate from integration into `main`.

<!-- Refresh supported counts from the union of accepted implementation branches,
     using their pinned set membership and coverage reports. Deduplicate by physical
     identity; never demote an accepted set using a checkout that lacks its merged
     integration branch. Keep current-main admission separate from accepted coverage.
     Reconcile set acceptance with its rules/game evidence and published PR. -->

| Set | Code | Supported cards | Implementation status |
| --- | --- | ---: | --- |
| Foundations | FDN | 517/517 (100%) | ✅ Complete |
| Lorwyn Eclipsed | ECL | 273/273 (100%) | ✅ Complete |
| Teenage Mutant Ninja Turtles | TMT | 195/195 (100%) | ✅ Complete |
| Aetherdrift | DFT | 111/276 (40%) | ⏳ Partial |
| Avatar: The Last Airbender | TLA | 100/286 (35%) | ⏳ Partial |
| Bloomburrow | BLB | 132/279 (47%) | ⏳ Partial |
| Duskmourn: House of Horror | DSK | 87/276 (32%) | ⏳ Partial |
| Edge of Eternities | EOE | 115/266 (43%) | ⏳ Partial |
| Final Fantasy | FIN | 135/312 (43%) | ⏳ Partial |
| Marvel Super Heroes | MSH | 281/281 (100%) | ✅ Complete |
| Marvel's Spider-Man | SPM | 79/193 (41%) | ⏳ Partial |
| Murders at Karlov Manor | MKM | 82/279 (29%) | ⏳ Partial |
| Outlaws of Thunder Junction | OTJ | 101/276 (37%) | ⏳ Partial |
| Reality Fracture | FRA | 285/285 (100%) | ✅ Complete |
| Secrets of Strixhaven | SOS | 271/271 (100%) | ✅ Complete |
| Tarkir: Dragonstorm | TDM | 130/276 (47%) | ⏳ Partial |
| The Big Score | BIG | 30/30 (100%) | ✅ Complete |
| The Hobbit | HOB | 193/193 (100%) | ✅ Complete |
| The Lost Caverns of Ixalan | LCI | 108/291 (37%) | ⏳ Partial |
| Wilds of Eldraine | WOE | 153/281 (54%) | ⏳ Partial |

Supported counts use the union of the accepted branches’ coverage reports
and full physical set membership from pinned XMage tables and Oracle facts,
including reprints and starter products. Banned members remain in the totals and
count as unsupported when gated; cosmetic printings count once. Partial-set
counts are admission flags, not full-set acceptance.

Refresh the counts when card support changes; update the status and summary
when a set passes full acceptance. When refreshing the Standard
snapshot, reconcile this list against `set_tables` in the pinned inventory. Reprints may belong to multiple sets, so
set card counts must not be added to estimate unique Standard coverage.
