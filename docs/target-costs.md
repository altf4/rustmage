# Target-dependent spell costs

Issue [#211](https://github.com/altf4/rustmage/issues/211) adds nineteen complete Standard bodies on the stacked target-library core `a46509bca97267c596fe9b7442784573de4e09d6`. The public acceptance files are authored separately against the integrated main branch; the counts below describe this standalone core.

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

Hand casts, flashback, hybrid choices, offspring declarations and forage resource choices reuse the shared lock and funding paths. The standalone parent lacks exile permissions; their integration is exercised by the separate public acceptance and root-owned exile regression tests. The direct single-target shortcut explicitly requires the source to be in hand. Other origins retain their explicit announcement actions and permissions.

No mutable state, public actions or observation fields are added. The standalone schema remains **57**, C ABI 1 and control protocol 2. Clone, snapshot, restore and stale-index behavior retain the existing contract; private pending choices stay private. Action overflow restores targets, locked cost, resources, generations, stack, RNG and revision together.

## Validation and immutable catalogue

The standalone catalogue supports **1,487 / 5,164** Standard identities, plus six regression cards, with **71** token factories and one emblem. Exactly nineteen of 5,255 definition rows change; the other 5,236 rows, all 5,183 ordinary card IDs and all prior recipes/metadata remain unchanged. All 442 prior numeric helper arms are preserved, with twenty additions: nineteen reduction rules and Swampsnare Trap's Aura helper. The fingerprint is `a62ff78b04c186b6c8ef71341d43ceea607f2272a3f6a9e89a32bb81318e310c`; a compiler regression verifies that changing only a discount changes the fingerprint.

Twelve native test groups exercise every complete body in both seats, discounted and full payments, colored requirements, nonqualifying targets, actual payment-induced tapping after a full-price lock, multi-group completion, source/target reincarnations, wrong actors, private observations, printed values, static reducers, alternative selected bases, snapshot replay, stale indices and early/late capacity rollback. Synthetic target-plan and selected-cost fixtures isolate reusable boundaries without claiming new Oracle admissions.

The core passes **1,318 Rust tests** (1,306 unit and twelve integration), **159 fresh-release Python tests**, formatting and all-target Clippy with warnings denied. All **251 compiler/data tests** pass (302.725 seconds), along with generated-catalogue verification and whitespace checks. Public Python and reproducible Standard-100 witnesses are separate acceptance changes, not included in this core commit.

Initialized logical state remains **19,912 bytes**, unchanged from the parent. A 1,000-game smoke benchmark completed 571,049 decisions at about 243,000 decisions/s, with 163 ns clone and 189 ns restore under concurrent host load. This includes native policy and observation construction, excludes model inference, tensors and JSON, excludes capacity/allocator overhead, and is not an XMage comparison.
