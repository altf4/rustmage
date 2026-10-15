# rustmage

An **in-process Rust library** for fast, deterministic Magic simulation and search.
No JVM, client/server connection, or network protocol runs in the simulation loop.
Python can load the same library directly using the included `ctypes` binding.

**Status: migration in progress, not a full XMage rewrite.** The native catalogue
knows all 5,164 pinned Standard identities; 656 have executable rules so far.
There are also six non-Standard regression cards. Unsupported decks are rejected. The full
migration is tracked in [issue #1](https://github.com/altf4/rustmage/issues/1).
The first training target is [current Standard, pinned to October 15, 2026](docs/standard-target.md).

## Build and use

Requires stable Rust with edition 2024 support. Dependencies are only `serde` and
`serde_json`; the latter belongs to the optional control interface.

```sh
cargo test
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py' -v
cargo run --release --bin benchmark -- 1000
cargo run --release --example branch
cargo run --release --example standard
cargo run --release --example standard_triggers -- 100
cargo run --release --example standard_counters -- 100
cargo run --release --example standard_lands -- 100
cargo run --release --example standard_search -- 100
cargo run --release --example standard_modes -- 100
cargo run --release --example standard_groups -- 100
cargo run --release --example standard_legends -- 100
cargo run --release --example standard_casts -- 100
cargo run --release --example standard_steps -- 100
cargo run --release --example standard_payments -- 100
cargo run --release --example standard_vital -- 100
cargo run --release --example standard_equipment -- 100
cargo run --release --example standard_tokens -- 100
cargo run --release --example standard_activations -- 100
cargo run --release --example standard_mana -- 100
cargo run --release --example standard_library -- 100
cargo run --release --example standard_stats -- 100
cargo run --release --example standard_keywords -- 100
cargo run --release --example standard_blocking -- 100
cargo run --release --example standard_exile -- 100
cargo run --release --example standard_targeting -- 100
cargo run --bin coverage
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

The [inventory and coverage report](docs/standard-inventory.md) records all legal
identities and implemented definitions. `cargo run --bin coverage` reports actual
native coverage. JSON uses canonical English names; legacy tokens such as
`GrizzlyBears` remain accepted as aliases.

The implemented slice includes two-player 20-life games, London mulligans,
bottom-card ordering, turn steps, priority and passes, instant/sorcery timing,
one land per turn, explicit mana abilities and payments, every generic mana
payment option, stack responses and resolution, target revalidation, failed-draw
loss, lethal damage, simultaneous losses, summoning sickness, attack/block
declarations, multiple blockers, damage division, cleanup discard, and temporary
power/toughness effects. Declarations and damage choices are incremental, so legal
action generation does not enumerate all subsets or blocking configurations.
The implemented keyword rules are flying, reach, haste, vigilance, defender,
flash, lifelink, deathtouch, trample, first strike, double strike, hexproof and
indestructible and menace. Fixed temporary keyword gains share these rules.
Incremental blocks enforce cumulative evasion and blocker-count restrictions. Damage uses
source-aware simultaneous batches; strike steps and damage choices can be copied
and restored. Zone transitions assign new object generations, reset transient state, and remove
combat membership. Targets retain their chosen incarnation. A strict compiler
admits exact vanilla/keyword-only creatures and the complete text of Unsummon,
Fell, Murder and Bilbo’s Deadly Slice. Typed immutable programs also implement
entry/death/attack triggers, controller ordering, draw/discard and life effects for
31 additional creatures and two draw spells. Optional effects and private discard
choices suspend in owned continuations. Shared target predicates and incremental
announcements support targeted triggers, multi-target spells, partial target
invalidation and reanimation. See [targeting](docs/targeting.md),
[effect programs](docs/programs.md), [activated abilities](docs/activations.md), [mana](docs/mana.md), [payment windows](docs/payment-windows.md), [private scry/surveil](docs/library-choices.md), [derived stats](docs/derived-stats.md), [effective keywords](docs/effective-keywords.md), [blocking restrictions](docs/blocking-restrictions.md), [linked exile](docs/linked-exile.md) and
[object identity](docs/object-identity.md).

`Constructed` checks 60-card minimum and the four-copy limit; **it does not claim
Modern/Standard or other format legality**. `Standard` separately enforces the
frozen legal-card pool and copy restrictions, then rejects unsupported semantics.
`Freeform` explicitly opts out of
those construction restrictions for experiments. All modes require 7..=65535
cards per deck, an explicit current representation limit. Unknown cards fail to
deserialize; known cards with unimplemented rules fail game creation. There is
no fallback that treats unsupported cards as vanilla.

Missing: most other triggered/static abilities, multiple target groups, broader costs/modes,
replacement/prevention effects,
continuous-effect layers and dependencies, most keywords, general token abilities/copies, wider counter operations,
planeswalkers, battles, control changes, blink, alternate costs/modes/X, sideboarding, formats and multiplayer.
Those cards are known by the catalogue when Standard legal, but cannot enter a
game until implemented. There is no claim of complete
rules conformance, XMage parity, or a ready-to-train general Magic environment.

## Search and information boundaries

`legal_actions_into` reuses a caller-owned action vector. `apply` checks actor,
timing, objects, targets, and costs before mutating. Invalid actions leave state
and revision unchanged. `apply_index` also checks the observation revision.
Concession is allowed from either player while the game is running.

`observe(player)` returns public objects plus that player's hand, and legal
actions only for the pending actor. During a scry/surveil continuation, only its
actor receives looked-at card views and partial destination orders. Other library
identities/order, the opponent's hand and RNG remain hidden. `Game` and `Snapshot` are trusted
simulator objects containing full private state; do not give them to a policy.

Fork/restore is the substrate for MCTS, not an implemented hidden-information
search algorithm. Searching the actual hidden state would let an agent cheat.
`examples/branch.rs` demonstrates isolated root-action rollouts and restoration;
it deliberately labels the example as privileged perfect-information planning.
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

`scripts/benchmark.py --games 10000 --repeats 5 --cpu N --output report.json`
records repeated release measurements with CPU/compiler metadata. Build the
release binary first; `--cpu` is optional and pins only the benchmark subprocess.
The [initial measurements](docs/benchmarks.md) report roughly 2 million native
decisions/second on the limited workload, with 2.7 KB of initial state payload.

[Nonbasic lands](docs/nonbasic-lands.md) include exact pre-entry tapped conditions,
unrestricted choice mana, complete entry triggers and selected utility abilities.

[Library searches](docs/library-search.md) provide private incremental choices,
qualified fail-to-find, deterministic shuffling, fetches and public reveal knowledge.

[Modal spells](docs/spell-modes.md) choose a legal mode before its targets and
payment, with mode-specific resolution and complete announcement replay.

[Distinct target groups](docs/target-groups.md) bind independent target occurrences
and support current-power noncombat fight/bite damage with complete replay.

Legendary permanents and snapshot-owned state-based choices are described in
[the legend-rule milestone](docs/legend-rule.md).

Completed-cast triggers, prowess and public turn-local spell counts are described
in [the casting milestone](docs/cast-triggers.md).

Beginning-of-step events, survival, raid and intervening-if checks are described
in [the step-trigger milestone](docs/step-triggers.md).

Simultaneous sacrifices and exact conditional discard/sacrifice costs are described
in [the resolution-payment milestone](docs/resolution-payments.md).
