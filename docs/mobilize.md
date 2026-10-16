# Mobilize: attacking entry and exact delayed sacrifices

The combined integration includes Convoke and executes **1,656 / 5,164 Standard identities**, schema **69**, with78 immutable token factories. Fingerprint: `2832c7d3b10b18fbfba527d035b862053e17ef028334526101b249980cc98596`. Standalone measurements below retain their exact original base.

Issue [235](https://github.com/altf4/rustmage/issues/235). Eight complete bodies are executable: Avenger of the Fallen; Dalkovan Packbeasts; Dragonback Lancer; Nightblade Brigade; Reigning Victor; Shock Brigade; Stadium Headliner; Zurgo's Vanguard. This branch builds on frozen saddle9539958 and has 1,646 Standard bodies, 77 immutable token factories and schema68. Broad compiler and Python collections are recorded below when completed.

Mobilize creates actual red1/1Warriors tapped and attacking. Their batch enters simultaneously: all newcomers are installed with combat membership before entry observers inspect the battlefield. They are not declared attackers and cause no attack/declaration/raid history or tap events. The ordinary token helper retains its old entry behavior. Avenger's exact creature-card graveyard count is evaluated at trigger resolution before any newcomers exist; zero creates neither tokens nor an empty delayed packet. Vanguard's creature-count characteristic base power applies in every zone and immediately sees actual new creatures. Ordinary printed mana values/colors and selected two-brid payments are retained.

CR508.4 permits a separate defending-player or defending-planeswalker choice for each entering attacker. The initial scout's fixed-opponent assumption was incorrect: the accepted engine already supports Ajani, Caller of the Pride and Professor Dellian Fel. When there is more than one defender, the choosing actor receives `mobilize_choice: {source, token, amount, defenders}` and `ChooseMobilizeDefender { defender }` for every currently legal opponent or exact planeswalker incarnation. These are non-target choices with no ward. Planned choices remain actor-private; all tokens enter together after the last choice. With only the opponent available there is no artificial choice window.

Every nonempty creation resolution owns one public pending `DelayedSacrificeView {id, source, source_card, controller, tokens}`. `Observation.delayed_sacrifices` exposes the bounded future obligations to both viewers. Source lifetime is irrelevant. At the next beginning-of-end-step event, that batch is consumed once and becomes an ordinary APNAP-ordered stack packet of kind `DelayedSacrifice`; its `SpellView.delayed_sacrifice` owns the same exact batch identity and token references. The packet gets its own ordinary stack identity, accepts responses/countering, and resolving it sacrifices all still-controlled surviving exact incarnations as one genuine batch. Departed/ceased/reused slots are never rebound. Countering the packet leaves surviving tokens and never schedules it again.

The state uses optional owned boxes for the pending creation plan and delayed batch registry, and one optional boxed context on stack packets. There is no stored per-object mobilize flag or global mutable table. Snapshots and clones own planned defenders, attack membership, delayed references and future queue identity. The measured logical sizes and all-section pinned-Java manifest are recorded below.

The four other mobilize cards remain gated as whole bodies: Bone-Cairn Butcher, Venerated Stormsinger, Voice of Victory, Zurgo, Thunder's Decree. Their unsupported recipient, death-observer, casting-restriction or sacrifice-prohibition residuals are not discarded. Unknown amounts, duplicate declarations, unsupported printed costs and mixed alternative/additional cost declarations also remain gated.

Primary rules: [Comprehensive Rules, September25,2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf), CR702.181,508.4,603.7 and400.7.

## Complete admission and independent acceptance

The [all-section manifest](mobilize-admissions.json) records all eight complete Oracle bodies, the four explicitly gated neighbors and the actual constructor/payment/token/delayed-target helpers at the pinned Java revision. All **98 hash records / 68 unique Java files** match both checkout bytes and the pinned commit. No missing residual is replaced by a partial instruction body.

Canonical comparison to exact parent `9539958e909267260b9dd9f7532aef891be618cb` proves **eight changed / 5,252 unchanged definitions**, one added red Warrior factory, all **76 prior immutable recipes and metadata** retained, all **5,183 ordinary card identities** and every prior color preserved. All **530 prior numeric helper arms** are retained, with one new scaling helper. Token/emblem numeric factory positions are catalogue scoped: normalization compares referenced immutable recipe identities. Replays compare the same catalogue fingerprint, `9ed00d270ce2d757442fd8222e0349ff3c1d5b16e71502f44fdb95fad6d6097b` on this standalone branch.

Fourteen native groups pass, including all eight actually paid bodies and all Victor equivalents in both seats; real player/two-planeswalker combat damage; menace with two actual blockers; simultaneous entry and historical counts; source departure and zero/live dynamic counts; exact ceased-token slot reuse; multiple independently ordered delayed batches; countered creation and sacrifice packets; source sacrifice and live Stadium damage; full Surveil and all-zone Vanguard characteristics; and late registration, second queue, second sacrifice and partial-entry overflow with complete action rollback. The full native collection passes **1,512 tests** (1,500 unit plus twelve integration). All-target Clippy, formatting and generated catalogue checks pass.

The public module has **four methods / 38 paid scenarios**: every full body in both seats, every Victor equivalent in both seats, actual paid planeswalkers with three separate defender choices, actual Stadium mana/source sacrifice and damage, and an actual opponent Shock response killing the source before Mobilize resolves. Every action checks owned clone, snapshot/restore and stale-index rejection. Private plans reject wrong actors, own-player defenders and illegal passes without mutation; plan, pending delayed registry and ordinary delayed stack branches survive closing their immediate parent. Both viewers observe the same committed attackers and exact delayed obligation, while planned defenders remain private.

The reproducible public Standard driver completes **100 legal 60-card Standard games**, twice with **63,181 identical decisions/replays/stale rejections**, **136 created Warriors** and **12 paid Stadium activations**. All eight cards have positive paid-cast, complete-effect and delayed-sacrifice vectors under both controllers: the first four have `[7,6]`, the remaining four `[6,6]`. Its ledger checks actual tapped-and-attacking entry, no new declaration history, simultaneous nonland-entry count, live Vanguard power, full Nightblade/Victor residuals, source-independent sacrifice, real deaths and finished engine outcomes.

The measured standalone initial logical state is **20,136 bytes**, versus the exact Saddle parent's **20,088**: **+48 bytes of actual enclosing layout**. This differs from merely summing raw pointer widths. The state has two sparse optional owned pointers and each stack packet has one optional boxed delayed context; active storage grows with actual planned defenders and exact delayed token references. Standard100 reaches **22,508 bytes**. A contended 1,000-game native smoke run measures about **268,112 decisions/s**, **210.55 ns clone** and **210.44 ns restore**, including the limited-catalogue smoke policy and observation construction, with no model, tensors or JSON. This is not an XMage throughput/parity comparison, and allocator overhead/spare capacity are excluded from logical byte counts.

Four compiler methods cover every admission and gated neighbor, every Victor equivalent, bounded fixed and exact dynamic counts, duplicate/unknown declarations, unsupported tails, malformed/overflow/variable printed costs and reciprocal alternative/additional/history cost compositions. Valid widened mana totals remain admitted; an individual component overflowing u32 is rejected. Historical tests now explicitly admit the two newly complete Stadium/Victor bodies and include the added immutable Warrior factory.

```sh
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/generate_catalogue.py --check
python3 -m unittest discover -s scripts
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python
cargo run --release --example standard_mobilize -- 100
cargo run --release --bin benchmark -- 1000
```

Use a HOME-backed target and an explicitly supplied immutable library path for isolated public checks. An environment variable does not change `Game`'s default library selection, and a library mapped by a live process must not be overwritten.

Standalone broad public validation passes a **200-test fresh-library collection in 255.749 seconds**. That collection includes the first two Mobilize methods; the later added Stadium and source-departure methods are independently included in the final **four-method / 38-scenario** focused run, passing in **4.824 seconds** against the same immutable library. This verifies all 202 currently present Python methods across the broad and final focused runs without representing the earlier collection as a 202-test run. The complete 298-test compiler rerun is still in progress at this checkpoint after correcting only three stale pre-admission fixtures (two formerly gated names and the added factory count).
