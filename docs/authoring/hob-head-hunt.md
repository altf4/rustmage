# Head of the Hunt: whole-card and replacement-core proof

Base96dcb190891ef9ba425ceeb99c3a4ca41e65a487; isolated branch
`codex/hob-head-hunt`. One complete candidate, no new identities. Root owns
central registration, generated linkage, schema adoption and final release.
No PR, push or merge.

Oracle0523e271-5e0b-4e4b-9eac-86f915d18952, full pinned body from
`data/standard/cards.json`:2BB black4/3Wolf, flash; opposing controlled creature
would die=>exile instead; when that replacement is used, create green2/2Wolf.
Read complete XMage HeadOfTheHunt.java and all helpers listed in paired
provenance at492369c7844b8d76479f0393ff9eb6b0b0cf30d7:
SimpleStaticAbility, ReplacementEffectImpl, ReflexiveTriggeredAbility,
CreateTokenEffect, FlashAbility and WolfToken. No source absence or Oracle
conflict. Existing exact `@token/hob-wolf`5412 is green2/2Wolf, reused unchanged.

Rules authority: official2026-09-25 CR cached at
`<cache>/rustmage-authoring/rules/MagicCompRules-20260925.txt`, SHA256
8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca.
CR113.7a/400.7 (independent old-source packet/new incarnation),603.2d
(object-trigger multipliers exclude created reflexive abilities),603.12
(reflexive trigger),604.2 (static effect while present/available),614.6
(replaced death event does not occur),616.1 (affected controller chooses among
competing replacements),733.1 (selective failed-cast mana reversal) apply.

| Printed obligation / risk | Discriminating proof |
| --- | --- |
| Paid2BB flash4/3Wolf | real cast during opponent's turn, exact mana0, stats/type/flash asserted |
| Mandatory opposing creature death replacement | paid Shock kills opponent-controlled Bear; actual result Exile, no Wolf until reflexive resolves |
| Correct source/controller and LKI | source leaves while reflexive waits; old packet creates Wolf for original controller |
| Actual token | token2/2,green,Wolf,untapped,summoning sick; recipe5412 unchanged |
| Current creature/controller, not owner/card kind | own Bear, enemy noncreature artifact, creature discarded from hand and direct exile do not trigger; your owned Bear stolen by opponent and animated opposing Forest do; opposing creature token also counts |
| Distinct competing Heads | two separate source-identified public options, JSON distinct; stolen creature's controller rather than owner chooses, each chosen Head yields only its own packet and one Wolf |
| RIP and Colossus competition | actual Lotus Ring tap/sacrifice mana payment; choose Head=>Exile+Wolf, choose RIP=>Exile/noWolf, choose self-shuffle=>library/noWolf; indestructible doesn't stop sacrifice |
| Simultaneous source departure | Head+opponent Bear die in one batch: HeadGY, BearExile, Wolf for old Head; opposing Heads both replaced withExile and each old source creates oneWolf |
| Removal/reacquisition and copied source | paid Witness Protection suppresses replacement, removal restores; in-place copied Bear provides Head ability; blink/new Bear incarnation cannot overwrite queued old Head program/source |
| Trigger interaction | real independent reflexive can be countered; Staff does not multiply it (onepacket, counter=>zeroWolves); same equipped Head's ordinary granted prowess still doubles; Doubling Season makes2tokens from one packet |
| Funding parity | projected Bolg sacrifice usesExile, no virtual death count or prematurely available Wolf; parent snapshot unchanged; actual paid mana matches and has one deferred packet |
| Retained/reversed mana | actual Lotus Ring sacrifice during Candy Grapple cost proposal, foreign Aura attachment makes casualty1 unpayable; keep event=>same exile and exactlyone opposing Wolf; reverse=>source restored and noWolf; both seats/owned failure root |
| Transaction/privacy/ownership | representative competing replacement and copied packet roots check wrong actor, invalid index, stale revision, raw/index, snapshots, both observations, restore and ancestor disposal; only affected controller sees replacement-choice view |

Mechanics are directly typed. New ZoneReplacementRule identifies opposing
creature death and its reflexive ability slot. Sparse static source registration
includes this descriptor. FrozenRules captures eligible sources' full pre-event
ObjectViews before any simultaneous departure. Suppressed abilities are excluded;
current control/types and copied providers participate. A selected source snapshot
is carried only in an ephemeral Plan until actual successful movement commits.
The post-success hook queues the existing ordinary reflexive triggered program;
no token is made inline, and no packet is queued for a nonselected/prevented move.
Frozen source availability, ability origin, program and controller survive source
departure or a later incarnation. Created reflexive packets exclude object-trigger
multipliers under CR603.2d; the repair below clears only the owned witness count.

