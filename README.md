# Rustmage

Rustmage is a Magic: The Gathering rules engine written in Rust. 
It aims to be an extremely efficient rules enforcement engine that runs in the 
same process as your application. Rustmage is not a server, but you could make
one using it!

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

Card behavior is authored directly in Rust, organized by set, and built on shared
abilities, effects, targets, and costs. Implementations attempt to favor rules
being placed in the core engine, with cards merely picking from sets of common 
abilities. However, there are many mechanically unique effects that keep this
from being universal. 

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

Build outputs are reused under `~/.cache/rustmage-targets` unless
`CARGO_TARGET_DIR` is set. Reports remain under `~/.cache/rustmage-build-reports`.

Full validation also requires the pinned XMage Git objects (see CI's source-fetch
step). Set `RUSTMAGE_XMAGE_SOURCE` to that checkout and run:

```sh
python scripts/validate.py --release --python smoke
```

## Standard set tracker

**13 / 20 sets complete · 65%**

`█████████████░░░░░░░`

**4,005 / 5,163 unique Standard cards supported in this checkout · 78%**

`████████████████░░░░`

Scope: the repository's frozen **2026-10-15 tabletop/MTGO Standard snapshot**
([inventory](data/standard/xmage-inventory.json)), rather than live format legality.

<!-- Refresh supported counts from this checkout's generated coverage report
     and pinned set membership. Deduplicate the overall count by physical
     identity. Reconcile set completion with its rules and game evidence and
     published pull request. -->

| Set | Code | Supported cards | Implementation status |
| --- | --- | ---: | --- |
| Foundations | FDN | 517/517 (100%) | ✅ Complete |
| Lorwyn Eclipsed | ECL | 273/273 (100%) | ✅ Complete |
| Teenage Mutant Ninja Turtles | TMT | 195/195 (100%) | ✅ Complete |
| Aetherdrift | DFT | 276/276 (100%) | ✅ Complete |
| Avatar: The Last Airbender | TLA | 108/286 (38%) | ⏳ Partial |
| Bloomburrow | BLB | 135/279 (48%) | ⏳ Partial |
| Duskmourn: House of Horror | DSK | 91/276 (33%) | ⏳ Partial |
| Edge of Eternities | EOE | 266/266 (100%) | ✅ Complete |
| Final Fantasy | FIN | 312/312 (100%) | ✅ Complete |
| Marvel Super Heroes | MSH | 281/281 (100%) | ✅ Complete |
| Marvel's Spider-Man | SPM | 193/193 (100%) | ✅ Complete |
| Murders at Karlov Manor | MKM | 86/279 (31%) | ⏳ Partial |
| Outlaws of Thunder Junction | OTJ | 106/276 (38%) | ⏳ Partial |
| Reality Fracture | FRA | 285/285 (100%) | ✅ Complete |
| Secrets of Strixhaven | SOS | 271/271 (100%) | ✅ Complete |
| Tarkir: Dragonstorm | TDM | 276/276 (100%) | ✅ Complete |
| The Big Score | BIG | 30/30 (100%) | ✅ Complete |
| The Hobbit | HOB | 193/193 (100%) | ✅ Complete |
| The Lost Caverns of Ixalan | LCI | 117/291 (40%) | ⏳ Partial |
| Wilds of Eldraine | WOE | 156/281 (56%) | ⏳ Partial |

## Modern tracker

**13 / 113 sets complete · 12%**

`██░░░░░░░░░░░░░░░░░░`

**4,013 / 22,711 unique Modern-legal cards implemented · 18%**

`████░░░░░░░░░░░░░░░░`

Tabletop snapshot through **2026-10-15**.
