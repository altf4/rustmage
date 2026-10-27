# Thranduil's Decree

Isolated base `ccfff430bbb261906f5138f54de924a1fd10e664`; complete-card candidate only, pending two independent nonauthor reviews and root integrated release acceptance. No PR/push/merge. Root regenerates module/catalogue/coverage artifacts.

Full Oracle: identityd7cba934-02ad-4677-bb4d-50808b01b4f9, catalogue4558,4UU instant. Counter target spell; only a permanent spell successfully countered this way is exiled instead of going to its owner's graveyard, then the Decree controller may cast that exact card without paying its mana cost while it remains exiled. A counterable spell is not required for legal targeting. Spells include actual copies, but a copied spell supplies no exiled card permission.

Read complete pinned XMage ThranduilsDecree.java and its local effect, PlayFromNotOwnHandZoneTargetEffect, AsThoughEffectImpl, TargetSpell and FixedTarget at492369c7844b8d76479f0393ff9eb6b0b0cf30d7. Core SpellStack.counter and Spell.counter call paths reviewed and pinned (unrelated stack/card utility methods are not asserted as card-specific proof). Rust retains normal timing, explicit free alternative, additional costs and exact incarnation; it permits only casts, so a land face is never a land play through this permission even though Java uses its generic play helper.

Official20260925 CR cached `<cache>/rules/MagicCompRules-20260925.txt`, SHA2568d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca. CR701.6 countering and no cost refunds; CR118.9 alternative cost and additional costs; CR601.3e chosen spell-face qualities; CR400.7 exact zone-change identity. The permission is source independent after creation and grants no unusual timing permission.

## Shared implementation and wire

Private CounterPermanentExileCast instruction uses existing counter_stack_item. The selected target's current frozen stack definition determines permanent status. Grant requires successful counter, non-copy packet, a new incarnation, and actual final exile. Nonpermanent spells counter normally, even if Rest in Peace independently exiles them. No new token identity or State field.

Existing ExilePlayPermission gains omitted-false `without_paying_mana_cost` and ExilePlayExpiry gains WhileExiled. Exact departure cleanup already removes permissions; turn cleanup preserves this new expiry. Paid and land routes exclude this free-only permission, including alternate Adventure paid-origin checks. Existing life permission constructors explicitly remain false. The existing initialized-row `size_of` accounting includes the changed inline payload.

Public Action::AnnounceExileFree {card,face,flash_surcharge} selects only this route. Actual and preview configuration clears unrelated paid-exile spending/replacement/stash flags, preserving independent Inside Information/life and paid routes. It uses existing CastMethod::Free (X0), target/extra-cost/funding flow and cast rollback checkpoint. A private shared preview wrapper adds explicit independent-free configuration; existing permission callers retain their original configuration. Policy gives the new cast action the existing cast-announcement priority.

These are new public field/expiry/action values: central 163 was accepted at authoring; central 164 was accepted before integration, so root must coordinate 165 or later plus inherited-row audits. This leaf does not edit central schema or old trace baselines.

## Obligations and assertion map

| Obligation / risk | Independent expected outcome |
| --- | --- |
| Actual paid body | Paid4UU instant response counters paid opposing Bear; original costs stay spent; card in exile and Decree in graveyard |
| Beneficiary / ownership | Only Decree controller gains cast; later permanent retains opponent ownership with beneficiary control; own spell also legal target |
| Permanent / chosen face | Normal Goblin front grants; countered Clap! Snap! sorcery does not; exiled front can later choose either legal spell face for free |
| Counter failure / replacements | Uncounterable Sphinx remains and resolves; stale stack target gives no grant; countered copy gives no card; RIP-exiled nonpermanent gives no grant |
| Ordinary timing | Creature rejected with beneficiary priority on opponent turn; Flash Sailor allowed then; permanent without Flash allowed in own empty main |
| Optional / persistent | Can leave uncast across two real turn cleanups; exact departure/reentry destroys permission |
| Free cost obligations | X0; optional extra4 not waived; actual mandatory sacrifice paid; no unrelated paid or land permission granted |
| Independent routes | Same exact synthetic card carries free, paid modified and life permissions: selected route keeps correct flags/life; an independent paid land row remains usable |
| Owned / privacy | Raw/index, parent disposal, restore and rejected actor/action contracts at permission/announcement roots; ordinary private hands stay hidden |
| Failed transaction | Real foreign Aura entry invalidates casualty during free cast; exact exile permission restored with retained/reversed actual Ring mana |

Synthetic permission fixtures for nonpermanent Stir/Mind Spring/Inside Information overlap explicitly test the reusable permission mechanism, not a claim that Decree can counter those nonpermanent cards into its own exile. Real card-path tests independently establish the actual printed permanent requirement. No admitted generic prevention of exile movement exists; actual copy disappearance, failed counter, stale target and independently replaced nonpermanent destination are the concrete grant boundaries. The grant code additionally requires actual new final exile.

