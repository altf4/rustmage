# Offspring casting and immutable 1/1 copies

Issue [#198](https://github.com/altf4/rustmage/issues/198) adds ten complete Standard cards, including their optional casting costs and all residual abilities. The library exposes the decision directly; no client/server layer is involved.

| Card | Offspring cost | Other rules text |
| --- | --- | --- |
| Coruscation Mage | {2} | Whenever you cast a noncreature spell, this creature deals 1 damage to each opponent. |
| Finch Formation | {3} | Flying When this creature enters, target creature you control gains flying until end of turn. |
| Flowerfoot Swordmaster | {2} | Valiant — Whenever this creature becomes the target of a spell or ability you control for the first time each turn, Mice you control get +1/+0 until end of turn. |
| Intrepid Rabbit | {1} | When this creature enters, target creature you control gets +1/+1 until end of turn. |
| Iridescent Vinelasher | {2} | Landfall — Whenever a land you control enters, this creature deals 1 damage to target opponent. |
| Rust-Shield Rampager | {2} | This creature can't be blocked by creatures with power 2 or less. |
| Splash Lasher | {1}{U} | When this creature enters, tap up to one target creature and put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) |
| Starscape Cleric | {2}{B} | Flying This creature can't block. Whenever you gain life, each opponent loses 1 life. |
| Steampath Charger | {2} | When this creature dies, it deals 1 damage to target player. |
| Tender Wildguide | {2} | {T}: Add one mana of any color. {T}: Put a +1/+1 counter on this creature. |

The exact complete-text compiler accepts a single fixed mana offspring cost and an optional exact reminder. It compiles the whole remaining body before admitting the card. Unknown residuals, extra qualifications, malformed reminders, duplicate offspring lines and unsupported costs remain gated. Bushy Bodyguard, Darkstar Augur, Manifold Mouse, Pawpatch Recruit, Thornplate Intimidator, Thundertrap Trainer and Warren Warleader remain unsupported.

## Casting, entry and rollback

Observation schema **54** adds `Action::ChooseOffspring { pay }` and the `offspring` decision. `CastInfo.offspring: Option<bool>` is owned by the announcement and then the completed stack packet: `None` before the choice or for cards without offspring, `Some(false)` for declining, and `Some(true)` for paying. Cast origin and normal/flashback method retain their separate meanings. C ABI version 1 and control protocol 2 remain unchanged.

The optional cost is declared before targets and payment. Its colored and generic requirements are added to the selected spell cost before generic reductions. Only fundable choices are exposed; wrong actors, stale indexes and invalid choices leave the snapshot unchanged. Offspring creatures require staged announcement rather than bypassing the decision through a direct cast action.

A paid spell marks only its exact staged battlefield entry. This event queues an ordinary, suppressible offspring ETB trigger alongside other entry triggers, following APNAP and controller ordering. Countering the parent prevents its entry and copy. Torpor Orb or Doorkeeper Thrull suppresses the trigger without refunding the paid cost. Once queued, the trigger retains its immutable copy recipe even if the parent leaves. Returning that parent is a fresh incarnation without the old payment qualification. Token entries likewise do not inherit the payment, so copies never recursively produce offspring. Capacity failures restore the complete state atomically.

No permanent payment flag, mutable copy registry or branch-global history is introduced. Existing COW snapshots own the announcement, event and stack packets. The initialized logical state remains **19,880 bytes**, excluding unused capacity and allocator overhead.

## Copy identity and complete abilities

Ten fixed factories preserve each printed parent's name, mana cost and value, colors, creature types, keywords, triggers and activations, while setting the copiable base power and toughness to **1/1**. Counters and temporary effects on the parent are excluded. This bounded implementation supports these immutable recipes; it does not implement arbitrary mutable copying.

`Card::name()` and `TokenInfo.name` expose the copied printed name. `Card::identity()` and the new `TokenInfo.factory_identity` expose the distinct canonical factory key. Card serde, native lookup and JSON `card`/`source_card` fields use that key, so a copy roundtrips as a copy. `Card::from_name` with the printed parent's name still returns the ordinary card. All factory identities remain illegal deck entries, including under open deck validation.