**Public schema change:** ZoneReplacementKind adds
`ExileReflexive { source: ObjectRef, ability: u16 }`. Equal exile destinations
remain distinct because choosing a Head has a different consequence from choosing
RIP or another Head. Existing Exile/RevealShuffle/Battlefield values serialize
unchanged. Pick/choice owned-byte accounting uses sizeof and includes the larger
kind automatically. No new hot State fields, persistent provider histories or
public keys are introduced. Root has now adopted central schema162. This vocabulary must advance the next
coordinated schema (currently163); the exact version must be coordinated again if
other leaves integrate first.
The central schema file is intentionally untouched here. Integration must adopt
the new public vocabulary and audit inherited rows before admission.

Mana journal handling deliberately reuses existing exact packet retention:
record_cast_mana_zone_plan borrows the ephemeral Plan and keeps the existing
compact destination/shuffle journal. A successful original event's reflexive is
captured in ManaEvent.triggers. During replay, Plan.reflexive=None; the existing
replay truncates recomputed triggers and restores captured actual packets andIDs.
It must not recalculate old source witnesses or append another reflexive. Thus
there is no redundant provider payload in retained journal state and no new
journal allocation/byte-accounting requirement. New Plan payloads do not outlive
actual movement planning/commit. The funding destination match recognizes only
current opposing BF creatures; deferred token creation never runs in funding.

Committed noncentral test registrations:
- `src/assurance/sos_mana_rollback.rs` appends path
  `hob_head_hunt_rollback.rs`, module `head_hunt_tests` (reuses real failure helper).
- `src/engine_funding_sacrifice_tests.rs` appends path
  `assurance/hob_head_hunt_funding.rs`, module `head_hunt_tests` (reuses private
  actual/projected mana helpers).
Local central assurance registration `mod hob_head_hunt;` and generated catalogue/
coverage are excluded; root must register and run link_catalogue at integration.

Validation: stable dedicated `<cache>/rustmage-authoring/hob-head-hunt/target`, consistent
CARGO_PROFILE_TEST_OPT_LEVEL=0, remaining default author profile. Guard1job/1test
thread/shared6GiB; no repo profile change. Fixtures' exact names, admission,
printed costs and action shapes preflighted. Persistent driver plus complete
stdout/stderr outer.log redirection from launch and per-group results.json.
Every consuming job/source/driver stayed fixed until natural terminal; no cancel,
relaunch, active-output deletion or required-library removal.

Prior receipts preserved under the same cache:
- run-98ceaaf6f5cf4af78a5b77203ca339f9, session88208 naturally101:
  three test comparisons used private Card constructor and one test action
  double-borrowed the game; guard46.851706s,peak2000162816,noOOM/timeout/orphans.
  Fixed only after terminal. Removed an unconsumed unmatched legacy test filter
  before the next driver; no zero-execution group is claimed as coverage.
- run-060816d982da438582fc9181b54badb8, session48081 naturally101:
 9own passed/1funding assertion failed; actual post-payment packet was already
 stacked rather than pending. Corrected assertion to count actual source packet
 across pending/order/stack, and strengthened stolen-owner decision proof, only
 after terminal. No card/engine rule repair. Compile30.67s,guard67.373459s,
 peak2568613888,noOOM/timeout/orphans.

Initial leaf receipt `<cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03` (superseded by review repair below):
original session97026 naturally terminal exit0, resource terminal=true/exit0;
guard42.057303s including queue,compile29.76s,peak2556039168 bytes;
oom/oom_kill0, no timeout/orphans. Eight positive groups:
10own (8card+1funding+1retained rollback) +3existing mana rollback
+10sacrifice funding +14multiplicity +14Balin +11library +2RIP +6castrollback
= **70executions/70unique scenarios,0failures/ignored**. Native reported execution
sum0.80s; bounded rules scenarios only, not a full-game production benchmark.
Source pin checker5417definitions/1600blobs, rustfmt and diff check pass.

Initial leaf source/receipt SHA256:

