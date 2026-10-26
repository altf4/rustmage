# The Notary Hobbits whole-card proof

Isolated author base e5c23533; branch codex/hob-notary. Root owns final integrated
acceptance, generated registration and two independent nonauthor reviews. No PR,
push, merge, new identities or public wire vocabulary. Existing schema158 copy
recipe overrides.legendary=false and source_copy_recipe projections are reused.

Oracle dc20b29b-856e-4d45-99cd-5e12cfc2324d:3GG legendary green1/1 Halfling
Advisor; nontoken ETB creates exactly two nonlegendary copies; tap addsC per current
controlled Halfling. Full XMage card and behavior helpers are pinned at
492369c7844b8d76479f0393ff9eb6b0b0cf30d7, including token copying and dynamic mana.

Official2026-09-25CR authority cache:
`<cache>/rustmage-authoring/rules/MagicCompRules-20260925.txt`, SHA256
8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca.
CR603.4 requires the intervening-if at both boundaries. Exact-incarnation token
status is immutable (CR111): a copy effect does not change whether an object is
a token. SelfNontokenEnters checks it once at entry; any later source incarnation
uses departure LKI, preserving the same answer. Tokens with the complete copied
ability therefore never recurse. CR707.2/707.9 govern copiable characteristics and
copiable exceptions;608.2h/113.7a govern current values and exact source LKI.605.1a
makes the untargeted mana ability immediate;302.6 still restricts its tap cost.

| Obligation/risk | Discriminating proof |
| --- | --- |
| Paid3GG1/1 and two exact nonlegendary copies | paid mana/body/name/color/types/MV/abilities; counters/Growth/tap/damage not copied; two untapped sick tokens, original legendary |
| Nontoken guard, including copies | token entry produces none; copy of nonlegendary copy retains exception; copied nontoken entry produces two |
| All other copiable values | layered artifact3/4/reach/name blueprint retained; only legendary changes; Halfling/Advisor preserved |
| Current and departed source values | change source to Bear before resolution; leave as Bear; old/new blink incarnations create two Notary/two Bear copies separately |
| Frozen ability controller | source leaves or changes control; original controller owns both tokens |
| Replacements and multiplicity | Bifur plus transient Dwarf=>4copies; Season=>8; external subtype not copied; two Seasons=>8 without recursion |
| Owned replacement/public privacy | live replacement recipe raw/index/restore/parent disposal; hidden Shock remains hidden; no schema vocabulary changes |
| Current controlled Halfling count | own converted subtype, lost subtype, stolen copy/enemy excluded, copied token's ability; initial sickness and already tapped rejection; zero is legal tap/no mana |
| Generic funding versus colored | real announcement/payment using3C for Pick2 leaves1C; cannot fundG; cost-root branch contract |
| Projected funding world | current zone/control/subtype witnesses and keys; copied source; zero cannot fund1; owned projected tap, live Game unchanged |
| Retained/reversed actual mana | reuse foreign-Aura failed cast: actual2C Notary tap kept/reversed, exact mana/tap restored, owned failure-root contract |

Runtime additions: private SelfNontokenEnters trigger predicate and private
CreateNonlegendaryCopyTokens instruction. CopyRecipe and existing CopyOverrides
merge only legendary=false, then existing simultaneous copy/token replacement
pipeline runs unchanged. Existing CountSubtype(color5,Halfling) supplies both
actual and funding quantities; no mana runtime extension.

Latest-departure copy recipe is refreshed only for programs opting into the new
source-copy instruction in the existing source-departure item loop. This avoids
using trigger-time values after a later copy effect changes the source. Ordinary
blueprints retain no new allocation; exceptional or changed definitions use the
existing optional Box. Previous exceptions are cleared if no longer applicable.
Existing consumers and inherited observation rows are unchanged. No new state or
pending choices are introduced; existing owned size accounting includes the Box.

Test registration hunks for integration:
- Local/uncommitted src/assurance/mod.rs: `mod hob_notary;` (root registers).
- Committed src/engine_funding.rs tail: cfg(test), path
  `assurance/hob_notary_funding.rs`, module `notary_tests`.
- Committed src/assurance/sos_mana_rollback.rs tail: path
  `hob_notary_rollback.rs`, child module `notary_tests`, reusing its private fixture.

Fixtures preflight against admitted definitions: Notary1/1/cost3GG, Bear Cub2/2,
Bifur/Season/Pick/Growth/Firebrand/Shock. Zero and retained mana tests use aged
battlefield fixtures; initial paid copies separately prove summoning sickness.
Both Notary and Firebrand are0power fixtures and cannot rescue the deliberately
failed casualty1 cost; their current Halfling count remains2. Fresh token-copy creation uses the runtime recipe system; no token
ID or legend-rule approximation is needed.

Validation passed fixed-input guarded coherent run; dedicated opt0 author target
and persistent driver/entire outer log/per-group results. Root retains production
release and full-pool game/performance acceptance.

