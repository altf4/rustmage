# Bilbo, Luckwearer // Burglar's Plot

Base1152072fc2625f4c46d57d9dc74f3a1907559a3f. Whole-card author candidate; no set acceptance claim. Both faces use Oracle identity a73d9109-36e0-40ea-ab06-5c5f7eb4ac54. Full XMage BilboLuckwearer class/custom target and full primary helpers were read at492369c7844b8d76479f0393ff9eb6b0b0cf30d7; paired records pin all source hashes. StaticFilters nonland declaration is a utility scope.

| Obligation / shared boundary | Outcome proof |
| --- | --- |
| 1U legendary 1/1 Halfling Rogue, can't be blocked | Actual paid front cast, summoning sickness rejection, real attack/block rejection and player combat damage |
| Mandatory combat-player draw then discard | Zero/noncombat/creature damage excludes; stolen controller/source departure/Staff multiplier/suppression; private Miracle and discard raw/index/restore; failed late draw action rolls back |
| 4U Sorcery Adventure with exact two distinct nonland targets sharing card type | Paid face-specific stack cost/name/MV; no pair/no mana/land/subtype/supertype-only/isolated type/duplicate/incomplete targets rejected |
| Any current card type, including multitype overlap | Artifact/creature intersection and current type changes; keyword/controller does not define type |
| Entire fixed exchange or nothing | One/both illegal, hexproof, changed type, same controller, current resolution controllers; common timestamp installed before one controller projection |
| Duration/source/incarnation | Cleanup and later type changes do not undo; old temporary control cannot undo; later fixed control supersedes; blink clears only departed exact effect |
| Attachments/control layers | Equipment stays independently controlled; exchanging Confiscate Auras simultaneously changes their controlled recipients |
| Adventure lifecycle | Counter/all-illegal no permission; resolving failed exchange exiles; paid front from exile only; exact exile departure clears right; real paid spell copy/retarget cannot create another physical permission |
| Owned relational quantities | Exact target predeparture kinds LKI, private wire omission, snapshot/restore and parent disposal; existing affected controller-LKI and cast/mana rollback regressions |