```text
cfcdfb48a2486bd17c58c626a64ae66c9a1bc15ced3d302f3703c669a337c95a  src/cards.rs
407fb56ff28ca48ef5223302af68220b7382270d5b2bfaaf9bea106a98300ddc  src/engine_zone_replacements.rs
45b11e137dd1eba541e139fb30429d4f720a8a67d1ca492a8f6c0f3b263fbb8a  src/engine_stats.rs
87f44eb80ef7306c52af31bc07c7fc17a48ac91a9a2cf8e52f1afca871bd38ff  src/engine.rs
7edb625dd363bdc29fbf3d6b48a4a6afd8c6e0b7fe91de195c692a5a25769e68  src/engine_cast_cost_planning.rs
38fe7c03f500baa0179698fe6db62cbefe449cb8a5340b574cd8de00422e9d92  src/engine_cast_mana_recording.rs
28ae1c7a943ce73d0134449093d035bad0c1595e101bda32c44ddd38e2e8f497  src/engine_cast_rollback.rs
e689c31c34fcd8e2e0fd297a64feca431ff77c6ffe2a7b3b8b81e493c12d18c1  src/engine_cast_rollback_mana.rs
860686da04b7c0ee9aa74402308a378ab8fcb475d9d084b604e142b5936cbccf  src/card_definitions/hob/head_of_the_hunt.rs
62a0f27784265654e92d99799e5c5e77588aa4b1a5ab8fa26205562452a45102  src/card_definitions/hob/head_of_the_hunt.json
93d1602c66578bce145ef38cfde0b1c7bf21cb100ddfbffd30f26ce9d43cf4f7  src/assurance/hob_head_hunt.rs
e7b841c11309749825ac452e772c716f95f52149727e2b7ab596038028dfef73  src/assurance/hob_head_hunt_funding.rs
94e8bb35b35c71250c94cf54b3550fb5d92d6f362a481de07bad390452b00be2  src/assurance/hob_head_hunt_rollback.rs
7fe2cbdbe349cae7aa5b8d2d077db056271fdc2568891a2a0957524f685b4355  src/assurance/sos_mana_rollback.rs
75288757339388f96cf83f49e9a8c5d4e70258290128f11059884f065260960d  src/engine_funding_sacrifice_tests.rs
a3fe160a99654d2a8feb9918b2db897965bd741918fdcd594341298fa9038991  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/resource.json
bc3e062987c5742cd82af7eb570eaf457f767d9625231fa4b9ebf329bd9ebb3e  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/results.json
b99b516ff4bd448b5338f013a9248f2cee075db910d4540b6e1e9bf8396a1434  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/driver.py
f7da31123ad65aa80d42f8b1eb3256207f2da60d9b5c0a15b643f27734bd4dd8  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/outer.log
659ac924df1fbf3970ee6d34f277b55daa7c44abf48837f9661a7f5193cea93a  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/0-hob_hunt_.log
f14a7d1b0ecbc7e063cb2622f317b74068295001d331569c32d5741d09fffad4  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/1-sos_mana_rollback_.log
259c6514e17aed1ac6074bf172d7581f85b7d9a83ed0dc268e9f11e5e97725bd  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/2-hob_sacrifice_funding_.log
72c6fbfc6fde9511c6a6fa802119fbb323514aa5d4a5950085214eefd1e91d93  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/3-hob_extra_triggers_.log
61723859941aed272ce4cb705738b1f13dcd66d622bac2c8b9895d3320942ee9  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/4-hob_balin_.log
4295babcf09dd2464c11d467eac596acb60d2c3e942296a0efac700cc06eb25b  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/5-big_library_tests.log
0720a7c75d414eb29343251c5871ad4150d8ea0fdf6b5d1d7df34451a8e06477  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/6-big_rest_in_peace.log
16c0587d34947daf1fb2e81813bbd37d79ee2485af07aa07ca5041ebbdd4de18  <cache>/rustmage-authoring/hob-head-hunt/run-5f8d524bfb844a21b5341aabfdf68a03/7-cast_rollback.log
```

Remaining gates: two independent nonauthor reviews, coordinated schema adoption/
inherited audits, integration and root full release acceptance. No known printed
obligation or engine behavior blocker remains in this candidate.


## Blocking review P1: created reflexive multiplier repair

Base leaf3f3e2e6bceb1ac1bfc47ca9dd5c200fc08a8b68a had a rules bug and an
incorrect assertion: it passed the source object's additional-trigger count into
ordinary queue_trigger, so Wizard's Staff incorrectly doubled the newly created
reflexive. Its initial70passes were not evidence for that erroneous expectation.
Independent review identified CR603.2d: object-trigger multipliers apply to the
object's own triggered abilities, not delayed/reflexive abilities created by its
abilities. This repair corrects both implementation and the mistaken fixture.

