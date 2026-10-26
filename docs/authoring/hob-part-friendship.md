# Part in Friendship: whole-card proof

Base `3fa4cf1a`; isolated branch `codex/hob-part-friendship`.
The integrator owns central registration, generated linkage and final release
validation. No PR, push, merge, token identity or observation schema change.
The inherited schema is159; no new public keys, values or vocabulary are added.

Full Oracle identity `e1a9c467-2d64-4e11-adf3-7333c923b187`, cost4G,
green Enchantment. Complete body read from pinned `data/standard/cards.json`.
Complete XMage `PartInFriendship.java`, `DiesCreatureTriggeredAbility.java`,
`TriggeredAbilityImpl.java`, and referenced StaticFilters definitions and
PlayerImpl.putCardsOnBottomOfLibrary method were read. Their complete source
blobs are pinned in the paired provenance at revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. No source absence or Oracle discrepancy.
The threshold is current controlled land count, never the departed creature's
power or an earlier land-count snapshot. X contributes0 to library mana value.

Rules authority: official2026-09-25 CR, cached at
`<cache>/rustmage-authoring/rules/MagicCompRules-20260925.txt`, SHA256
`8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca`.
CR113.7a (independent ability),400.7 (new incarnation),603.6c/603.10
(death/leave-battlefield pre-event information),608.2h (resolution-time
information), and614.6 (replacement event) apply. Once-per-turn caps are
acquisition-time trigger limits, not successful-resolution limits.

| Printed obligation / risk | Discriminating scenario |
| --- | --- |
| Actual4G paid cast and death event | paid enchantment, exact zero remaining mana; paid Shock kills own Bear; real reveal follows, both seats |
| First creature only; reveal public, no deeper disclosure | top Shock/Forest/Bear, deeper Growth excluded; both observations agree on revealed prefix and only Bear candidate |
| Mandatory movement / live land threshold |0/1lands=>hand,2/3lands=>BF for MV2; MV3=>hand with2lands;5 enemy lands excluded; nonbasic land changes to your control after death and counts |
| Library mana value and SBA after entry | XG Wildwood Scourge is MV1:0lands=>hand,1land=>BF as0/0 then dies to SBA; no additional trigger after used cap |
| Untapped BF entry or publicly known hand | Bear untapped on BF; hand result's exact incarnation appears in opponent known_hand |
| Rest randomly bottom, preserve deeper library | two-card remainder has exact multiset, RNG advances; deeper card remains top; frame gone afterward |
| No creature / empty library | entire eight-card noncreature library revealed then randomly bottomed; empty library completes; neither draws nor loses |
| Once each turn, including countered trigger | simultaneous two deaths=>one packet; actual counter_stack_item consumes it; later death no packet; opponent's next turn permits one |
| Current creature/controller/token/death qualifications | own token, enemy creature, hand return, Rest in Peace exile-replaced death excluded and do not consume cap; stolen opponent-owned creature you control qualifies |
| Source independence | source leaves or changes controller after triggering; original controller still uses own library/lands and receives creature |
| Source itself if currently creature | shared layer fixture animates enchantment into Dwarf2/2; self death qualifies; live Bifur+story cannot defeat printed trigger cap |
| Ability removal / reacquisition / fresh incarnation | paid Witness Protection on animated source suppresses; removal restores unused cap; source blink gets new cap in same turn |
| Paused entry | Adaptive Automaton entry-type choice suspends, preserves selected exact creature and uncommitted remainder; choose Dwarf, creature enters and remainder bottoms |
| Legality / privacy / transactional owned branches | mandatory Finish before selection and noncandidate Select reject atomically; branch_contract at public reveal and paused entry tests wrong actor, invalid index, stale revision, raw/index equivalence, snapshots, both observations, restore and ancestor disposal |

Shared core: a private typed destination
`BattlefieldIfManaValueAtMostElseHand(AmountValue)` supports exactly one forced
candidate (asserted). It evaluates current quantity and creature library MV at
resolution, then normalizes to existing Hand/Battlefield before constructing the
existing library frame. No new per-game fields, continuations or allocations.
Existing enum storage/owned byte accounting uses sizeof automatically. The frame
never stores the conditional route; public destination uses existing strings.
Existing UntilKinds, public reveal, known-hand registration, random bottom and
entry continuation remain authoritative. Existing death observer/source LKI,
OncePerTurn and additional-trigger cap are reused without changing them.
No new unsupported entry-prevention mechanic is introduced; actual graveyard
replacement and SBA outcomes exercise relevant admitted replacement/failure paths.

