# Equipment-owned and creature-granted attack triggers

Issue [#214](https://github.com/altf4/rustmage/issues/214) adds ten complete Standard Equipment bodies on standalone parent `e4480721866ef8a14d921fc2c652ffee417e65f4`. This branch's counts exclude the separately integrated target-cost and subject-condition stages. Public Python and Standard acceptance are authored in a separate worktree.

| Card | Complete rules |
| --- | --- |
| Baseball Bat | {G}{W}, entry attachment, +1/+1, Equipment-owned optional creature tap on equipped attack, equip {3} |
| Captain America's Shield | Legendary {2}, intrinsic indestructible, +0/+8 and vigilance, Equipment-owned defending-player creature tap, equip {2} |
| Stormbeacon Blade | {1}{W}, +3/+0, Equipment-owned attack trigger drawing only with three currently attacking controlled creatures at resolution, equip {2} |
| Thunder Lasso | {2}{W}, entry attachment, +1/+1, Equipment-owned defending-player creature tap, equip {2} |
| Ultima Weapon | Legendary {7}, +7/+7, Equipment-owned opponent-creature destruction on equipped attack, equip {7} |
| Candlestick | {U} Clue Equipment, +1/+1, grants the creature attack-surveil two, {2} plus sacrifice draws one, equip {2} |
| Diamond Pick-Axe | {R}, intrinsic indestructible, +1/+1, grants attack-Treasure creation, equip {2} |
| Medic's Kitesail | {2}, +1/+0 and flying, grants attack-life gain one, equip {2} |
| Pirate Hat | {1}{U}, +1/+1, grants attack-draw then discard, restricted Pirate equip {1} and ordinary equip {2} |
| Web-Shooters | {1}{W}, +1/+1 and reach, grants attack-opponent-creature tap, equip {2} |

The [provenance manifest](attachment-attack-admissions.json) records every complete admitted Oracle text, six fully gated candidates, and **91 SHA-256 entries across 43 Java files**, verified against both the checkout and pinned revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Atomic Microsizer, Genji Glove, Saw, Black Mage's Rod, White Mage's Staff and Wrench remain unsupported: base setting, extra combats, grantor/attacker-bound sacrifice exclusions, Job Select, dynamic types and granted activations are outside this stage. Unknown residual text rejects the whole card.

## Exact source and immutable program provenance

Equipment-owned `EquippedAttacks` and creature-granted `GrantedAttacks` are distinct immutable trigger events. The engine walks its existing live attachment-source index once after the complete attacker declaration, including vigilance attacks that never tap. Partial selections, tapping outside attack declaration, and attachments to creatures that did not attack do not trigger. Current exact incarnation links qualify the event; subsequent detachment, re-equipping, grantor departure and recipient departure cannot substitute another source or cancel an already queued independent ability.

Observation schema **59** adds `SpellView.attachment_trigger`, either null or an owned boxed record:

```text
{ grantor: ObjectRef, granting_card: Card, attacker: ObjectRef,
  defender: Player, granted: bool }
```

For an Equipment-owned ability, the ordinary `source_card`, `card`, `object` and `controller` describe the Equipment. For a quoted granted ability, those fields describe the actual recipient creature. The immutable `granting_card` supplies the granted program and target plan at the published ability index; it never impersonates the creature's printed source card. Multiple identical attachments produce independent stack IDs and distinct grantor references, and coexist with the creature's printed abilities even when their definition-local indices coincide. Ordinary spell and ability packets carry null context. C ABI stays 1 and control protocol stays 2.

Both viewers can observe this public source provenance. Actor-private surveil and discard continuations retain their existing private views and action lists. Context, program selection, pending ordering, targets, RNG and private continuations are owned by cloned states and snapshots. There is no global mutable ability registry or per-object ability graph. [CR 113.7–113.7a and 603.3a–b](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf) govern source identity, independent abilities and ordinary APNAP ordering.

Defender targeting resolves the immutable `DefendingCreature` placeholder using the captured defending player, independently of the Equipment controller and any later attachment link. Accepted games remain two-player with owner/controller equivalence; attacks on the opposing player or their planeswalker have that same defending player. Native tests additionally isolate a foreign-owned Equipment attachment without introducing a control-changing card. Normal target revalidation, ward and hexproof still apply.

Stormbeacon's threshold is an effect condition, not an intervening-if trigger condition: its packet is always queued for the equipped attack, then counts live controlled attacking creatures when resolving. The shared attacking-count evaluator now explicitly checks controller ownership, including the foreign-owned fixture. Intrinsic Equipment indestructible never grants indestructible to its creature. Pirate's restricted equip ability uses normal targeting and sorcery timing; Changeling can satisfy Pirate, while the ordinary option remains available for every owned creature.

## Validation and catalogue preservation

Standalone support becomes **1,505 / 5,164** Standard identities, plus six regression cards, **72 token factories** and one emblem. Exactly ten of 5,256 definition rows change; the other **5,246** rows, all identities, all **5,183 ordinary card IDs**, every previous color mask, all recipes/metadata and all **446 prior numeric helper arms** remain unchanged. Ten Equipment helper rows are added. No token factory is added. Fingerprint: `af84fcf95e137ab9a8dd9da6e41af0b8905254e0cdcc81518faa23f405178b1f`.

Thirteen native test groups exercise paid complete casts/equips and all residuals in both seats; exact granted/owned source identity; printed and multiple granted abilities; partial declarations and reattachment; source/recipient departure and reincarnation; vigilance; optional zero targets and full fizzles; live Stormbeacon counts; Pirate alternatives and timing; private surveil/discard ownership; ward payment/decline, hexproof, wrong actors and APNAP placement across Equipment/creature controllers; intrinsic indestructible; cloned divergent choices; stale revisions; and early/late capacity rollback. Logical state-byte accounting includes every allocated attack-context record, including private pending and resolving packets. A memory test removes only those records from a cloned fixture and checks the exact reported difference.

The full core passes **1,346 Rust tests** (1,334 unit and twelve integration), **162 fresh-release Python tests** (90.858 seconds), all-target Clippy with warnings denied, formatting, generated-catalogue verification and whitespace checks. All **256 compiler/data tests** pass in 344.223 seconds.

Initialized logical state remains **19,944 bytes**, unchanged from this exact parent. Each active attack-context record adds **40 initialized bytes**; `SpellView` is 256 bytes including its optional pointer. The native fixture verifies accounting for two simultaneous owned records. A 1,000-game smoke run completed 571,049 decisions at about 293,000 decisions/s, with 158 ns clone and 162 ns restore under concurrent host load. This includes native policy and observation construction, excludes model inference, tensors, JSON, spare capacity, allocator overhead and external retained snapshots, and is not an XMage comparison. Public Standard acceptance is a separate commit and must verify full private transactions, actual Treasure use and sacrifice-to-draw, rather than treating an intermediate continuation action as a completed effect.

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
python3 -m unittest discover -s python -p 'test_*.py'
cargo clippy --all-targets -- -D warnings
cargo fmt --check
python3 scripts/generate_catalogue.py --check
cargo run --release --bin benchmark -- 1000
```
