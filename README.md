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

Ten full sets are implemented across accepted branches: **Foundations**,
**Lorwyn Eclipsed**, **Teenage Mutant Ninja Turtles**, **The Big Score**,
**Secrets of Strixhaven**, **Marvel Super Heroes**, **The Hobbit**,
**Reality Fracture**, **Aetherdrift**, and **Tarkir: Dragonstorm**, with
accepted rules and interaction assurance.
The long-term goal is complete Standard coverage; implementation of the wider
card pool is still in progress. Standard games reject cards whose rules have not
been implemented.

Card behavior is authored directly in Rust, organized by set, and built on shared
abilities, effects, targets, and costs. XMage's implementations and authoritative
Oracle text guide that work. Gameplay executes deterministic Rust code.

The Cargo workspace keeps the shared engine, per-set definitions and integration
tests in separate compilation units. Applications still use one `rustmage`
dependency. See the repository's [authoring and validation instructions](AGENTS.md).

## Examples and bindings

- [Rust examples](examples/) show how to drive games and branch game state.
- [Python bindings](python/rustmage.py) expose the engine to Python applications.

## Build and run

Install Rust through rustup (the repository pins 1.98.0, matching CI) and Python
3.10 or newer. In a Python
virtual environment, install the portable build supervisor:

```sh
python -m pip install -r scripts/requirements-build.txt
python scripts/link_catalogue.py
python scripts/resource_guard.py -- cargo build --release --lib --bin rustmage --example standard
python scripts/resource_guard.py -- cargo run --release --example standard -- 1
```

These commands work in native macOS, Linux and Windows terminals with the virtual
environment activated (`python3` can be used instead of `python`). The example
plays a Standard game and prints its terminal outcome. The `rustmage` executable
is a JSON-lines diagnostic client; it expects a configuration on stdin.

The guard serializes builds across worktrees and limits Cargo and test concurrency
to one. It uses a hard 6 GiB cgroup limit when a Linux user systemd manager is
available. Other hosts use a 6 GiB aggregate RSS watchdog, sampled every 100 ms.
That budget can overshoot between samples; it is not a hard allocation cap.
Run markers recover detached children, and a live supervisor is required for
compiler admission. This prevents accidental unmanaged builds, not deliberate
escape by processes that remove their markers or forcibly kill the supervisors.
Windows builds bootstrap a small native compiler launcher inside the guard,
avoiding shell limits on long compiler argument lists.
`--backend cgroup` requires hard Linux containment; `--backend portable` selects
the watchdog. Once cgroup setup starts, failures stop the build.

Build outputs are reused under `~/.cache/rustmage-targets` unless
`CARGO_TARGET_DIR` is set. Reports remain under `~/.cache/rustmage-build-reports`.

Full validation also requires the pinned XMage Git objects (see CI's source-fetch
step). Set `RUSTMAGE_XMAGE_SOURCE` to that checkout and run:

```sh
python scripts/validate.py --release --python smoke
```

Validation retains a hashed native library snapshot (`.so`, `.dylib`, or `.dll`)
with its report. The build never requires Java or a running XMage server.

Card provenance that cites a path under `docs/` refers to evidence retained in
[the repository history](https://github.com/altf4/rustmage/tree/b072c7932fc363b51938d429b99cb1b43d4beca7/docs).

## Standard set tracker

**10 / 20 sets complete · 50%**

`██████████░░░░░░░░░░`

**3,576 / 5,164 unique Standard cards implemented across accepted branches · 69%**

`██████████████░░░░░░`

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
| Aetherdrift | DFT | 276/276 (100%) | ✅ Complete |
| Avatar: The Last Airbender | TLA | 108/286 (38%) | ⏳ Partial |
| Bloomburrow | BLB | 135/279 (48%) | ⏳ Partial |
| Duskmourn: House of Horror | DSK | 91/276 (33%) | ⏳ Partial |
| Edge of Eternities | EOE | 122/266 (46%) | ⏳ Partial |
| Final Fantasy | FIN | 136/312 (44%) | ⏳ Partial |
| Marvel Super Heroes | MSH | 281/281 (100%) | ✅ Complete |
| Marvel's Spider-Man | SPM | 82/193 (42%) | ⏳ Partial |
| Murders at Karlov Manor | MKM | 86/279 (31%) | ⏳ Partial |
| Outlaws of Thunder Junction | OTJ | 106/276 (38%) | ⏳ Partial |
| Reality Fracture | FRA | 285/285 (100%) | ✅ Complete |
| Secrets of Strixhaven | SOS | 271/271 (100%) | ✅ Complete |
| Tarkir: Dragonstorm | TDM | 276/276 (100%) | ✅ Complete |
| The Big Score | BIG | 30/30 (100%) | ✅ Complete |
| The Hobbit | HOB | 193/193 (100%) | ✅ Complete |
| The Lost Caverns of Ixalan | LCI | 117/291 (40%) | ⏳ Partial |
| Wilds of Eldraine | WOE | 156/281 (56%) | ⏳ Partial |

Supported counts use the union of the accepted branches’ coverage reports
and full physical set membership from pinned XMage tables and Oracle facts,
including reprints and starter products. Banned members remain in the totals and
count as unsupported when gated; cosmetic printings count once. Partial-set
counts are admission flags, not full-set acceptance.

Refresh the counts when card support changes; update the status and summary
when a set passes full acceptance. When refreshing the Standard
snapshot, reconcile this list against `set_tables` in the pinned inventory. Reprints may belong to multiple sets, so
set card counts must not be added to estimate unique Standard coverage.
