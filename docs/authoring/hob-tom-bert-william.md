# Tom, Bert, and William — isolated author proof

Base `33ff3207b1421992690cf6d2b19097f91b4ce725`. One whole-card candidate; no set acceptance claim. Oracle identity `0ea58cfe-b37c-49a6-a3be-7e60065b8238`; paired provenance pins complete XMage class and reused helpers at `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

## Rules and obligations

| Printed obligation / boundary | Independent outcome |
| --- | --- |
| 3BG legendary 5/5 Troll | Real paid cast and actual Murder; original ownership preserved |
| 1 plus sacrifice another controlled creature | No source/enemy/noncreature/unfunded payment; instant-speed activation, repeats, no tap |
| Draw paid creature's current power, then discard one | Signed current power at cost commitment; zero/negative still discard, empty hand resolves; stolen creature/token; RIP changes destination without losing amount |
| Ability remains independent of source/fuel | Exact saved amount after both change zones, including private Miracle pause |
| Creature-qualified ordinary death trigger | Noncreature artifact death/ability suppression/exile replacement do not trigger; ordinary Staff multiplicity retained |
| Return the exact new graveyard object | Old departure/reentry does not rebind; countered packet has no return; tokens do not return |
| Artifact, no longer creature | CR611.2e simultaneous entry type: Archivist sees artifact, Duo/Authority see no creature; other characteristics and abilities retained |
| Permanent noncopiable effect | Cleanup survives; blink resets; later animation can create a new creature-qualified death |
| Owned/transactional choices | Rejected costs unchanged; private discard raw/index/restore and parent disposal; generic suspended entry form/restore; explicit synthetic skipped entry |

CR references use the cached official 2026-09-25 rules, SHA256 `8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca`: 110.2a,400.7e,603.4,608.2h,611.2e,613.1d,701.21. Return is an ordinary triggered ability, not a replacement. The historical creature condition stays true after triggering. XMage applies the type effect after movement; Rust intentionally applies it as entry per611.2e. Java's full primary helpers were read; StaticFilters controlled-another-creature declaration and CardUtil battlefield lookup were inspected as utility sections.

Existing `SpellView.sacrificed_power` owns the paid amount; existing packet bytes/snapshot accounting includes its boxed i128. New EntryMove/AnimationView bools and GrantedSubtypes optional type mask are inline and included by existing size-based accounting. No new State allocation, catalogue ID or public choice is introduced.

## Wire coordination

`AnimationView.artifact_noncreature: bool` is omitted when false. True means set artifact card type, retaining exceptional instant/sorcery types. `GrantedSubtypes.card_types: Option<u16>` is omitted for ordinary projections and exposes final layer-four kinds only for this form; subtype membership is filtered by CR205.3 taxonomy. Later animation can restore correlated creature subtypes. Existing form rows serialize identically. This is a real new public field requiring central schema167 or later after root166; central schema is unchanged in this leaf. Permanent lifetime uses existing `expires_turn: null` AnimationView semantics. Exact departure removes the form. Root owns generated registrations and full inherited-row schema audit.

## Validation

Author scope uses dedicated persistent `<cache>/hob-tom-bert-william/target` and consistent `CARGO_PROFILE_TEST_OPT_LEVEL=0`; this is no production performance claim. Required integration/default full acceptance remains root-owned. Receipts will record every naturally terminal run, including any failures. No consuming inputs may change while a job is active.

First run `<cache>/hob-tom-bert-william/run-1220e21f3ec9430fa72573fd492847d8` naturally exited101:5/14 passed,9failed; compile36.73s, native0.03s, guard226.110303015s including queue, peak2657329152bytes, all OOM counters0/noorphans. Failed real return exposed use of original BF reference instead of captured exact `source_move.after`; fixed explicit BF→GY binding/token guard. Two fixtures assumed first legal discard was the preexisting Shock and direct staging immediately advanced entry; corrected to explicit discard and `advance_entries`. A separate read-only official-rules audit identified XMage's incorrect preservation of prior card types; corrected artifact-only typing/subtype correlation after natural terminal. Original receipt retained.

Final coherent focused receipt `<cache>/hob-tom-bert-william/run-a0cb475fba584a5cb0fab2c01140bd78` naturally exited0:16 own,11 Supper,10 sacrifice funding,14 Balin,12 mana rollback,6 cast rollback,14 trigger tests,1 shared owned root.84 executions/83 unique; no failures/ignored; compile30.54s, native1.35s, guard63.762528685s including queue, peak2690633728bytes, all OOM counters0/noorphans. The `hob_extra_triggers` filter positively matches function names, despite no matching module filename; an intermediate commentary mistakenly inferred zero matches and was corrected after reading the actual log.

Supplemental driver receipt `<cache>/hob-tom-bert-william/run-58f683e187844aeca1fdfdb86812a2e6` naturally exited1 before Cargo because of an extra bracket in its Python filter list. No executable source changed. Guard42.379857562s including queue, peak26775552bytes, OOM0. Preserved unchanged; subsequent driver was AST-checked before launch.

Corrected supplemental receipt `<cache>/hob-tom-bert-william/run-d404937ee6334f1d95d9dcda35a07ee1` naturally exited0:13 trigger-multiplicity,13 subtype,13 Beorn scenarios; no failures/ignored, no compilation, native0.15s, guard58.091341650s including queue, peak31805440bytes, OOM0/noorphans. Across both successful receipts: **105 unique positive scenarios,123 executions**, native1.50s. Repeated scenarios are counted only once in the unique total. No simulation throughput benchmark was run.

Source verification:5418 definitions/1624 source blobs, Java verified; rustfmt and diff-check pass. Local links regenerated for compilation only. Integration registration is exactly `mod hob_tom_bert_william;` appended to `src/assurance/mod.rs`; this central hunk and generated catalogue/coverage changes are excluded from the leaf. Root must regenerate links, advance the public schema, run final integrated acceptance and obtain two independent nonauthor reviews. No PR/push/merge performed. Synthetic skipped-entry proof is explicitly an injected pipeline outcome; no admitted card is claimed to prevent this return.

Author interval began with isolated checkout creation2026-10-27T02:17:32Z and ended with final evidence capture2026-10-27T02:37:45Z; scope one whole-card candidate plus shared type/quantity mechanics, including guarded queue/build time. This is authoring throughput, not accepted-set or simulation throughput.

## Exact source and receipt pins

Executable/provenance/test source manifest (ledger excluded):

```text
097e8da2bf2b5239ddc061e9b6127c8c053412abcf04d60c8cd9781e68e4a039  src/card_definitions/hob/tom_bert_and_william.rs
0cda5d5d4ef9019ba816254342e5c98be4d000e4231ccaa1cd2046f63f756620  src/card_definitions/hob/tom_bert_and_william.json
1e2d9f9721ddf405baf8b99ea4fc51b76d3ca0e7b583d8f8055c36e0774737ae  src/cards.rs
250e447173f008efb7c911ba25839f03a7c4ea69bfcd89a21addcf574d57e3fe  src/engine_additional_costs.rs
f5399cedfb52f2075968076e15370efb1dba6601b8edc7d18b3637db92b754d5  src/engine_characteristic_layers.rs
cbe61c7f4dbca0b2308ae6ad97c1b73200ea1b12abe69d900635f336e375ceed  src/engine_entries.rs
c243d8038eeba8b263f74856274bb3cf48f73a9c4169a6d3c51d8f4b3684fb26  src/engine_funding_entries.rs
ce08ba32007a170d853e46d4fa3096cd989af796af9e4221a87a3ae2fb6182c2  src/engine_funding_sacrifice_tests.rs
97e9effec498e9206bc36b020c1fc7b5ff98b3e39125dfd606d56d923b2637d1  src/engine_programs.rs
0c627bccfb2b8f20ac74748a0549b1990f68444c36048adfb67b7b0492c7f4bd  src/engine_scalar_x.rs
3c4675392ca48b333e822474ba8c20bad01e5acd722cf71b023a152a26ae86b8  src/engine_stat_values.rs
a4aea1abf03de99b47b59a9397dfc54eb224d6e7edabc442b115336d8487698c  src/engine_stats.rs
de8c7c9d81a7bec35d43c9a3b8941dc8b6ed3e281dd1fd4f10ddca1ba60528b0  src/engine_subtypes.rs
507126eb3b55c292b479b37b9e9a39dace9cc9e0927e299995a7f9c3aca98c87  src/assurance/hob_tom_bert_william.rs
```

Manifest SHA256 `b42fa8173074e0156248d9e5d84cc7d232f3e600ac2d1c4dc2b0a81440f9ced7`.

`run-1220e21f3ec9430fa72573fd492847d8`:

```text
de2bcd459fc34f0598da0ab38edb80ff4fd2825bf45e6dc778978d2c05cd5e4a  0-hob_tom_.log
193829c573ede1175327e833198223e94ab92ded9d1e824f7eb37c12ce963b37  driver.py
f91a7c5864ac83a5d66daa6bb2fcf92c96579828a9c9f06a69d877e417c592a2  outer.log
6578fc89cfde81dd66088acdc4383bbf613b647b6667486c1a86b55a175f7a1f  resource.json
f720341f981f5797ac7b3f4ddcaeb3287184fc04599e314c927bf6621efc6af2  results.json
```

`run-a0cb475fba584a5cb0fab2c01140bd78`:

```text
90f6b88150c9451aa726890749d59fee94a46fda92538d6fed9610ce97bd5d2c  0-hob_tom_.log
a23783aea03c456f799536e6f950cfe0b2210046734ba2648a66ecbb70be6251  1-hob_supper_spiders.log
07dc57a2b4dc16b2c772a1b02cc56e1e7d2b0a31310c3b3a694635149e8083e0  2-hob_sacrifice_funding.log
09cc7e81d745c5ca5d3120122f3cc5aca981582f2e2bbe6be4d1c80f6d315866  3-hob_balin.log
21d1b458e183f44b30ef1d9116b5e69c4f4925daec2c940634898da392e87ea2  4-sos_mana_rollback.log
0225af0efa4a24dd7850e950d083e2fc01c8db26405a515402cb22e7fe10b208  5-cast_rollback.log
7b0488e8c02d2addaa696093b1d5eda0e50b880d51465d63354b75f31f5836a7  6-hob_extra_triggers.log
f71affb1e7b4756d6dd4de34b4c234fdf920f01d41538fa388841b214780660c  7-owned_raw_index_and_restore_at_distinct_decision_roots.log
e5b1ec6f31a2c2840cf1882322ec63195b93cacc55bb0e675fe0c89e5db4d56f  driver.py
e0554f28c09e7698575e37fd4b8af66428d40d3183cba4807ff235d0d7507eb7  outer.log
d0912c7827f8349e9d6adefa22d69de4bde9ac507c08fae100aaed2d6fea35ba  resource.json
5c38bf23fff47a46fbd6f5dbce67539fadb2984390c97ef155eb4275d9a1f65e  results.json
```

`run-58f683e187844aeca1fdfdb86812a2e6`:

```text
6dbcf58611f662c66e67414006bfbd8cd70d211c71201882c4ecab80d0928f9b  driver.py
06ba825c417c7a5477a1818109b699042ff73f1513460ab2d5ad769a5c1320ca  outer.log
9fc185695dfff52378ae3820ec4926edd4b887d692555bd8e84e158bf09f065e  resource.json
```

`run-d404937ee6334f1d95d9dcda35a07ee1`:

```text
21cd0b98a5db22dcc536deb4aa9336aaef99368999245cb1d3e983eecb0896ff  0-hob_trigger_multiplicity.log
14969d382d52bede71a179fde7f69fb9848963527a291faec85511f6a2660358  1-subtype.log
29799a87f80aee542f4ed9f7b96d0de6990efbfc4816cd556aac9b5f53394ab0  2-hob_beorn.log
6d9f727371ebfea7d29e724e9c35080ff4cd653a480ae5ac227967a5d8c18577  driver.py
07c38e80d84e847a811819fada19db77f1f37ec67835d2e75f90f4af752727c8  outer.log
de07519663859222ad79cf8e69c1a81ee742aa6a419d3b2f15669809a48bee3d  resource.json
5b4adbbac290e490db3229641a73d919855114237a0a3ddb653d2b8ef77c0d68  results.json
```

Utility source pin (inspected declaration/function scope described above): `Mage/src/main/java/mage/filter/StaticFilters.java` SHA256 `c37c0bf0ea99fe000c8262cedbb3375bd188ca48267e45eafd0eee2a34d3b105`.

Utility source pin (inspected declaration/function scope described above): `Mage/src/main/java/mage/util/CardUtil.java` SHA256 `02831d3dfba658e0c87bc2111df8e0bc7ec6f56a52305a12ff12809da536fb06`.