Initial run-2e0329a0b78f4750b38102e9b17f790b naturally terminal101:
12passed/1fixture failure, no timeout/orphans/OOM. Compile35.79s;
guard166.648411s including shared-slot wait, peak2559549440bytes. The real
foreign-Aura fixture's casualty is1, not2; two added1power Halflings rescued the
cost and prevented failure. Made both0power/1toughness fixtures, preserving count2,
so the intended rollback opens. No runtime/card repair; all other source and
funding proofs already passed. Original logs retained; inputs fixed to terminal.

## Final terminal evidence

`<cache>/rustmage-authoring/hob-notary/run-762a87ae3c4344a8a5c4c9a021bb2bab`
preserves driver.py, entire guard stdout/stderr outer.log, results.json, resource.json
and all6native group logs. Session82750 naturally returned0, resource terminal=true,
exit_code0,6GiB cap,one Cargo job/test thread. No timeout, orphaned process,
memoryhigh/max/OOM events. Inputs remained fixed through natural termination.

171passed scenario executions across six groups:13Notary (10card/copy,
2funding,1real rollback),52copy,79token,7mana rollback,6cast rollback,14multiplicity.
Filters overlap; 152 unique scenarios pass,0failed/ignored. This includes ordinary
non-copy token creation consumers after the opt-in LKI change.
Compile29.84s, guard118.149187925s including shared-slot wait/driver overhead,
peak2515402752bytes. Reported executable scenario execution totals1.42s. Consistent
opt0 author cache/profile; no production-profile or full-game performance claim.

Implementation interval:1whole card plus shared typed copy/LKI support from
isolated checkout creation2026-10-26T23:14:09Z to final guard terminal
2026-10-26T23:31:45.049578+00:00, elapsed1056.050s including source reads, authoring, both
runs and their slot waits. Both attempts' persistent evidence remains preserved.
This local cohort interval is not extrapolated to full-set throughput.

Full source pins5417definitions/1579source blobs verify. Catalogue version
2afb668bfada18b2f69cb51370aa9268d74a8d05c6ca4429f803e78e664c3981.
Format and diff whitespace checks pass. No known whole-card blocker remains;
two independent nonauthor reviews and root integrated acceptance remain.
Central assurance registration and generated catalogue/coverage remain local,
excluded from the author commit; the two explicit cfg-test registrations listed
above are committed so their disjoint proofs compile after integration.

Compiled input SHA256:

- `src/card_definitions/hob/the_notary_hobbits.rs`: `562c47a7a3964978ec52f50a27dfcf4907be5aea9cc093dac4c3bb0eb885c5e2`
- `src/card_definitions/hob/the_notary_hobbits.json`: `d07b55127336142637b6539818e1fc41039a80de28e982c5db69e7c71589ae04`
- `src/cards.rs`: `8961f71e33fe224e77f07f6b368590439d3de537e0279e31de14d5f6a0c8a8ba`
- `src/engine.rs`: `ec56b81f3ca3a01344297ebf58b1e934e40211bbfbcf6126e4cf7006b4c2359c`
- `src/engine_programs.rs`: `50826157982b04e68e3b409b506e34dfaed9f74157b1da63a72da1a886b03a26`
- `src/engine_funding.rs`: `065df6bd283eebf27eaa6f1fd107289f3c41a2c8536d59a5b52291b6de4bb3a7`
- `src/assurance/sos_mana_rollback.rs`: `b9cb307ce6a9623ad6b46e4f5d45637c0a15ad8a296d0f2ce637726fc7ab0152`
- `src/assurance/hob_notary.rs`: `2dffeebbbcc575d2442db9855a636b70f9d8a08bcad4cb6bc99c14ad6c4def52`
- `src/assurance/hob_notary_funding.rs`: `b7eb357c8efef9304771dfee0daabea1da25d6438d8a3bb3aebfd78fd7b42def`
- `src/assurance/hob_notary_rollback.rs`: `9c06e4d7a30b3e43cb826d9145ba348d1b539e602ac97395c5cb7118ecf51abd`
- `src/assurance/mod.rs`: `9ad2c4dada48a1b1a73150df7eaf9ec0c09f79495f62b306d78e48a587f7278a`
- `src/generated/catalogue.rs`: `3a1c8d2a8bb3b290387c582d453284af8dc3d9ce4f984eecf9647c95da5322e8`

Receipt SHA256:

- `driver.py`: `8f930aeea600f2fced2e3542fe9ca9185bd49546e5ef3fa265f23cca9cdffe01`
- `outer.log`: `9e35c336b255c11458760c34f1b4a640d3f8ac3307d2254979e64ebdf68605ba`
- `results.json`: `eb36f0aba3becfab0a5cceaef07a484b4bcb5d7e9a5a85af30db6f623095ccd7`
- `resource.json`: `3ae8221c5d4e93dcab1774700a7e65847bddaa76b037886e2e2bf0287265217a`