At the post-success replacement hook, Plan's owned frozen source witness now has
additional_triggers set to0 before constructing its created reflexive packet.
The actual object, its live multiplier query, the global ordinary trigger path,
source availability/controller/program and all replacement selection/LKI facts
are unchanged. This excludes Staff and every other object-trigger multiplier
without preventing token-creation replacements such as Doubling Season.

The corrected both-seat scenario pays2for Goldvein Pick on the same Staff-equipped
Head. Its ordinary granted prowess produces2packets and changes4/3to6/5. The live
Head still reports1additional trigger. A later opposing death produces exactly
one created reflexive; resolve=>oneWolf, counter=>zeroWolves. The real retained/
reversed mana failure now also equips the opposing Head with Staff: retain the
approved mana event=>one restored reflexive/oneWolf, reverse=>noWolf/source
restored. This discriminates against redoubling in journal restoration. Existing
simultaneous source departure, opposing Heads, multiple chosen sources, RIP/
Colossus choices, copies, suppression, funding, owned roots and ordinary trigger
multiplicity regressions all reran in the same coherent family.

Repair receipt:
`<cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a`.
Original session23989 naturally terminal exit0; resource terminal=true/exit0,
guard35.629996s including queue,compile29.78s,peak2560102400 bytes; shared6GiB,
1Cargo job/test thread, oom/oom_kill0, no timeouts/orphans. Same8positive groups:
10own +3mana rollback +10funding +14ordinary multiplicity +14Balin +11library
+2RIP +6castrollback = **70executions/70unique,0failures/ignored**. Native reported
execution total0.83s, not a complete-game performance claim. Persistent driver,
complete outer.log redirection and per-group results retained from launch;
inputs fixed to natural terminal, no cancellation/relaunch. Source pins5417/1600,
rustfmt and diff check pass. No failed job in this repair iteration.

The repair itself adds no wire fields or vocabulary. The original Head
ExileReflexive vocabulary remains unadmitted; central162 was adopted elsewhere,
so root must coordinate next163 adoption and inherited audits. Central schema
file and generated artefacts remain excluded. Fresh TWO independent nonauthor
reviews are required on this corrected leaf before integration.

Repair exact source/receipt SHA256:

```text
83deded0f8f71c813e5e32442f420a6482590f9be08743fdaeb9a5fb9fb2029f  src/engine.rs
af021c1b161872c9d0a815d8712e0dfa3a0c311638210047fd30f086e68ad5da  src/assurance/hob_head_hunt.rs
8bd6badb7b88759f0863e4253cb4495d3431b491eed8a2bc6b055e75cb84c7d9  src/assurance/hob_head_hunt_rollback.rs
112981cbfbcf7ad5e8b47cc8bcd8e8450f6dfd10b5fdbf35de1c4d25be887969  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/resource.json
5fd8b82d3ffc6f57265177d8fa24fd2d479da554682b21e744a9dae54d1a1721  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/results.json
b99b516ff4bd448b5338f013a9248f2cee075db910d4540b6e1e9bf8396a1434  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/driver.py
87ce0ec8fa0cd2a0cc79a0e4a5c508a8b7b49abb0955163549a1ce073cba48d3  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/outer.log
1bf577d4532698e769e33c2b85671ee962a0937291b9884faed15b3a1dbe41ca  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/0-hob_hunt_.log
45112690950895ccf1d03a3972bdcd980bd3a204e66befad101778e9946cb84f  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/1-sos_mana_rollback_.log
b939bcb8c266e13e5a9c124d7bfce0be877175f119c14aec46b39c44b72433cc  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/2-hob_sacrifice_funding_.log
1cee2a6416d93edbabd7494da307cb901d04b09fd176e39e5c3c1fdaf9323090  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/3-hob_extra_triggers_.log
f8f5579986be986e9551838eb7a818214958f18d2cbc2ca1b28d7095577ee564  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/4-hob_balin_.log
26ccb636dd4e2e74ae593cba93f9187469741c6e989bc70e90761b5b201314e4  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/5-big_library_tests.log
0720a7c75d414eb29343251c5871ad4150d8ea0fdf6b5d1d7df34451a8e06477  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/6-big_rest_in_peace.log
fe8a865d34a5fc6cd4e664f314cb8c74c8770c900feab5b9addb15858fc61560  <cache>/rustmage-authoring/hob-head-hunt/run-05c65c6cd34349acade402b48f96c85a/7-cast_rollback.log
```
