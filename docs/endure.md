# Fixed Endure and optional resolution payments

Issue [#237](https://github.com/altf4/rustmage/issues/237), PR [#241](https://github.com/altf4/rustmage/pull/241). Eight complete bodies are executable: Anafenza, Unyielding Lineage; Descendant of Storms; Dusyut Earthcarver; Fortress Kin-Guard; Inspirited Vanguard; Kin-Tree Nurturer; Sandskitter Outrider; Sinkhole Surveyor. The combined accepted Mobilize baseline executes **1,664 / 5,164 Standard identities**, schema **70**, with **81 immutable token factories**. Catalogue fingerprint: `e1bc2e84bd56971a17d789743c446eb5260b8925df98d0f780bb6da79d243788`.

CR701.63 creates one N/N white Spirit unless N +1/+1 counters are put on the exact source permanent. The three factories are nonflying1/1,2/2 and3/3 Spirits. A living exact source exposes actor-only `endure_choice` and `ChooseEndure { put_counters }` during resolution. The opponent receives no private frame. A departed/reentered source automatically creates the token, without an impossible counter option. Endure0 does nothing. Neither branch targets or opens a mana window.

The owned `Pending` variant carries the source reference, controller, amount and immutable factory. Snapshots/clones retain it without a global table or a new per-object field. Counter/token operations use existing checked transactional paths. A late capacity failure restores the entire suspended choice, object arena, histories, queues, revision and randomness.

Descendant retains its actual optional1W payment, decline and mana-ability window before enduring. The generalized resolving payment stores an optional related stack victim: existing ward/counter requests keep their exact victim; optional controller payment has none and cannot counter another item when declined. Anafenza observes only other nontoken friendly creature deaths, including simultaneous source departure. Vanguard retains both entry and attack triggers; Sinkhole loses one life before enduring and state-based actions wait until the resolving choice completes. Intrinsic flash, first strike, reach, lifelink, menace and flying remain on their full printed bodies. Krumar Initiate and Warden of the Grove remain gated as whole cards because variable amounts and different recipients require further support.

The [manifest](endure-admissions.json) authenticates all eight complete Oracle bodies/constructors, fixed action helpers, exact token recipes and the two gated neighbors at Java492369c7844b8d76479f0393ff9eb6b0b0cf30d7. All32 hash records/32 Java files match both checkout and pin. The [September25,2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf) provide the rules basis.

Canonical comparison to frozen Mobilize1c6531f verifies eight changed/5,254 preserved definitions, three new nonflying Spirit recipes, all78 previous immutable recipes/metadata, all533 prior numeric helper arms, ordinary identities and colors preserved. The admitted fixed grammar consumes every full residual;96 focused compiler negatives cover unknown tails, unsupported compositions and malformed costs. A separate16-case reciprocal Endure/Mobilize regression rejects unsupported combined recipes. The standalone factory-count assertion was corrected80→81 for the combined catalogue.

Eight focused native groups cover every real paid body under both controllers and both outcomes, owned clone/snapshot/restore/stale replay, parent closure, source replacement, real optional payment and decline, simultaneous death filtering, repeated attacks, zero action, life-zero continuation, and late counter/token-entry observer failures. The combined1,534-test native collection, all-target Clippy and formatting pass. The public module contains two methods/36 paid scenarios, including actual opponent Unsummon source departure, both Endure choices, actual Anafenza nontoken deaths, Vanguard entry and attack, Sinkhole life ordering and genuine paid/declined Descendant resolution.

The reproducible legal60-card Standard driver completes100 games twice identically with64,428 decisions/replays,157 private frames and parent-closed continuations, and64,742 stale/wrong-actor/illegal-pass denials. Every admitted body has positive paid-cast, complete-effect, counter and token vectors under both controllers. Exact nonflying1/2/3 Spirit vectors are `[[28,23],[26,24],[5,4]]`; actual1W payments `[8,5]`, intended decline effects `[2,2]`, actual Anafenza other-nontoken deaths `[7,6]`, and Sinkhole life-before-choice witnesses `[6,6]`. Independent integrated measurement starts at20,136 logical bytes and reaches22,776; these exclude spare capacity and allocator overhead. Endure adds no initial logical bytes above exact accepted Mobilize. It is not a Java parity or throughput measurement.

Standalone core verification at b9 passed1,515 native tests, an independent300-test compiler collection and202 fresh-library Python tests (400.750seconds). Those use the prior Convoke baseline, schema69/factory80/fingerprint48c2461a82b19c4ac6759fb0e61c6edba00da5ad4aabda9da8fa15f2ff6bdfd9. They are not represented as full combined306/206-test collections. The combined full collections and final two independent nonauthor reviews are recorded before merge.

```sh
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/generate_catalogue.py --check
python3 -m unittest discover -s scripts
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python
cargo run --release --example standard_endure -- 100
cargo run --release --example standard_mobilize -- 100
```

Use an explicit immutable library path for isolated public checks. Never overwrite a native library mapped by a live process; an environment variable does not override the binding's default constructor path.