Relative to the final flashback author base `a2d62c69650d4ddcf78e3e246780dadcfdd69b4f`, normalizing factory references proves exactly ten changed old Definition rows, **5,234 unchanged rows**, ten new factories, and unchanged ordinary card IDs. All **395** existing typed helper arms and all old color masks remain unchanged. All **60** existing token definitions and metadata rows preserve their semantics; the additive identity field records the existing factory key. Sorted numeric factory IDs remain scoped to the catalogue fingerprint.

Integrated on main `13bcfb7ae5c40df8dd5a86cabb306691fe3d4dcd`, this branch contains **1,434 / 5,164** supported Standard cards, 3,730 gated identities, 70 token factories and one emblem. Its fingerprint is `d1ae7ad81463066d46af708a9ffe6f1aa8fe38944bea1823af85e7d5632e23ad`. The standalone author branch on the final flashback base supported 1,417 Standard cards with fingerprint `89d7261c075d1601faa4cd45f1ffa0cdf42f8e40a409e0dc4ca38340bae072b2`; measurements below were first collected there. [The provenance manifest](offspring-admissions.json) records 77 SHA-256 entries across 41 Java files at pinned XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`, including all ten complete constructors, OffspringAbility and the copy implementation.

## Validation and reproduction

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release --lib --example standard_offspring
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_offspring -- 100
cargo run --release --bin benchmark -- 1000
```

Standalone validation passed **1,245 Rust tests** (1,233 unit and 12 integration), **237 compiler tests** and **154 release Python tests**. Thirteen native offspring tests cover both seats and costs, colored requirements, reductions, funding, countering, suppression, APNAP, source departure/reentry, token slot reuse, copied Tender activations, separate Flowerfoot incarnation histories, serde and allocation overflow. The Python offspring test runs twenty paid/unpaid full-card scenarios through owned observations, clone/restore and stale-index rejection. Compiler negatives keep all seven unimplemented offspring cards gated and reject unknown residual text. Integrated on current main, the Rust and fresh-release Python suites pass 1,257 (1,245 unit and 12 integration) and 155 tests. The integrated offspring and flashback 100-game runs reproduce all metrics below.

Two release runs of [standard_offspring](../examples/standard_offspring.rs), each with 100 legal 60-card Standard games, reproduced exactly **53,447 decisions and replay boundaries**. Its public ledger independently predicts trigger qualification, token properties, fixed effects, counters and targeting histories; it also checks private choice visibility. Both optional branches and offspring creation occur for all ten cards:

| Metric | Count |
| --- | --- |
| Paid casts / verified children, manifest order | 13, 11, 21, 34, 29, 7, 5, 7, 26, 26 |
| Unpaid casts, manifest order | 52, 58, 52, 46, 50, 68, 74, 77, 72, 66 |
| Independently qualified triggers | 742 |
| Public stat/keyword comparisons | 187,888 |
| Private choices | 169 |
| Tender mana activations, parent / copy | 140 / 21 |
| Tender counter activations, parent / copy | 126 / 18 |
| Rampager forbidden-block checks, parent / copy | 413 / 40 |
| Maximum initialized logical state | 21,300 bytes |

The existing 100-game flashback workload also reproduced its prior 47,896 decisions, 330 independent effect units, 56 paid discards, 220 private decisions and 7,988 replay boundaries.

Every action is replayed after restoring the pre-action snapshot. Trigger-effect witnesses cover all eight cards with triggered residual abilities; Rampager's evasion and both Tender activations have their separate public witnesses above. The smoke benchmark completed 1,000 games and 571,049 decisions at roughly **243,000 decisions/s** (425 games/s), with clone/restore around 127/124 ns in that run. This limited-catalogue measurement includes native observations and policy decisions; it excludes model inference, JSON and tensor encoding and is not an XMage parity comparison.