Local-only central registration (exclude from leaf commit):
`src/assurance/mod.rs` adds `mod hob_part_friendship;`.
Run `python3 scripts/link_catalogue.py` on integration. Generated catalogue and
coverage changes are intentionally excluded.

Validation uses a stable worktree-specific target at `<cache>/rustmage-authoring/hob-part-friendship/target`,
consistent `CARGO_PROFILE_TEST_OPT_LEVEL=0`, default remaining author profile,
one Cargo job/test thread and shared6GiB guard. This is author-test scope;
production optimization/performance and full release acceptance remain root-owned.
Fixture identities, complete admission and exact costs/types were preflighted.

Prior natural terminal receipts, preserved under the same cache:
- `run-4294750d17c74c528366157ca3f3011f`: exit101, compilation failed on three
  fixture change_controller calls using CardId rather than exact ObjectRef;
  guard39.791s, peak1981300736 bytes, noOOM/timeout/orphans. Corrected only after
  original session67429 naturally terminated. Also enabled actual Bifur story
  and added X-cost coverage before next coherent run.
- `run-ee75102d73cf4454a5cb10eca0acd6c6`: exit101,9passed/1failed;
  simultaneous-death fixture omitted pre-event death observers and queued0.
  Guard32.846s, peak2496425984 bytes, noOOM/timeout/orphans. Corrected observer
  capture only after original session58366 naturally terminated. Added RNG
  assertion and pinned trigger-limit helper; no engine/card behavior repairs.

Final coherent receipt `<cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58`:
original session79475 naturally terminal exit0; resource report terminal=true,
exit_code0, guard31.466795s, peak2502586368 bytes, oom/oom_kill0;
no group timed out or left orphan processes. Compile30.39s. Five positive groups:
10 Friendship +3 Page +11 existing library +14 trigger multiplicity +6 cast
rollback = **44 executions /44 unique scenarios,0 failures/ignored**.
Reported native execution total0.60s; this bounded scenario throughput is not a
complete-game simulation benchmark. Every job's entire stdout/stderr was
redirected from launch to its persistent outer.log; results.json persisted after
each group. Inputs stayed fixed through natural terminal; no cancellation.
Source pin checker5417definitions/1583source blobs and rustfmt/diff check pass.

Final immutable source and receipt SHA256:

```text
7ad076bafb5f1a035fa4f61d7e4ed6a1e45afa43f496165ac3f70cb768227d75  src/cards.rs
da2f87a40baa44b712a2e7c9d386917d66ce06659420e2432adfc63df37179b6  src/engine_library_selection.rs
7478082c42b9ec08ebaa9b11fe9cb4a08480eab2b5d7cb92813bc44211ad4baa  src/card_definitions/hob/part_in_friendship.rs
1e8736a3f7b73c962d60cd2bb3d5ebe969b5a633e86fe51473c6f561fd471a99  src/card_definitions/hob/part_in_friendship.json
1c053987c340db9bba0aae0d9147f44ce7643a5610c7cb3656a782c7633e7ac3  src/assurance/hob_part_friendship.rs
bbad97e6fbf9a336701d4d329df9135bbe5be7a3b3fc463d9f2e56239a383dc6  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/resource.json
bf5166bc3550e4c19a7fe6def476b7cdcc2bd0f7155f7b6125e87e090e841392  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/results.json
220e92d19ae4ea7b53e58ff1071e6e86ca2b3c15b2a2adf59f45e04739c87326  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/driver.py
16987314d0c9483a7f6b338ba191bc955da8dcc6be6eb078ae9be7afd8f38f1e  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/outer.log
5c8111c381f61c1ef48222aa5da2788a03f6f7c2a7080794e2bc87bfd0ca7503  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/0-hob_friendship_.log
bf76ce3a64674225e42e7b9e79cfb481c882424987b3dcbc7c242aac8c4de422  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/1-sos_page_.log
a870bb73a084299531c6a6e3803aeb5c62dcc4be540435108781dba645464d13  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/2-big_library_tests.log
79cb80b31c63d3643f8b673e9a171acb9b73f08abe993476ad5f98538b0579f5  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/3-hob_extra_triggers_.log
d170c8c0a35dbebf136e6b32bc208d2a694e2f12c8035d5d7bc763b1aa17387b  <cache>/rustmage-authoring/hob-part-friendship/run-75bb047a302e48a397941bdd173c8b58/4-cast_rollback.log
```

Remaining gates: two independent nonauthor reviews and root integration/release
acceptance. No known card obligation or shared-core blocker remains.