Rules: cached official2026-09-25 CR SHA2568d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca, especially400.7,608.2b/h,613.1b/613.7 and701.12a/b. [Official WAR release notes, Role Reversal](https://magic.wizards.com/en/news/feature/war-spark-release-notes-2019-04-19) confirm indefinite duration, one-illegal no exchange, same-controller no effect, attached objects' independent control, and card type versus supertype. These are applied to the identical exchange wording with Bilbo's additional nonland restriction.

Reviewer attention: stale relational target qualification mirrors the already accepted `DifferentControllers` contract: a departed target contributes its exact last card types to the survivor's relation, but cannot itself be affected. Current same-incarnation type changes use current types. The [official BLB release notes, Run Away Together, pages44–45](https://media.wizards.com/2024/downloads/BLB_Release_Notes_DgtOCtpFAl/EN_BLB%20Release%20Notes%2020240611.pdf) expressly permit information about an illegal target solely for its partner's legality, using last-known information after departure. The card-type relation applies that same relational rule; this is an inference supported by CR608.2b/h and the established controller relation, rather than a card-specific Bilbo ruling. WAR Role Reversal notes do not explicitly distinguish Adventure's exile outcome. No contrary ruling was identified; independent reviewers should check this inference and the resolving-one-illegal/all-illegal assertions.

No public wire fields/vocabulary change: `SpellView.target_kinds_lki` is private and serde-skipped; optional boxed exact-reference/kinds rows are allocated only when a SharedCardType target actually leaves. Owned bytes include the Vec and rows; ordinary inline packet size uses existing accounting. Snapshots own the field. Copies/retargets keep exact binding facts and unused trigger templates clear them. Existing target-controller history remains independent.

## Integration/dependencies

Burglar's Plot exclusively reserved5419. Exact Eagles5418 token metadata/pair from `<temp>/rustmage-hob-eagles-coming` is an excluded local build dependency, not Bilbo-authored work. Central catalogue changes are excluded from the leaf. Apply `hob-bilbo-luckwearer-metadata.patch` after reviewed Eagles5418 integration; the patch contains only Bilbo front metadata/aliases and own5419 Adventure row/alias/color. Regenerate links. Local test registration is exactly `mod hob_bilbo_luckwearer;` in assurance/mod.rs and is excluded from commit. Root owns central schema166+ coordination/full integrated acceptance/two nonauthor reviews. No PR/push/merge.

## Validation

Dedicated persistent `<cache>/hob-bilbo-luckwearer/target`, consistent author-only TEST_OPT_LEVEL0, guarded one job/thread shared6GiB. Persistent drivers are syntax-checked before launch; entire guard output redirected from launch. Inputs freeze until each same job naturally terminates. Full release acceptance remains root-owned. Receipts and exact pins follow after terminal proof.

First receipt `<cache>/hob-bilbo-luckwearer/run-e870611897614ad9be4cda172ce966b6` naturally exited101,17/18 own scenarios passed. Only failure was a test fixture selecting hand-only AnnounceCast for the Adventure exile creature route; corrected to the established AnnounceExile action. No engine repair was needed. Guard195.647819263s including queue, peak2697347072bytes, OOM0/noorphans.

Second receipt `<cache>/hob-bilbo-luckwearer/run-4425bc9b733149fcb8687c25490d4694` naturally exited0:19 own plus affected families,88 positive executions (deduplication recorded below), no failed/ignored. Guard425.101635519s including queue, peak2666192896bytes/OOM0. Final proof refinements changed tests only: physically artifact Equipment animated into creature then departing/reentering distinguishes frozen current kinds from printed/new-incarnation kinds; actual targeted Treasure mana sacrifice exercises the owned enclosing announcement's LKI and failed payment boundary. All source inputs stayed fixed through natural terminal before those refinements.

Final coherent receipt `<cache>/hob-bilbo-luckwearer/run-a7df15cbdb034d9786b7ebf2c310d6fc` naturally exited0: **89 unique positive scenarios/89 executions**,20 own,1 Silverquill control-Aura,5 actual spell-copy,14 storied/control,15 inherited Adventure,15 Gandalf/Adventure,6 cast rollback,12 mana rollback,1 shared owned root. No failed/ignored/timeouts/orphans. Compile34.03s, native2.36s, guard37.613100611s, peak2629177344bytes, configured6442450944bytes, all OOM counters0. Same persistent target/profile throughout all runs. Prior compile times were41.53s and33.67s respectively; guard intervals include shared-slot queue and are not simulation timings.

Final checks:5420 paired definitions/1628 source blobs, Java verified; rustfmt and diff checks pass; reverse-checking the exact owned metadata patch against the compiled local metadata succeeds. The full local metadata is a compiled excluded dependency combination, not a committed integration manifest. No full validator or simulation benchmark was run in this author leaf; root owns integrated default-profile acceptance.

Author interval2026-10-27T02:42:09Z through2026-10-27T03:13:38Z; one whole-card/two-face candidate plus shared relational/atomic-control core, including queued tests. Complete supported-set/full-pool acceptance remains open until root integration and required nonauthor reviews.

## Exact source, dependency and receipt pins

Owned source/provenance/test/metadata-patch manifest (ledger excluded):

```text
28cd41d80ce68e1681d2931518ae67404b7bf54f658e65d0e0e05b353eec7a92  src/cards.rs
36ea20a88e75089410065ff21423943a52f61fadfe07325eeb1643d46189f3aa  src/engine.rs
dd654ca6e3fdecd4b098a22c77e230bd2c19d563864fd20e934d4aa9daab725e  src/engine_activations.rs
56d111b411b1a391cb628773d8b4074070274f3be7dc09c6ba16412d92fcacda  src/engine_continuous_control.rs
05575c945957df35cda1a1759a73893602ab1e3e8840f48583f4858c0850e0ef  src/engine_mobilize.rs
5ba53bb1b82db7c26215ff34cbda2f0bb1d35149c700d845767d1e3ed2030be2  src/engine_paradigm.rs
b160714774e5086c43bf073e00658714230a375e99e92b9907666bb59d97a6d2  src/engine_programs.rs
45b71a8645df5f185098f065309a09fe9fb13b9ac37010d8681bc5129fa7e525  src/engine_stack_copies.rs
46d9ff5699789e365c74b795a1ea7ced0c4c471bb614ff64a68cc686a3a4fb52  src/engine_targeting.rs
ceb0eafeeef8c38b7cc3702ff494f32091cd6895d3ddcd1c21cf0fb6fb6089a0  src/engine_warp.rs
6e85f485466dc202c7c206e1dca2764818e748afe453e00007541164b650c6f7  src/engine_zone_effects.rs
07b129d7c0dec47cdcf98913812fe9b5de9c1837e330048656c555337e099105  src/card_definitions/hob/bilbo_luckwearer_burglar_s_plot.rs
06f4827b8dde840e94441cd06f87dc524685ba2b6f3f5970d7efbb27084179a0  src/card_definitions/hob/bilbo_luckwearer_burglar_s_plot.json
46f38706aee9cacac98db56766ed9ca8542e28054ba68e5fdfda13e1aa5de5d7  src/card_definitions/synthetic/burglar_s_plot.rs
57491da8ad9d34e3713a897efcd9c2c16ab1edcaa85f44a7eb12f830f7a17462  src/card_definitions/synthetic/burglar_s_plot.json
a029fbece4994d1206464fc54605184c80b1fbc216e81d42bc13d275d2facb45  src/assurance/hob_bilbo_luckwearer.rs
d01e45741a37171978062aee2b4f39b1140e16596fbd023780aca36498c19eca  docs/authoring/hob-bilbo-luckwearer-metadata.patch
```

Manifest SHA256 `72a33fcaee3e60a55c013f8b6e5c6b39730cbbb9acead8fa731b2f5fea9a1411`.

Excluded Eagles token dependency is from final leaf `cb660b00eb11d13fd8358006e4be91832da43337`; exact row5418/white-color/alias/token recipe and these paired files were copied before freezing.

```text
494112fde43f2e0812a17ee48ae9959896fd658d17a76d54f213e66b5f5e92f7  src/card_definitions/synthetic/token_hob_bird_soldier.rs
bf5d3db64485f7d92332ebfb80ebb00c784dd95c03251b9700c8e8e20d790cc1  src/card_definitions/synthetic/token_hob_bird_soldier.json
81fb74583f1480c7ebccdbab5d96a2f3bd1288ed6a5daae4e2b8a54e9dad8ba3  data/catalogue-identities.json
```

Canonical excluded token-row/color/aliases/recipe dependency SHA256 `b45e41d038e77d9e58a86605fc4c49538651ec305e7fea0c02869d9245f027e3`. The full metadata hash above includes owned Bilbo changes and the excluded5418 dependency; the committed patch contains only Bilbo changes.

`run-e870611897614ad9be4cda172ce966b6`:

```text
e825a14f781964fcec25f786e10f1ce0c9aa76984bfa209c3a995234bc869f53  0-hob_luckwearer_.log
bb794363123c4cb79fc276624f91931a44bd5530e3ba722b7209626aacdc068c  driver.py
015aeb848268ed5d78cd1b7397ffbaf5ab55e38007c4ed838620755a99df4986  outer.log
148fff0cc29ae7061e2dab402789e5279ad82b98ee856afb97d3a4a1bd96319f  resource.json
6fd1c357f9ecfcaee8518c4cf0d348319ce72115b80607c326c56f485e4e7109  results.json
```

`run-4425bc9b733149fcb8687c25490d4694`:

```text
4b709ef13b1001f251997e5d317de9923b2ce7c64c9dd9ed81b388f80b059045  0-hob_luckwearer_.log
6180a0f5f55d7ddd0f3d2bb52dcaa08e2a81c4aeb6b70568630c2004eb3f51c7  1-sos_silverquill_control_aura.log
6699f1a9104ff035663bf1e8751639a0afe982abe291ab6c618715998c1d3c74  2-sos_sparks.log
cbc0b1478cb2000b2b0ececbe8e4e54aad984a6d9f4d504522a5d6f5ee635041  3-hob_storied.log
2a0dd411b16be02ad9f3e4e95d59571ccd48137ab79826da84385f1be46553a2  4-hob_inherited_adventures.log
5a0336728d2013725f7212540be3012cbb505d1f5c5dbcaf45a7d90539b6540c  5-hob_gandalf_flameshape.log
860aa6bbdbbc51785b42ba16dd1b0b802c815d83a6126fbc9d01885e093b03a1  6-cast_rollback.log
12ed9804e8eec6896de979ec90fce01e38fad7234639589ded489f9a52d58c26  7-sos_mana_rollback.log
79b4ac073f37d9847a4cc79792e82fb759a7e1e842fd0d7bdea5d81d46468f92  8-owned_raw_index_and_restore_at_distinct_decision_roots.log
8f85b4c6361db50f373041586f70a91b879e0fb68c17eb178750430b432fcf05  driver.py
67847d228eadf4bd8d3b54d2339a7b4e2a1f7b253956683b1eb742ecc9a417c4  outer.log
f66e5a301572047f94426808e53fa10ed14e2da222fa8ee6e4ff9a1d985bcfb8  resource.json
79d933a10ddd40354f501d2c02f4545372546ba0a6aafa925bc2536bc3a536a8  results.json
```

`run-a7df15cbdb034d9786b7ebf2c310d6fc`:

```text
dfcc3da9972b4d51ef408706fef9a68fc1aa0af11221a3d746523d45c82afe0a  0-hob_luckwearer_.log
64f53867a5f10f1d0b6b6d9568f0ad6d753347e5a52086cc1659df0d7a6ca4a2  1-sos_silverquill_control_aura.log
4ea1980764e8ecb2f065695f88e4bdb8066ebbbdcba563999ce841278c108130  2-sos_sparks.log
8a7d94ed595d125ad55c485ee040e7d3d676c6d74f2441d73ab10332fdb90d79  3-hob_storied.log
c6de503c3bded67363302bf9e1b6b07fd9c2734d40c3beaa8de6686b98ba4d00  4-hob_inherited_adventures.log
0ff66a401674101eb97eae33e5b85f69294c524bb19462f431958ac6280dd2b3  5-hob_gandalf_flameshape.log
f3a26a5330bee69fab3cda7558115e39554f7403e714b73545e36e8bd0eb8310  6-cast_rollback.log
1cdfb13435cedf21db7027594ad0510d53e2c7d7d76b1a7e4b42115247ce86d4  7-sos_mana_rollback.log
21638cbcde3d7fd395e7a6ecfed7d8412fbb05045f2257ed058586311e51c02a  8-owned_raw_index_and_restore_at_distinct_decision_roots.log
8f85b4c6361db50f373041586f70a91b879e0fb68c17eb178750430b432fcf05  driver.py
a747330efb0dd436caa4f4d212ef7cf3546a74f2d5cd6f9a7acaa24bffc21d87  outer.log
3f074e521eb7ed4f65322961a4a301ee903a132571b0491411a0461cf6d082da  resource.json
c277ee19f66fd10d0a8630a48229b5ab15ba19b4208c53715ac1c11d81523032  results.json
```

Additional source audit pins (same XMage revision): full FilterNonlandPermanent and CardWithSpellOption helpers, and StaticFilters nonland declaration utility scope:

```text
451cffb8d2af71777225c79e09772339e74603483d0f32ad2cc65c88e3a73cb9  Mage/src/main/java/mage/filter/common/FilterNonlandPermanent.java
370a50d75c092a9cb5b0f3111af32fc2895479fb3f14f656267685bc9d09dea0  Mage/src/main/java/mage/cards/CardWithSpellOption.java
c37c0bf0ea99fe000c8262cedbb3375bd188ca48267e45eafd0eee2a34d3b105  Mage/src/main/java/mage/filter/StaticFilters.java
```

## Post-review provenance correction

All three original resource reports were confirmed naturally terminal before editing. Full `Mage/src/main/java/mage/target/TargetPermanent.java` was inspected at pinned XMage revision492369c7844b8d76479f0393ff9eb6b0b0cf30d7. SHA256 `1287b2516023fe0a65202c59b9c8cfde8144239d0fbaf8b2a8da636b55a4d2fb` matches the reviewer-supplied pin. Both paired records now explicitly include this directly inherited helper. Its constructor carries exact min/max, battlefield zone and real targeting; `canTarget` checks source spell/source permanent protections before the nonland filter. No additional rules discrepancy was found.

This follow-up changes only the two provenance records and this ledger; card/engine/test Rust and metadata patch are byte-identical to3624b970. Link/source checks were refreshed:5420 definitions,1628 deduplicated source blobs, Java verified, catalogue version `252c7f1068898dfd666a7d1e861001df9856d155263ef4c9b92422d5092658f8`; diff-check passes. Native rules evidence remains the sealed89-scenario receipt above, whose compiled catalogue version was `4d2adf8a12acb3695cd8d93c05dd2ea788966d4837a6c33abaf961bbd5c83ca1`. No new native execution is claimed for the provenance-derived digest; root regeneration/full integrated acceptance remains required. Earlier source/receipt manifests remain historical exact pins rather than being silently rewritten.

Current owned source/provenance/test/metadata-patch manifest (ledger excluded):

```text
28cd41d80ce68e1681d2931518ae67404b7bf54f658e65d0e0e05b353eec7a92  src/cards.rs
36ea20a88e75089410065ff21423943a52f61fadfe07325eeb1643d46189f3aa  src/engine.rs
dd654ca6e3fdecd4b098a22c77e230bd2c19d563864fd20e934d4aa9daab725e  src/engine_activations.rs
56d111b411b1a391cb628773d8b4074070274f3be7dc09c6ba16412d92fcacda  src/engine_continuous_control.rs
05575c945957df35cda1a1759a73893602ab1e3e8840f48583f4858c0850e0ef  src/engine_mobilize.rs
5ba53bb1b82db7c26215ff34cbda2f0bb1d35149c700d845767d1e3ed2030be2  src/engine_paradigm.rs
b160714774e5086c43bf073e00658714230a375e99e92b9907666bb59d97a6d2  src/engine_programs.rs
45b71a8645df5f185098f065309a09fe9fb13b9ac37010d8681bc5129fa7e525  src/engine_stack_copies.rs
46d9ff5699789e365c74b795a1ea7ced0c4c471bb614ff64a68cc686a3a4fb52  src/engine_targeting.rs
ceb0eafeeef8c38b7cc3702ff494f32091cd6895d3ddcd1c21cf0fb6fb6089a0  src/engine_warp.rs
6e85f485466dc202c7c206e1dca2764818e748afe453e00007541164b650c6f7  src/engine_zone_effects.rs
07b129d7c0dec47cdcf98913812fe9b5de9c1837e330048656c555337e099105  src/card_definitions/hob/bilbo_luckwearer_burglar_s_plot.rs
315a6ee60be7e3b4527daf70e596d5f6bc3c431f62b654cffd82734054c9e1ae  src/card_definitions/hob/bilbo_luckwearer_burglar_s_plot.json
46f38706aee9cacac98db56766ed9ca8542e28054ba68e5fdfda13e1aa5de5d7  src/card_definitions/synthetic/burglar_s_plot.rs
eb02a8ad4e0619f1160ec7d859888b3f6138ded0b7dbe74db740be3bb0298f06  src/card_definitions/synthetic/burglar_s_plot.json
a029fbece4994d1206464fc54605184c80b1fbc216e81d42bc13d275d2facb45  src/assurance/hob_bilbo_luckwearer.rs
d01e45741a37171978062aee2b4f39b1140e16596fbd023780aca36498c19eca  docs/authoring/hob-bilbo-luckwearer-metadata.patch
```

Current manifest SHA256 `00cceeafd0caf90ff674212668a32adaba8b4b9907579c4ae1a0924e55fe3da7`.
