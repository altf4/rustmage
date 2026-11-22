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

Card and rules scenarios keep their source paths under `tests/`. Declare each
new scenario module in its set's suite entry point so Cargo links it with the
other scenarios. The native test runner accepts individual scenario names via
`--test-target`. For example, to run the Bloomburrow food-layer tests directly
with Cargo, use `python3 scripts/resource_guard.py -- cargo test --test blb_scenarios blb_food_layers_engine::`.

## Examples and bindings

- [Rust examples](examples/) show how to drive games and branch game state.
- [Python bindings](python/rustmage.py) expose the engine to Python applications.

## Build and run

Install Rust through rustup (the repository pins 1.98.0) and Python 3.10 or
newer. From the checkout root, with a Python virtual environment active, run

```sh
python -m pip install -r scripts/requirements-build.txt
python scripts/link_catalogue.py
```

**Native library for Rust, C or Python.** Build the library and print the
absolute path to the shared library for this platform.

```sh
python scripts/resource_guard.py -- cargo build --release --lib
python -c "from pathlib import Path; from scripts.build_platform import cargo_environment, library_name; print(Path(cargo_environment()['CARGO_TARGET_DIR']).resolve() / 'release' / library_name())"
```

For Python, put the checkout's `python` directory on `PYTHONPATH` and pass the
printed path to `Game(..., library=...)` or set `RUSTMAGE_LIBRARY` to it.
The filename is `librustmage.so` on Linux, `librustmage.dylib` on macOS and
`rustmage.dll` on Windows.

**Standard gameplay.** To play one game and print its terminal outcome, run

```sh
python scripts/resource_guard.py -- cargo run --release --bin benchmark -- --standard-smoke 1
```

**Diagnostic CLI.** To build the optional JSON-lines client, which reads its
configuration from stdin, run

```sh
python scripts/resource_guard.py -- cargo build --release --bin rustmage
```

The guard uses a per-checkout target directory under
`~/.cache/rustmage-targets` by default and honors `CARGO_TARGET_DIR`. Reports
are stored under `~/.cache/rustmage-build-reports`. Setting `XDG_CACHE_HOME`
changes the cache root.

Full validation also requires the pinned XMage Git objects. Set
`RUSTMAGE_XMAGE_SOURCE` to the checkout prepared by CI's source-fetch step and run

```sh
python scripts/validate.py --release --python smoke
```

## Standard

**100% complete · 21 sets · 5,459 unique cards**

Frozen tabletop/MTGO snapshot as of **2026-11-14**.
[Standard inventory](data/standard/xmage-inventory.json).

## Pioneer tracker

**20 / 68 sets complete · 29%**

`██████░░░░░░░░░░░░░░`

**5,173 / 15,079 unique Pioneer-legal cards implemented · 34%**

`███████░░░░░░░░░░░░░`

Tabletop snapshot through **2026-10-15**.

## Modern tracker

**20 / 113 sets complete · 18%**

`████░░░░░░░░░░░░░░░░`

**5,181 / 22,711 unique Modern-legal cards implemented · 23%**

`█████░░░░░░░░░░░░░░░`

Tabletop snapshot through **2026-10-15**.