Preflight verifies fixture admission and costs (Decree4UU, SparksRR, Sphinx5UU; Stir additional4 is not waived). Local `mod hob_thranduil_decree;` registration excluded from commit; committed disjoint rollback registration at sos_mana_rollback.rs tail is `#[path="hob_thranduil_decree_rollback.rs"] mod decree_tests;`.

## Validation

Final coherent focused run passed66 executions/66 unique scenarios,0 failures/ignores and no compiler warnings. Stable `<cache>/hob-thranduil-decree/target`, authorized test opt0 author scope; default production/release profile unchanged. Persistent driver and entire resource-guard outer stdout/stderr are retained from launch. Inputs fixed through natural terminal; no cancellations or active-output deletion. Root owns full release validation and reviews.

Initial run `<cache>/hob-thranduil-decree/run-595c12360fc348ab9c40e26227319cbb` naturally terminated101 (original session12891) before test execution: two test references used a nonexistent Card::BearCub associated constant. Replaced with the preflighted named factory only after terminal. Guard228.525054649s including queue, peak2012852224bytes, all OOM counters0; no cancellation. Production rule source was unaffected. Full focused family will be rerun from coherent fixed inputs.

Second run `<cache>/hob-thranduil-decree/run-0a3fbaaaac344d76a419b10cc069a6ed` naturally terminated101 (session68184):13 own scenarios executed,2 passed/11 failed. Ten shared target fixtures supplied noncanonical Target::Stack for ordinary spells, which legal enumeration represents as Target::Spell(card); the independent three-route fixture omitted the life cost-clause commitment after mana payment. Corrected fixtures only after terminal; no production rules changes. Compile29.62s, guard38.868421324s including queue, peak2617651200bytes/all OOM0. Full affected family is rerun, not counted as accepted proof yet.

Third run `<cache>/hob-thranduil-decree/run-3306b1fa0c66407bbb6f68b00bfde08b` naturally terminated0 (session95083):66 unique scenarios passed,0 failed/ignored. Compile29.85s, guard31.326025099s, peak2593214464bytes/all OOM0. It exposed one unused test variable after the canonical-target repair. After terminal, used that original packet ID to assert it remains separately stacked beside its copy, and added explicit omission proof for the unchanged paid-permission JSON. The same complete family is rerun to obtain final warning-free evidence; no production code changed.

Final receipt `<cache>/hob-thranduil-decree/run-d8f4b482b92a4a8bbbe9f84cb6961588/`: original session74489 naturally terminated0, immutable inputs throughout. Groups: own13 (12 main+1 actual rollback), Inside Information16, Big Score core8, library/free11, Zaffai4, mana rollback3, cast rollback6 and storm5. All eight commands returned0 with no timeout/orphan. Compile29.75s; guard384.498530090s including shared queue; initialized memory peak2565226496bytes within6442450944byte cap, one job/thread, all OOM counters0. Native execution reported0.79s combined; these bounded scenarios are not a whole-set game or simulation benchmark.

Grouped command elapsed31.253713338s (separate from guard queue/overhead). Source checker verified5417 definitions/1608 blobs; Rustfmt and whitespace checks pass. Prior failed/diagnostic receipts above are preserved and not substituted for this final pass. Root still owes two independent nonauthor reviews, the coordinated public-schema adoption/inherited audits and integrated full release acceptance.

Receipt SHA256:

- `driver.py`: `8089f6131fcc377a77d6b9cb2d10f87426160dfec3c34798baddc15c9728369b`
- `results.json`: `052dc3270358e02ef127f043927feb87249af01150b4e5226106dda213e7197c`
- `resource.json`: `a5ca1cb4e0bf42d5c94ab0550ef0edef2758d20b8c97e31e62b5f4083aec9407`
- `outer.log`: `fcf23936460dec3b6a83e6d786ba4f0400a74324e3fdf74205c084b95266057e`

Authored source SHA256:

- `src/card_definitions/hob/thranduil_s_decree.rs`: `341422aaa5175c74553dc9e213e89bfdc156c2e94ac2d21df14878f2cbd5caa6`
- `src/assurance/hob_thranduil_decree.rs`: `b011fccdf13ecd55b8d5f6cab0b29cd07d5c83a6169d350c9336a7c6f841f805`
- `src/assurance/hob_thranduil_decree_rollback.rs`: `5c3de4de5080acebf3dd43b29f719c7d0b34a23358ca14c3c3f16ad8e8f8f30a`
- `src/engine_exile.rs`: `ec9b953ffd1466012edca701ba346247647644f197fc97b3235c8450641bb1eb`
- `src/engine_resolution_mana.rs`: `0a1a8cc3fa81b473c07871f2b8646c4d3fa408d67728cf12f6885054ba038fe6`

Implementation interval from isolated checkout creation to final receipt recording:1587.8s for one complete candidate card/shared free-exile origin, including source review, authoring, queue and four guarded runs. This is author throughput, not merged completion or simulation throughput.
