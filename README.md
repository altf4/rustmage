# rustmage

An **in-process Rust library** for fast, deterministic Magic simulation and search.
No JVM, client/server connection, or network protocol runs in the simulation loop.
Python can load the same library directly using the included `ctypes` binding.

**Status: executable foundation, not a full XMage rewrite.** This version accepts
14 explicitly supported cards. Arbitrary XMage decks cannot run yet. The full
migration is tracked in [issue #1](https://github.com/altf4/rustmage/issues/1).

## Build and use

Requires stable Rust with edition 2024 support. Dependencies are only `serde` and
`serde_json`; the latter belongs to the optional control interface.

```sh
cargo test
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py' -v
cargo run --release --bin benchmark -- 1000
```

Use the Rust API from another crate with a path or Git dependency:

```rust
use rustmage::{Action, Card, Config, DeckRules, Game, Player};

let deck = [vec![Card::Mountain; 24], vec![Card::LightningBolt; 36]].concat();
let mut game = Game::new(Config {
    decks: [deck.clone(), deck], seed: 42,
    starting_player: Player::P0, deck_rules: DeckRules::Freeform,
})?;
let root = game.snapshot();
let mut branch = game.clone();
branch.apply(Player::P0, Action::Keep)?;
game.restore(&root);
# Ok::<(), rustmage::Error>(())
```

Independent clones can be sent to worker threads. A game has no background
threads, process globals, wall-clock decisions, or global RNG. All rules state,
including suspended choices and randomness, is owned and cloneable. Snapshots are
reusable. Restore advances an external revision so old action indices are rejected.

```python
from rustmage import Game

deck = ["Mountain"] * 24 + ["LightningBolt"] * 36
with Game([deck, deck], seed=42, deck_rules="Freeform") as game:
    obs = game.observe("P0")
    root = game.snapshot()
    with game.clone() as branch:
        index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "Keep")
        branch.step(obs, index)
    game.restore(root)
    game.drop_snapshot(root)
```

Run Python with `PYTHONPATH=python`. The binding locates the release library in
`target/release`; pass `library="/absolute/path/librustmage.so"` to override it.
Keep one native handle per environment. The wrapper serializes access to each
handle with a lock; separate games run independently. Close games and drop unused
checkpoints. Native Rust search bypasses the JSON encoding used by this initial
Python control interface. A packed/batched interface is a later profiling task.

## Current coverage

Supported cards: Plains, Island, Swamp, Mountain, Forest, Grizzly Bears, Goblin
Piker, Hill Giant, Llanowar Elves, Lightning Bolt, Shock, Giant Growth, Divination,
and Counterspell. JSON uses enum spellings such as `GrizzlyBears`.

The implemented slice includes two-player 20-life games, London mulligans,
bottom-card ordering, turn steps, priority and passes, instant/sorcery timing,
one land per turn, explicit mana abilities and payments, every generic mana
payment option, stack responses and resolution, target revalidation, failed-draw
loss, lethal damage, simultaneous losses, summoning sickness, attack/block
declarations, multiple blockers, damage division, cleanup discard, and temporary
power/toughness effects. Declarations and damage choices are incremental, so legal
action generation does not enumerate all subsets or blocking configurations.

`Constructed` checks 60-card minimum and the four-copy limit; **it does not claim
Modern/Standard or other format legality**. `Freeform` explicitly opts out of
those construction restrictions for experiments. Both modes require 7..=65535
cards per deck, an explicit current representation limit. Unknown cards fail to
deserialize. There is no fallback that treats unsupported cards as vanilla.

Missing: general triggered/static abilities, replacement/prevention effects,
continuous-effect layers and dependencies, most keywords, tokens/copies, counters,
planeswalkers, battles, control changes, reanimation/blink and generation-aware
references, alternate costs/modes/X, sideboarding, formats and multiplayer.
Those cards are not accepted by the catalogue. There is no claim of complete
rules conformance, XMage parity, or a ready-to-train general Magic environment.

## Search and information boundaries

`legal_actions_into` reuses a caller-owned action vector. `apply` checks actor,
timing, objects, targets, and costs before mutating. Invalid actions leave state
and revision unchanged. `apply_index` also checks the observation revision.
Concession is allowed from either player while the game is running.

`observe(player)` returns public objects plus that player's hand, and legal
actions only for the pending actor. Neither library identities/order nor the
opponent's hand or RNG appears in observations. `Game` and `Snapshot` are trusted
simulator objects containing full private state; do not give them to a policy.

Fork/restore is the substrate for MCTS, not an implemented hidden-information
search algorithm. Searching the actual hidden state would let an agent cheat.
Belief sampling/determinization and network inference hooks are tracked in
[issue #5](https://github.com/altf4/rustmage/issues/5).

## Diagnostics and provenance

`cargo run --bin rustmage` is an optional JSON-lines debugging process. Its first
input line is a `Config`; later lines use the requests in `src/protocol.rs`.
It is not required by Rust or Python callers. C ABI declarations and ownership
requirements are in `include/rustmage.h`.

Architecture and migration gates: [docs/architecture.md](docs/architecture.md).
This is new Rust code informed by the local XMage source at commit
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`; no Java implementation is vendored.
Rules reference: Wizards' [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Tests are focused independent fixtures and randomized legal-play checks; the
current tests are not differential acceptance tests against Java XMage.
