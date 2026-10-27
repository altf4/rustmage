# Settle the Wreckage: whole-card author proof

Base `0defaae8ed343e68ff264ca6ec17c55b0c893999`, isolated branch
`codex/hob-settle-wreckage`. One complete candidate; no new identities.
The integrator owns generated linkage, central assurance registration, schema
adoption and final release acceptance. No PR, push or merge.

Oracle identity `2a2ea189-b663-4f4a-bb23-ff7a4af25f71`:2WW white Instant.
Complete Oracle body read from pinned `data/standard/cards.json`. Complete pinned
XMage SettleTheWreckage.java and TargetPlayer.java, TargetCardInLibrary.java,
FilterAttackingCreature.java, AttackingPredicate.java, plus referenced StaticFilters
and full PlayerImpl.searchLibrary method were read. Complete source blobs are
pinned in paired provenance at revision492369c7844b8d76479f0393ff9eb6b0b0cf30d7.

Authoritative [Ixalan release notes](https://magic.wizards.com/en/news/feature/ixalan-release-notes)
confirm the spell targets only the player; hexproof creatures still get exiled.
The land allowance counts affected attacking creatures including tokens and
animated noncreature cards, even if they do not remain in exile. The qualified
search may find fewer than its allowance. Current official2026-09-25 CR cached
at `<cache>/rustmage-authoring/rules/MagicCompRules-20260925.txt`, SHA256
8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca:
CR506.4 (removed from combat),608.2b (illegal target),609.3 (do possible actions),
614.6 (replacement events),701.23a/b/g (qualified optional searches),701.24 (shuffle).

XMage's helper returns immediately on zero attackers and has no explicit shuffle
call after its library search. Oracle controls: the targeted player may still
accept a zero-card search and shuffle. The typed implementation offers that
choice; declining never searches or shuffles. Accepted fail-to-find still shuffles.
No card-specific source absence.

| Printed obligation / risk | Discriminating proof |
| --- | --- |
| Actual2WW instant, sole player target | paid answer to real legal attack declaration from defending seat; exact zero remaining mana; self target also works |
| All current attackers that player controls |2 attacking Bears exiled, idle own/enemy Bears remain; type/controller and combat membership read at resolution |
| Untargeted creatures | hexproof+indestructible attacker still exiled; player hexproof forbids cast target and later gain fizzles spell |
| Tokens / animated land count | actual declared Bear token and animated Forest each contribute1; token later disappears without losing allowance |
| Changes after cast / put attacking | blinked old attacker and control-changed attacker remain; newly created attacking token and surviving attacker exiled, count2 |
| Target player chooses / private search | both seats: defender's opponent casts, attacked player owns optional/search choice; caster sees no private library/selected search frame |
| Qualified bounded basic search | nonbasic Raucous Theater and Shock rejected; duplicate and over-limit selection rejected; Forest/Plains enter target player's BF tapped; extra Island stays library |
| Optional / zero / fewer / fail-to-find |0/2attackers, accept/decline, find0/1; accepted searches advance RNG/shuffle, declines preserve exact library and RNG |
| Normal land entry effects | real Rest in Peace doesn't replace requested exile; searched Plains enters tapped and causes Beorn's Hospitality landfall counter on own surviving Bear |
| Numeric full range | direct saved result65535/65536/u64::MAX reaches exact public maximum and serialized JSON without truncation or saturation |
| Replacements / non-moves | direct synthetic completed-event contract: redirected destination still counts; moved=false event does not; actual exile count/creature filtering tested separately |
| Transaction / owned | failed unaffordable or protected-player cast atomic; wrong search card/duplicate/overflow rejected; representative private search/high-bound roots check wrong actor, invalid index, stale revision, raw/index, both observations, snapshot/restore and ancestor disposal |

The redirected/prevented move contract is explicitly synthetic: the admitted zone
replacement inventory changes graveyard moves, not battlefield-to-exile moves.
It modifies completed ZoneChange facts for the shared count function; it does not
claim an admitted card redirects or prevents exile, or that its fixture actually
moved to the modified destination. The real-path Rest in Peace case proves that
a graveyard replacement does not falsely interfere with exile. Count uses completed
`moved` events rather than merely pre-counting attackers or filtering final exile
residents, matching the official ruling's replacement/commander boundary.

Shared typed changes:
- ExileAttackingCreatures(PlayerSet) freezes the current controlled creature subset
  of actual attackers, performs simultaneous raw zone movement, saves exactu64
  completed move count in existing Resolution.moved_count **before** dispatching
  frozen departure events. Tokens/noncreature cards count; suppressed non-moves
  do not. The existing result is already a public numeric group_move_count.
- MayPlayer uses the existing single-player EffectOptional frame.
- SearchTargetPlayerMovedCount reads only that existing result and starts the
  existing private qualified search. It deliberately has no generic AmountValue
  parameter requiring unrelated source/cast quantity capture walkers.
- SearchSpec.maximum and SearchChoice.minimum becomeu64; public
  SearchChoiceView.minimum/maximum likewise becomeu64. Action enumeration compares
  exact converted collection lengths. No narrowing casts, saturation, new state
  fields, allocation kinds, search choices or observation keys. Existing sizeof
  accounting includes the widened private frame fields.

**Wire adoption obligation:** both existing public minimum/maximum fields now
support a larger numeric domain thanu16. This is a schema change even though old
values serialize byte-identically. The author checkout inherits schema160 and
intentionally leaves the central schema file unchanged. Root reserved161 for
Inside Information while this leaf was authored; Settle must advance the next
accepted version (currently expected162 if integrated afterward), with inherited
row projections/audits. This leaf must not be admitted under the old schema.

Local-only registration, excluded from commit: add `mod hob_settle_wreckage;` in
`src/assurance/mod.rs`, and run `python3 scripts/link_catalogue.py` at integration.
The engine_attacking_exile helper's ordinary engine module registration is part
of the authored source commit. No generated central artifacts are committed.

Validation scope: stable dedicated `<cache>/rustmage-authoring/hob-settle-wreckage/target`, consistent
CARGO_PROFILE_TEST_OPT_LEVEL=0, remaining default author profile; no repository
profile changes. Every Cargo command guarded,1job/1test thread, shared6GiB cap.
Fixture names/admission/costs and scenario action shapes preflighted. Persistent
on-disk driver, complete outer stdout/stderr redirection from launch, per-group
logs and results.json writes. Every run remained fixed to natural terminal; no
cancellation, relaunch or active-output deletion.

Prior receipts in the same cache:
- run-8ecee9d71dd24a2da13209aabfc39ab8: session49286 naturally exit101;
  fixture compiled Keywords with unsupported BitOr. Corrected to existing union
  only after terminal. Guard23.519448s,peak1997668352,noOOM/timeout/orphans.
- run-98e3680463b948e5920a38900fcb1b2f: session94310 naturally exit0,50/50passed;
  compile31.10s,guard361.110662s including queue,peak2540404736,noOOM/timeout/orphans.
  Only after terminal, root approved narrowing generic scaled search to the
  dedicated saved-count instruction to avoid unneeded generic quantity walkers.

Final exact-source receipt `<cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319`:
original session20948 naturally terminal exit0; resource terminal=true/exit0,
guard221.126299s including shared-slot queue,compile30.76s,peak2594476032 bytes,
oom/oom_kill0; no timeout or orphan in any group. Eight positive groups:
8Settle +11library +3ForestGate +13WereWorm +1Erode +1target-player search
+7ElvenPassage +6castrollback = **50executions/50unique scenarios,0failures/ignored**.
Native reported execution total0.49s; bounded rules-scenario throughput only,
not a complete-game simulation or production performance claim. Source pins
5417definitions/1592source blobs, rustfmt and diff check pass.

Exact final source and receipt SHA256:

```text
0a3e35f7563e73e7c652eee3cc06b15c1a853ae1268fe7d6f027797de95e3baf  src/cards.rs
e988c459aee7208fb84544add9f9b8b7d76a0b7ce0119973f8921a44ce572ae7  src/engine.rs
e7292cae0c05024d05059e90c8b2be8c4f87fddbf90e7b4f131f27b5e52b5afc  src/engine_programs.rs
62e4d521ca0993a5a291940467d75bd700527ca1848492e997a3d6be9215dbeb  src/engine_search.rs
ed0f0cbabab735f511ee81f07bdc563d296b4612b127d44acc500cfef75fc69e  src/engine_attacking_exile.rs
2a4a24e5b8451d3ec0cac91656f484f231e6acd43e9ad3e0be09445ce240eacf  src/card_definitions/hob/settle_the_wreckage.rs
9cbb203f5e9f4fce8642d205b8da7d0bd46be4cc99e43ea3e7b5f7e4d4b24317  src/card_definitions/hob/settle_the_wreckage.json
b926553d0d34a8284ae36a17a756ee2c0aa03e81e1d46765448390384f5ff3ae  src/assurance/hob_settle_wreckage.rs
d048be7602c0d36d9bbca12909079597546cd621b9042da8b549238da90c71a7  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/resource.json
a2d00cb16a38426f59e838ed99ea2e93cc2dff24bd2028a114f05f49207ecc21  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/results.json
3f831c646af48e04685536bb9aa26bfbc30d32c4cdea2be149d8c6f52aa054bd  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/driver.py
7b7ff8d22a487abf8f8bd7620df52f22e912898493c546a0dc6e3560a278db76  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/outer.log
6d65cf8893ff95dfe5a7ce9dee57fe7e0297eda378bc381c729d307e41db0b83  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/0-hob_settle_.log
c676b65f66bf45cff6b4ef443c8f160f27bcc6282e7354fa4a1437de7836a294  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/1-big_library_tests.log
5f5ec03a990b67d47df5585f9035441fa8458a215641e5c150a9e7598f3df7a4  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/2-forest_gate_.log
7fa64e3c635eb3645279c5cbac019e73f8a14738c656950c7edd21a66caf65fa  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/3-hob_desert_worm_.log
cac7cb5a489c5636010d40cf89d97df663af9d4c292314316a885df6616bfe87  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/4-sos_erode_.log
d2d03060b8220c937d644a5d752a500c6abfb3e64d1e7598bc51db1ffa8936c8  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/5-msh_c_target_player_search.log
43e4457eba814795fc84d2ec651f577bab7c74ddba774fbbf2cddd4072767286  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/6-elven_passage_.log
e038d5d8af3783dbee949613c467fec44afbd2d16e41ccbdcf1d47c784efd602  <cache>/rustmage-authoring/hob-settle-wreckage/run-875d08f2bf564da6a643a51fda913319/7-cast_rollback.log
```

Remaining gates: two independent nonauthor reviews, coordinated new-schema
adoption/inherited row audits, integration and full release acceptance. No known
printed obligation or engine behavior blocker remains in this candidate.
