# Forage costs and resolving payments

Issue [#202](https://github.com/altf4/rustmage/issues/202) adds one reusable forage operation and six complete Standard bodies. The operation exiles exactly three owned graveyard cards or sacrifices one controlled Food, then emits one completed-forage event. Incidental exile or Food sacrifice does not produce that event. These semantics follow [CR 701.61](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt) and the [official Bloomburrow release notes](https://magic.wizards.com/en/news/feature/bloomburrow-release-notes).

| Card | Complete rules text |
| --- | --- |
| Bushy Bodyguard | Offspring {2} (You may pay an additional {2} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.) When this creature enters, you may forage. If you do, put two +1/+1 counters on it. (To forage, exile three cards from your graveyard or sacrifice a Food.) |
| Camellia, the Seedmiser | Menace Other Squirrels you control have menace. Whenever you sacrifice one or more Foods, create a 1/1 green Squirrel creature token. {2}, Forage: Put a +1/+1 counter on each other Squirrel you control. (To forage, exile three cards from your graveyard or sacrifice a Food.) |
| Corpseberry Cultivator | At the beginning of combat on your turn, you may forage. (Exile three cards from your graveyard or sacrifice a Food.) Whenever you forage, put a +1/+1 counter on this creature. |
| Curious Forager | When this creature enters, you may forage. When you do, return target permanent card from your graveyard to your hand. (To forage, exile three cards from your graveyard or sacrifice a Food.) |
| Feed the Cycle | As an additional cost to cast this spell, forage or pay {B}. (To forage, exile three cards from your graveyard or sacrifice a Food.) Destroy target creature or planeswalker. |
| Treetop Sentries | Reach When this creature enters, you may forage. If you do, draw a card. (To forage, exile three cards from your graveyard or sacrifice a Food.) |

## Owned library decisions and rules

Integrated observation schema **56** adds `ChooseForageRoute`, `ChooseForage { object: ObjectRef }` and `DeclineForage`. `ForageRoute` distinguishes `ExileThree`, `Food` and Feed the Cycle's `Black` alternative. Black pays an additional black mana and never counts as foraging. The C ABI remains version 1 and the control protocol remains version 2. The standalone offspring-based implementation used schema 55; schema 56 includes both forage and private-library-selection fields.

For spell and activation costs, the method is declared before targets; actual non-targeted objects are selected after targets and before paying costs. The black requirement is included before generic cost reductions. Only fundable methods are offered. Food reservations protect the selected exact incarnation from conflicting mana-source payments. The operation uses the existing exact funding solver without hypothetical untaps or mutation during action generation.

For resolving effects, a boxed continuation holds the route, selected exact references and conditional skip location. Declining skips the conditional benefit. Choosing fewer than three graveyard cards changes only the continuation; it neither exiles cards nor emits events. The third distinct valid selection commits all three exiles atomically. Food uses the real sacrifice path, so sacrifice and death observers see the correct cause. Ownership, zones and generations are revalidated, and wrong actors, stale indexes, duplicates and unavailable resources are rejected. Capacity failure restores the action, its partial selection, zones, pending triggers and revision.

The payer's `ForageView` preserves the chosen method and references throughout target selection and payment, including a paid flag. Candidate lists and route actions appear only during their respective decisions. Other players do not receive that owned choice packet. Snapshots and clones own every continuation; no mutable global payment registry exists.

Food sacrifice observers capture the pre-event battlefield and queue once per sacrifice batch, including when several Foods are sacrificed simultaneously. The distinct forage event occurs after its cost actually moves the selected objects and queues each live controlled forage observer once. Paying through the Food route therefore triggers both kinds of observer exactly once; sacrificing Food for another purpose triggers only Food-sacrifice observers.

Curious Forager creates a separate reflexive triggered packet after successful forage. It does not choose or require a target while deciding whether to forage. Consuming every eligible permanent is legal; the later packet cannot be put on the stack without a target. When another trigger is simultaneously waiting, the untargetable reflexive can still be publicly waiting for ordering before its attempted announcement is dropped. Remaining permanent targets are selected afterward and retain ordinary response priority, zone checks and incarnation revalidation. The permanent predicate includes creatures, lands, artifacts, enchantments, planeswalkers and battles. The queued entry and reflexive abilities remain independent of a departed source.

Bushy Bodyguard's original and immutable offspring copy have separate forage entry abilities. A paid original can grow by two counters and create a fixed 1/1 copy whose own successful forage adds its own two counters. Neither paid status nor the parent's counters are copied. Camellia retains legendary status, menace, its other-Squirrel menace grant, the aggregate Food observer and its complete forage activation. Corpseberry retains all three hybrid casting equivalents, its own beginning-of-combat optional forage and its completed-forage counter observer.

Thornvault Forager remains gated: its full mana ability needs exact two-mana combinations and counterfactual funding that consumes forage resources. Osteomancer Adept remains gated: its graveyard creature permission and finality counters are separate missing rules. No partial implementations of either body are admitted.

## Catalogue, provenance and memory

Relative to offspring author base `d436325158903d1a0db3ff46cf809cd99f8ab61d`, normalized immutable definitions have exactly **six changed old rows**, **5,248 unchanged rows** and **one new Bushy copy factory**. All **70** previous token recipes and metadata, **435** existing typed helper arms, all ordinary card IDs and all prior color masks retain their semantics. Six helper arms are added. Numeric factory IDs remain scoped to the fingerprint.

Integrated on main `58414a294728761e45c05abc80c73dd532439e72`, this branch supports **1,455 / 5,164** Standard cards, with 3,709 gated identities, 71 token factories and one emblem. Its fingerprint is `e5819641011ac8befea007a7907ce34529bdec14d5b7d95c2eba00eeecbb3a43`. The standalone offspring-based implementation supported 1,423 Standard cards with fingerprint `11513e6f6dd6d15ec5cda9b87f56d85d1bfa08982bee30af4d76f3c2ee10c2ee`; measurements below were first collected there. [Pinned provenance](forage-admissions.json) records **69 SHA-256 entries across 38 Java files** at XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`; hashes were checked against both pinned blobs and the checkout.

The initialized logical state is **19,896 bytes**, unchanged from integrated main. The standalone implementation added 16 bytes to its offspring-only base; combined layout padding absorbs the library-selection continuation without another increase. A continuation allocates only when needed; logical memory accounting includes its box and initialized reference elements. The 1,000-game smoke benchmark completed 571,049 decisions at approximately 381,000 decisions/s, with clone/restore around 127/125 ns in that run. This limited-catalogue native measurement includes smoke policy and observation construction and excludes model inference, tensor encoding and JSON. It is not an XMage parity comparison.

## Validation and reproduction

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release --lib --example standard_forage
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_forage -- 100
cargo run --release --example standard_offspring -- 100
cargo run --release --bin benchmark -- 1000
```

The native suite has **1,258 tests** (1,246 unit and 12 integration), including thirteen dedicated forage adversarial tests. Compiler validation has **239 tests**. The release Python suite has **155 tests**; the added method runs thirteen complete-card scenarios covering both real forage routes and Feed's black alternative, owned views, exact resource movements, snapshots and stale-index rejection. Native probes additionally cover missing resources, foreign Food, partial decline, source departure/reentry, exact later target revalidation, two independent Bushy payments, multiple incidental Food sacrifices, funding and atomic completion overflow. The integrated branch passes **1,287 Rust tests** (1,275 unit and 12 integration), **245 compiler/data tests** and **159 fresh-release Python tests**, plus formatting, all-target Clippy, catalogue and whitespace checks. Its forage and selection Standard100 runs reproduce all prior vectors and replay totals. All 436 previous helper arms, 70 factory recipes, 5,254 color masks and 5,183 ordinary IDs remain unchanged after canonical normalization. The integrated 1,000-game smoke run measured about 408,000 decisions/s, 258 ns clone and 111 ns restore under concurrent host load; these are host-dependent observations, not a Java comparison.

The public [Standard forage workload](../examples/standard_forage.rs) plays legal 60-card decks containing the six new bodies, Daggerfang Duo, Mineshaft Spider and Bumbleflower's Sharepot. It checks every action by clone and restore/replay, predicts trigger qualifications from public events, and independently checks paid destinations, conditional effects, reflexive targets, counters, base stats and Camellia's menace grant. Across 100 games it reproduces **65,729 decisions/replays**, **1,394 independently qualified triggers**, **112,773 stat/counter/menace comparisons**, **427 graveyard payments**, **185 Food payments**, **117 black alternatives**, **610 declines**, **68 reflexive returns**, **34 paid no-target reflexives**, **25 offspring** and **61 Camellia activations**. Maximum initialized logical memory is **21,004 bytes**.

| Source, manifest order | Graveyard | Food | Black |
| --- | ---: | ---: | ---: |
| Bushy Bodyguard | 61 | 26 | 0 |
| Camellia, the Seedmiser | 44 | 17 | 0 |
| Corpseberry Cultivator | 127 | 37 | 0 |
| Curious Forager | 65 | 37 | 0 |
| Feed the Cycle | 75 | 48 | 117 |
| Treetop Sentries | 55 | 20 | 0 |

The previous offspring100 workload reproduces its 53,447 decisions/replays, all ten paid/unpaid/copy branches, both Tender activations and both Rampager evasion witnesses. Its initialized logical memory increases by the same 16 bytes.
