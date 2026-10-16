# Target-dependent spell costs

Issue [#211](https://github.com/altf4/rustmage/issues/211) adds nineteen complete Standard bodies. The integrated release builds on main `e4480721866ef8a14d921fc2c652ffee417e65f4`, including exile permissions and target-library destinations. Historical standalone measurements are identified separately below.

| Cards | Discount qualification | Complete residual effect |
| --- | --- | --- |
| Ajani's Response; Grounded for Life; Luminous Rebuke; Seized from Slumber | Tapped creature | Destroy creature |
| Fate of the Sun-Cryst | Tapped creature | Destroy nonland permanent |
| Magnificent End | Tapped creature | Five damage to creature |
| Quicksand Whirlpool | Tapped creature | Exile creature |
| Ride's End | Tapped permanent | Exile creature or Vehicle |
| Brush Off | Instant or sorcery spell; discount includes blue | Counter spell |
| Out of Air | Creature spell | Counter spell |
| Dark Endurance | Blocking creature | +2/+0 and indestructible this turn |
| Depower | Attacking creature | -4/-0 this turn, then draw |
| Dire Downdraft | Attacking or tapped creature | Target owner chooses library top or bottom |
| Run Behind | Attacking creature | Target owner chooses library top or bottom |
| Uneasy Partings | Attacking nontoken creature | Target owner chooses library top or bottom |
| Grow Extra Arms | Spider | +4/+4 this turn |
| Roadside Blowout | Permanent with mana value one | Bounce opposing creature or Vehicle, then draw |
| Run Over | Mount or Vehicle you control | Owned creature deals its power to opposing creature |
| Swampsnare Trap | Creature with flying | Creature Aura giving -5/-3 |

The [manifest](target-cost-admissions.json) includes the complete Oracle text for each admission and 163 SHA-256 records across 50 pinned Java files. Both the checkout and revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7` were checked. Unknown residual abilities, multiple reduction clauses, variable discounts, unknown qualifications and activation-only reduction clauses remain gated. No Oracle body is shortened to gain admission.

## Cost and choice contract

Immutable `TargetSpellReduction` metadata contains a typed target rule and fixed generic/colored discount. Current battlefield, combat, subtype, mana-value and spell-kind facts determine qualification. One qualifying target is sufficient; multiple qualifying targets do not repeat the discount. Relation restrictions remain independent of the spell's target legality. Changeling supplies creature subtypes, and tokens remain excluded where specified.

Announcement preflight and every target choice require a legal, fundable completion. The borrowed completion query counts each typed target group's remaining capacity and eligible distinct candidates, rather than enumerating target combinations. It does not change state or hypothetically tap a target to invent a discount. Existing mana and mandatory-resource funding checks remain responsible for actual resources and reservations. No action-count cap is introduced.

After the complete target list is chosen, the existing owned `Announcement.total_cost` locks the selected base cost, optional surcharge, static reductions and target reduction before mana abilities or mandatory nonmana payments. Subsequent tapping, sacrifice, target departure or reentry cannot recompute that cost. Resolution still checks each exact target incarnation and legality. This follows [CR 601.2f–h and 118.7a–d](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf): generic reductions affect only generic costs; typed reductions first reduce their own component and transfer excess to generic, without reducing another color. Costs stop at zero. Printed mana value and color are unchanged.

Hand casts, flashback, hybrid choices, offspring declarations and forage resource choices reuse the shared lock and funding paths. Exile permissions reuse the same staged funding path, exercised by public acceptance and two native integration test groups. The direct single-target shortcut explicitly requires the source to be in hand. Other origins retain their explicit announcement actions and permissions.

No mutable state, public actions or observation fields are added. The integrated schema remains **58**, C ABI 1 and control protocol 2. Clone, snapshot, restore and stale-index behavior retain the existing contract; private pending choices stay private. Action overflow restores targets, locked cost, resources, generations, stack, RNG and revision together.

## Validation and immutable catalogue

The standalone catalogue supports **1,487 / 5,164** Standard identities, plus six regression cards, with **71** token factories and one emblem. Exactly nineteen of 5,255 definition rows change; the other 5,236 rows, all 5,183 ordinary card IDs and all prior recipes/metadata remain unchanged. All 442 prior numeric helper arms are preserved, with twenty additions: nineteen reduction rules and Swampsnare Trap's Aura helper. The fingerprint is `a62ff78b04c186b6c8ef71341d43ceea607f2272a3f6a9e89a32bb81318e310c`; a compiler regression verifies that changing only a discount changes the fingerprint.

Twelve native test groups exercise every complete body in both seats, discounted and full payments, colored requirements, nonqualifying targets, actual payment-induced tapping after a full-price lock, multi-group completion, source/target reincarnations, wrong actors, private observations, printed values, static reducers, alternative selected bases, snapshot replay, stale indices and early/late capacity rollback. Synthetic target-plan and selected-cost fixtures isolate reusable boundaries without claiming new Oracle admissions.

The core passes **1,318 Rust tests** (1,306 unit and twelve integration), **159 fresh-release Python tests**, formatting and all-target Clippy with warnings denied. All **251 compiler/data tests** pass (302.725 seconds), along with generated-catalogue verification and whitespace checks. These counts describe the historical standalone core before integration and the separate public acceptance files.

Initialized logical state remains **19,912 bytes**, unchanged from the parent. A 1,000-game smoke benchmark completed 571,049 decisions at about 243,000 decisions/s, with 163 ns clone and 189 ns restore under concurrent host load. This includes native policy and observation construction, excludes model inference, tensors and JSON, excludes capacity/allocator overhead, and is not an XMage comparison.

## Integrated release evidence

Against main `e448072`, support advances from 1,495 to **1,514 / 5,164** Standard cards; 3,650 remain gated. There are 1,520 supported bodies including six regression cards, 72 token factories and one emblem. Exactly nineteen of 5,256 definition rows change; 5,237 rows, all prior factory metadata, ordinary IDs and color masks remain unchanged. All 446 existing numeric helper arms are preserved, with twenty additions (466 total). The integrated fingerprint is `a917b33e39153e5c0102fa9900e92b00d95116213c7a25ca91f2f528d547ebd8`.

The combined release passes **1,347 Rust tests** (1,335 unit and twelve integration), **257 compiler/data tests** (285.459 seconds), and **164 fresh-release Python tests** (101.900 seconds). Formatting, all-target Clippy with warnings denied, separate acceptance-example Clippy, generated-catalogue verification and whitespace checks pass. Public acceptance covers 48 controlled cases: all nineteen hand-cast bodies and five actual Alania exile-cast bodies in both seats. Native integration additionally covers exact exile permission consumption, forbidden shortcut actions, ordinary exile-origin metadata, capacity rollback and replay.

`cargo run --release --example standard_target_costs -- 100` reproduces **60,950 decisions and exact snapshot replays**, with 1,115 locked-cost checks, 144 private choices and both controllers exercised (97/90 paid casts). In driver card order, full-price/discounted payment counts are `[[2,11],[1,3],[5,1],[2,9],[2,9],[5,8],[3,2],[1,13],[0,8],[0,14],[0,3],[0,12],[6,12],[7,5],[4,9],[1,4],[0,13],[0,5],[3,4]]`; the driver's exact card order is defined in `examples/standard_target_costs.rs` (the table groups related cards). Per-card effect witnesses are `[13,4,6,11,11,13,5,14,8,14,3,11,18,12,12,5,12,5,7]`. Adjacent library-destination, exile-play, filtered-selection and forage workloads retain their prior 71,875 / 60,677 / 71,080 / 65,729 decision and replay counts.

Initialized logical state remains **19,944 bytes**, unchanged from integrated main; the target-cost workload reaches 20,956 bytes. No mutable fields or schema changes are needed. A 1,000-game smoke run completed 571,049 decisions at approximately 394,000 decisions/s, with 105 ns clone and 113 ns restore under concurrent host load. These are workload observations, including native policy and observation construction and excluding model inference, tensors, JSON, allocator/capacity overhead and external snapshots; they are not an XMage performance comparison.
