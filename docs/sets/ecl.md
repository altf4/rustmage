# Lorwyn Eclipsed (ECL)

## Scope and baseline

Selected set: Lorwyn Eclipsed. Preserve the repository's explicit tabletop/MTGO
Standard snapshot of 2026-10-15; this run does not rotate the format.
Started 2026-10-22. Integration branch: `codex/ecl-complete-set`, base `61c61b69`.
Tracking issue: https://github.com/altf4/rustmage/issues/462. No implementation PR until all set gates pass.

Pinned XMage revision: `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.
Membership: [frozen manifest](../../data/sets/ecl.json), 273 physical identities
from 413 pinned XMage printings; zero identities unresolved against pinned facts.
Independent Scryfall set reconciliation matches all 273 physical identities (2026-10-22); see [reconciliation evidence](ecl-membership-reconciliation.json). Initial admission: 107/273;
166 gated. Admission is inherited coverage, not whole-card certification.
Oracle facts: `data/standard/cards.json`; canonical ownership:
`data/catalogue-identities.json`. Banned members remain in scope.

## Card obligations and evidence

The frozen manifest accounts for every identity. The [per-card obligation ledger](ecl-obligations.json) records every face and printed rules line with canonical ownership. All107 inherited cards have passed source/rules audits, with paid interaction evidence recorded below and in the validation JSON.

## Current implementation checkpoint (2026-10-22)

Admission flags: **273/273**, comprising **107 inherited** and **166 newly
implemented/tested physical candidates**; **zero remain gated**. All107 inherited cards have passed source/rules audits.
The supporting closure includes eleven tokens, seven backs and one emblem;
all have complete source pins and parent interaction evidence. Final combined
full-game acceptance, release validation and two published-head reviews remain
open. No newly authored cards have been merged upstream.

Three isolated authors implemented black/red, white/blue, and green/colorless
cards, with the integrator handling multicolor cards, transforming cards and
shared integration. All authored behavior is typed Rust. The combined ECL rules
suite passed **716 scenarios in 30.68 seconds** before the final funding repairs;
subsequent focused checks cover those repairs separately.

Shared mechanics include blight, wither, behold, vivid, evoke, convoke, persist,
proliferate, surveil, landcycling, transforming faces, linked exile and free
casting, exact counter payments, chosen creature types, copied permanents and
spells, entry replacements, Conspire, and trigger multiplication. Source and
ability epochs preserve queued triggers and linked permissions across copying,
control changes and source departure. Each physical card's obligations and
supporting token/face evidence are recorded in the JSON ledgers.

## Validation and measurements

The most recent full release checkpoint, at **265 admitted cards**, passed all
eight phases at `a9a47743`: **3,094 native tests** (one ignored), nine catalogue
checks, all 22 Foundations game suites, ECL regression games, provenance, Clippy,
release compilation and the benchmark. It took **640.02 seconds**, peaked at
**3.5 GiB**, and cleaned 1,143,128,064 disposable bytes. The report is
`<cache>/rustmage-validation-dydsqtfp/report/result.json`; required shared
libraries remain at the exact paths and hashes recorded in the validation JSON.
A new full release run on the final 273-card integration remains required.

That release's Freeform regression benchmark measured 49,832 decisions in
0.247475027 seconds: **201,361.73 decisions/second**, 404.08 games/second,
clone 205.46 ns and restore 154.59 ns. It includes native policy and observations;
it excludes JSON, model and tensor work. This measures regression performance,
not ECL acceptance.

The strict 273-card workload at `5180907b` completed **220 natural games** with
143,102 accepted decisions, 2,841 owned branches, 513 private decisions and full
journal replay in **205.91 seconds** (694.97 decisions/second). It includes all
273 physical identities across 29 pool decks and 13 mechanic themes, both
starting players, pool seeds 0/7 and theme seeds 0/7/19/31. All gameplay, privacy,
rejected-action atomicity, branching and restore checks passed. **The test failed
its final mechanic coverage requirement:** graveyard-entry copies and Dream
Harvest free casts were not observed. This is diagnostic evidence, not accepted
set validation. Resource report `42f48a66d0bf4d85a7783d0454c0f8a8` and
`<cache>/ecl-acceptance-strict-273-fixed.log` preserve that outcome.

The committed harness at `1207a753` preserves those 220 games and adds 16
reanimation games, for **236 final games**. A focused 24-game proof passed in
26.68 seconds with 19,883 replayed actions, 428 owned branches, 109 private
decisions, **23 actual Dream Harvest free casts** and **27 actual graveyard-copy
tokens**. The Harvest detector now binds the exiled object at announcement to
the later committed spell ID. All 20 mechanic requirements remain enforced.
The full 236-game combined run remains pending.

Three conservative mana-funding bounds removed repeated impossible-payment
searches involving restricted mana, missing eligible colors and missing creature
tap witnesses. Exact formerly slow queries now take approximately 0.3–1.1 ms;
117 funding regressions and all-target Clippy pass. A separate exact funding
repair handles mana costs that release linked exiled permanents and permit new
mana, convoke or entry choices. It remains under focused verification.

The measured authoring interval produced 166 newly implemented and focused-tested
physical cards in 26,826.03 seconds: **22.28 cards/hour**. This excludes inherited
audit credit, final acceptance, independent published-head review and upstream
merge. Compiler, hardware, commands, seeds and retained reports are in
[ecl-validation.json](ecl-validation.json).

## Resume and remaining work

Branch `codex/ecl-complete-set`; issue [#462](https://github.com/altf4/rustmage/issues/462).
Authenticate frozen membership using `python3 -m scripts.set_status ECL --check`.
Resolve the two missing actual-mechanic coverage paths, finish and integrate the
linked-return funding repair, then rerun the strict whole-pool workload and its
optimized throughput companion. Regenerate links and run final release
validation, including Python binding smoke checks for the changed public schema.
Keep consuming inputs fixed until each process terminates; poll the same job.

Publish one implementation PR only after every local gate passes. Obtain two
fresh independent nonauthor reviews of its exact final published head and resolve
all findings. No implementation PR exists yet. Merge remains subject to
repository permission. Historical checkpoints below record intermediate work;
the current section and JSON ledgers govern resume decisions.

## Verified checkpoint228 and inherited audit

Revision `7f001174` passed all8 release phases in553.77seconds:2,826 native tests (one ignored),9 catalogue tests,all22 Foundations game suites and ECL games,Clippy,provenance,release build and benchmark. Shared memory peak3.3GiB;1,100,824,576 disposable bytes cleaned. Required libraries and exact SHA hashes remain at the paths recorded in `ecl-validation.json`. Freeform regression:49,832decisions/0.23017189seconds=216,499.07decisions/s;clone212.57ns,restore190.40ns. This is regression measurement,not full-set ECL acceptance.

Spiral into Solitude brings admission to229/273. The inherited evidence audit now covers all5 basic lands against full pinned sources and actual land play/mana payments with both players,private observations,invalid-action atomicity,and owned raw/index replay/restore (0.07seconds).102 inherited definitions still need the evidence audit.

## Eirdu and optional tap integration

Both Eirdu/Isilu faces are implemented,including live/speculative creature-spell convoke and other nontoken creatures’ persist with frozen departure LKI,owner returns,and counter entry.301 ECL scenarios passed9.42seconds;106 funding regressions passed3.13seconds. All integrated Java provenance passes. Gravelgill Scoundrel and Meanders Guide add paid optional tap choices and a post-payment reflexive return;306 ECL scenarios passed10.05seconds on the author branch. Combined232-card release verification follows.

## Verified checkpoint232

The232-card fixed-input release passed all8phases in556.81seconds:2,845 native tests (one ignored),all9 catalogue checks,all22 Foundations game suites,ECL games,Clippy/provenance,release build and benchmark. Peak3.3GiB;1,102,700,544 disposable bytes cleaned. The three authored source formatting inputs were fixed before validation and recorded afterward in5201786d; exact hashes and required retained libraries are in the report. Mornsong Aria is integrated afterward for233/273 admitted,40 gated; its four focused scenarios and90 shared search regressions pass. Subsequent shared-slot verification is allowed to finish before another long full run.

## Inherited audit10/107

Gangly Stompling,Goldmeadow Nomad,Merrow Skyswimmer,Flock Impostor,and Rooftop Percher now have complete pinned source audits and actual paid interaction evidence. Scenarios verify both seats,hybrid routes,trample with real blockers,graveyard exile as cost and sorcery timing,exact token colors/types,opponent-turn flash with owner bounce,and partial graveyard target failure. Five inherited scenarios including the basic-land matrix pass0.18seconds; existing complete convoke/hybrid rule scenarios provide additional Merrow evidence.97 inherited cards remain unaudited.

## Combined checkpoint237

Kinscaer Sentry, both Sygg faces, Mirrormind Crown, and End-Blaze Epiphany are integrated. Sygg passes all2,845 native regressions and32 transform scenarios, including independent combat draw grants, color protection, attachment legality, and next-controller-turn expiry. Crown passes89 focused scenarios plus replacement, mobilize, and copy regressions. End-Blaze passes111 black/red scenarios and13 scalar-X regressions. These additions await combined release validation.

Five additional inherited creatures have full source and paid-body evidence: Boggart Prankster, Enraged Flamecaster, Kulrath Mystic, Gnarlbark Elm, and Iron-Shield Elf. Existing complete attack/cast trigger, named counter payment, discard cost, and indestructibility scenarios provide interaction evidence; all six inherited scenarios pass0.22seconds. Audit progress15/107, with92 remaining.

The combined237-card branch04b2ae89 passes381 ECL scenarios in13.05seconds. Resource report8947260bbbe8468e978883be0c3b5a78; peak2.6GiB.

Tam, Mindful First-Year is integrated with both hybrid casting routes and dynamic color-based hexproof for other creatures. Five focused cases pass0.19seconds and37 shared hexproof scenarios pass0.67seconds; all-target Clippy passes. Admission238/273;35 remain gated.

Kindle the Inner Flame is integrated.117 black/red scenarios pass3.37seconds;27 flashback regressions pass0.44seconds and8 copy-characteristic scenarios pass0.17seconds. The printed flashback additional behold cost uses three distinct Elemental permanent/card witnesses with joint funding; ordinary and independently granted casting routes and spell copies remain tested. Combined239-card release verification follows.

## Verified checkpoint239

Revision904c25c2 passed all eight release phases in580.76seconds:2,896 native tests (one ignored),9 catalogue tests,all22 Foundations game suites and ECL games,Clippy/provenance,release build and benchmark. Shared memory peak3.3GiB;1,107,525,632 completed disposable bytes cleaned. Required libraries are preserved at the exact paths/hashes recorded in the JSON. Freeform regression49,832decisions/0.23473785seconds=212,287.88decisions/s;clone205.26ns,restore178.25ns;initial logical state21,080bytes. This remains regression evidence,not full-set ECL acceptance.

## Combined checkpoint241

Grub and Meek Attack are integrated after the verified239-card release. Grub passes39 transform scenarios in1.22seconds,8 intrinsic-copy regressions in0.19seconds,17 mobilize regressions in0.16seconds,and all-target Clippy. Its non-targeting optional blight payment preserves the exact creature for copying before state-based actions; attacking copies receive a copiable removable sacrifice ability without gaining haste. Both simultaneous lethal/legend-rule choices and attacking planeswalker choices are checked. Meek Attack passes124 black/red scenarios in3.74seconds and all5 Sentry shared-entry regressions in0.10seconds,including private qualification,untapped nonattacking entry,source departure,permanent haste and captured-controller delayed sacrifice. Admission241/273;32 gated.

Abigale, Eloquent First-Year is integrated with permanent ability removal and flying, first-strike, and lifelink counters. The eleven whole-card scenarios plus five Tam cases pass0.51seconds;361 shared counter scenarios pass6.31seconds; all-target Clippy and formatting pass. Counter timestamps renew on addition of the same kind, retain on partial removal, and disappear with the final counter. Counter prohibition, competing replacement effects, private cards, borrowed counter overlays, combat and copy exclusion are tested. Admission242/273;31 gated.

Rhys the Evermore and Slumbering Walker are integrated with exact optional counter removal domains,including named and u64-wide counter allocations,and granted persist.114 white/blue scenarios pass2.63seconds;357 ECL author-branch scenarios pass12.07seconds;29 temporary-death regressions pass8.00seconds. The raw legal domain accepts exact allocations beyond the bounded indexed representative menu. Admission244/273;29 gated. Combined interaction verification follows.

The244-card integration1aa48507 passes428 ECL scenarios in15.11seconds; memory peak2.7GiB. This includes the newly combined counter-removal,keyword-counter,hand-entry and linked-copy behavior.

## Oko and inherited audit20/107

Both Oko faces, the Elk token and independent chosen-type emblem are integrated. All2,905 native regressions pass78.55seconds; six whole-card scenarios pass0.24seconds; all-target Clippy passes. Tests cover paid loyalty, both paid transforms, once-per-turn use across asymmetric face indices, permanent type effects,next-original-controller-turn growth, exact post-mill optional selection,source departure,zero/stale targets,token creation after loyalty death,and the chosen-type emblem affecting later creatures.

Chitinous Graspling,Chomping Changeling,Changeling Wayfinder,Shore Lurker,and Glister Bairn have complete pinned source audits. Existing changeling,optional search/removal,vivid and library interaction suites are supplemented by paid hybrid bodies in both seats and both private surveil destinations. All eight inherited scenarios pass0.31seconds.20/107 audited;87 inherited cards remain. Admission245/273;28 gated.

Curious Colossus is integrated with a resolution-frozen opponent creature group and permanent ability/type/base-stat effects.432 ECL author-branch scenarios pass15.74seconds;44 subtype regressions pass0.25seconds; Clippy passes. Scenarios preserve existing counters,growth,artifact types and colors,keep changeling types through later ability loss,and check later layers,control changes,cleanup and blink. Actual Rhys removal of Abigale keyword counters and subsequent persist also passes. Admission246/273;27 gated.

Soul Immolation is integrated with bounded nonmana X,mandatory X-sized blight payment,and simultaneous opponent/player-creature damage. All2,912 native regressions pass78.39seconds on the author branch,including133 black/red scenarios. Blight quantities preserve the full u32 scalar domain; zero payment requires a controlled creature but creates no counter placement. Forced blight now permits choosing a counter-prohibited creature,with placement correctly suppressed. Current September25 CR701.68a/b were independently rechecked. Admission247/273;26 gated.

Sanar, Innovative First-Year is integrated.24 student scenarios pass0.71seconds,29 library-selection regressions pass0.12seconds,130 exile regressions pass4.04seconds,and all-target Clippy passes. Public reveal-until-nonlands captures the available colors; injective color assignments use complete matching,including nongreedy cases and Hall-deficient subsets. Selected cards get exact turn-limited cast-only permission; land play permissions remain independent. Zero-color and short-library paths still shuffle. Admission248/273;25 gated.

Figure of Fable is integrated with every hybrid payment route,persistent subtype/base-stat evolution,resolution-time prerequisites,and protection from each opponent.411 ECL author-branch scenarios pass15.07seconds,45 growth regressions pass1.41seconds,and all-target Clippy passes. Tests cover both seats,all staged cost combinations,paid no-effect branches,stack ordering,source incarnation,cleanup and downgrade,preserved counters/growth,ability removal,opponent target rejection,colorless blocking/damage,unpreventable damage,own attachments and current-controller protection after control transfer. Admission249/273;24 gated. The combined249-card release follows.

## Verified checkpoint249

Revisionf9cceb28 passed all eight release phases in610.57seconds:2,968 native tests (one ignored),9 catalogue tests,all22 Foundations game suites and current ECL games,Clippy/provenance,release build and benchmark. Peak3.4GiB;1,116,618,752 completed disposable bytes cleaned. Required libraries remain at exact paths/hashes recorded in the JSON,including the retained239-card artifacts. Freeform regression49,832decisions/0.237338338seconds=209,961.86decisions/s;clone128.21ns,restore124.46ns;initial logical state21,080bytes. Full-pool ECL acceptance and inherited audits remain open.

Wanderwine Farewell and Sunderflock are integrated.127 white/blue scenarios pass2.92seconds; an added suspended linked-return case passes0.04seconds. Farewell counts actual returned permanents and checks the Merfolk condition after linked-entry choices complete. Sunderflock uses greatest Elemental mana value,including noncreature Kindred permanents,and distinguishes actual casts (including free casts) from copies/direct entry. Admission251/273;22 gated.

Deepway Navigator is integrated.468 combined ECL scenarios pass18.08seconds; all-target Clippy passes. The public frozen declaration history records exact attacker incarnations,controller and subtype facts,including attacks before Navigator enters. Repeated attacks deduplicate qualified objects while a new incarnation or later qualifying declaration remains distinct. Attacking token entry is excluded; cleanup resets history. Tests cover live ability loss/control,opponent-turn flash,and untapping other Merfolk including Kindred permanents. Admission252/273;21 gated; observation schema152.

High Perfect Morcant is integrated with optional non-targeting proliferate choices and a single mixed counter-placement event.468 ECL scenarios pass16.63seconds,383 shared counter scenarios pass,and all-target Clippy passes. Eleven Morcant scenarios cover source/other Elf entry,Kindred Elf witnesses,fresh creatures paying tap costs,sorcery timing,named and keyword counter timestamps,prohibition/replacement order,exact incarnations,and poison defeat/overflow rollback. Poison is the only player counter produced by the admitted pool. Admission253/273;20 gated; observation schema153.

Glen Elendra's Answer is integrated.134 white/blue scenarios pass3.20seconds,20 uncounterable regressions pass0.38seconds,and library Clippy passes. Frozen eligible opposing spell/ability packets are countered together; uncounterable packets survive and do not contribute Faeries. Physical spells move in one graveyard batch and replacement destinations still count successful counters. Tests cover actual spell copies,departed ability sources,own-stack preservation,Moonshadow's one-or-more event,and Faerie colors/types/flying. Admission254/273;19 gated.

Combined254-card revision60017348 passes494 ECL scenarios in18.89seconds; resource report055d5b404d4f46899d58b947824d38b5;peak2.7GiB. This checks the newly combined Navigator history,Morcant proliferation and simultaneous countering with the existing ECL mechanics.

Dream Harvest is integrated. Six focused whole-card scenarios pass0.17seconds,131 shared exile regressions pass4.00seconds,10 free-casting permission regressions pass0.47seconds,and all-target Clippy passes. Each opponent exiles exact top cards until total mana value reaches5; zero-value cards count as cards but add no value. Exact turn-limited free cast-only permissions support foreign ownership,normal timing,mandatory additional costs,alternate spell faces and X=0. Independent paid permissions remain separate; lands gain no play permission. Admission255/273;18 gated; observation schema154 adds the public without-mana permission field.

Lluwen, Imperfect Naturalist is integrated with its1/1 black/green Worm token. Six focused scenarios pass0.45seconds,nine shared mill-selection regressions pass1.68seconds,and all-target Clippy passes. Tests pay every hybrid cast and activation route,include the discarded land in the resolution-time graveyard count,and check exact optional creature/land recovery,public top-card knowledge,private choice observations,stale/wrong-card rejection,source departure/control transfer,sickness/tap restrictions and opponent-turn zero-token resolution. Admission256/273;17 gated; token definitions110;observation schema155 adds LibraryTop to the mill destination domain.

The five Eclipsed creatures have complete Oracle/XMage/helper source audits. Their twelve existing interaction scenarios pass0.06seconds,including actual casts in both seats,every hybrid route,exact subtype unions,Kindred changeling,private choices,optional declines,revealed-hand knowledge,random bottoms and owned replay/restore. Typed behavior is preserved. Inherited audit25/107;82 remain.

Ajani, Outland Chaperone is integrated.139 white/blue scenarios pass3.40seconds,29 library-selection regressions pass0.08seconds,and library Clippy passes. Five whole-card scenarios cover all three paid loyalty abilities,own main/once-per-turn limits,tapped target revalidation,real Cat tokens,and an ultimate using current resolution-time life with private any-number mana-value-limited permanent selection. Aura/PW entries,source loyalty death,decline/no-eligible/zero-look shuffle,and countered ultimate are tested. Admission257/273;16 gated.

Bre of Clan Stoutarm is integrated. Six whole-card scenarios pass0.15seconds,17 cast-origin regressions pass0.41seconds,10 free-casting regressions pass0.54seconds,and all-target Clippy passes. Paid temporary flying/lifelink produces real combat life gain; target/sickness/tap restrictions,source departure and target reentry are tested. The own-end-step intervening condition gates exact exile-until-nonland,life-bounded optional casting during resolution,and publicly known hand fallback for declines/ineligible/unpayable spells. Adventure spell-face mana value,X=0,and all-land libraries are covered. Admission258/273;15 gated; observation schema156 adds bounded casting/fallback facts to the rare resolution view.

Feisty Spikeling,Mischievous Sneakling,and Prideful Feastling now have complete pinned source/helper audits and paid hybrid evidence in both seats,including actual opponent-turn flash and lifelink combat. Existing all-zone changeling,dynamic own-turn first strike,and damage suites add interaction evidence. Nine accumulated inherited scenarios pass0.35seconds. Inherited audit28/107;79 remain. Combined258-card release validation follows.

## Verified checkpoint258

Revision6cea982a passed all eight release phases in633.91seconds:3,024 native tests (one ignored),9 catalogue tests,all22 Foundations game suites and current ECL games,Clippy/provenance,release build and benchmark. Peak3.4GiB;1,122,861,056 completed disposable bytes cleaned. Required libraries remain at exact paths/hashes recorded in the JSON,including the retained249-card artifacts. Freeform regression49,832decisions/0.250810206seconds=198,684.10decisions/s;clone206.05ns,restore156.17ns;initial logical state21,080bytes. Full-pool ECL acceptance and79 inherited audits remain open. The initial formatting-only failed attempt is retained at<cache>/rustmage-validation-kbobg2xr/report/result.json; fixed input formatting is committed in6cea982a before this successful run.

Twinflame Travelers and Winnowing are integrated. Twinflame passes506 ECL scenarios in19.40seconds,228 mana regressions in7.50seconds,and all-target Clippy. Trigger multiplicity is frozen before departures and expanded after complete packet capture; intrinsic copied sacrifice abilities and triggered mana share the additive rule. A real paid G+RR regression fixes optimistic funding that omitted Growthhook bonuses. Winnowing passes six whole-card scenarios in0.22seconds,144 white/blue scenarios,14 effect-choice regressions,and Clippy. The caster makes non-targeting mandatory anchors for both players before one simultaneous sacrifice batch,including typeless creatures,changeling,current type changes,and persist with a departing grantor. Admission260/273;13 gated. Observation schema157 combines the public multiplicity rows and shared-type choice domain.

Boggart Cursecrafter,Hovel Hurler,Morcant's Loyalist,Noggle Robber,Stoic Grove-Guide,and Thoughtweft Lieutenant have complete Oracle/XMage/helper audits and paid body evidence across both seats and every hybrid route. Real Treasure entry/death and noncreature Kindred Elf/Kithkin boundaries supplement existing interaction suites. All eleven inherited scenarios pass0.50seconds. Audit34/107;73 remain.

Combined260-card checkpoint024ba6f8 passes538 ECL scenarios in20.76seconds;resource report4b9ffde0a41b43ff9bd7bb22b2a26b8f;peak2.7GiB.

Kirol, Attentive First-Year is integrated. Seven whole-card scenarios pass0.62seconds,22 stack-copy regressions pass0.89seconds,seven trigger-source-binding regressions pass0.05seconds,and all-target Clippy passes. Copies preserve the original triggered ability's physical source and frozen event parameters while receiving a fresh stack identity; no source object is allocated or moved. Tests cover both seats,every hybrid cast route,fresh-creature external tap costs,controlled-trigger-only targets,new targets,source departure,countered packets,intrinsic/delayed sacrifice triggers,and reuse on the opponent's next turn. Admission261/273;12 gated.

Morningtide's Light is integrated. Eight whole-card scenarios pass0.26seconds,152 white/blue scenarios pass4.24seconds,149 shared return regressions pass5.03seconds,and Clippy passes. Target creatures exile together and return tapped under owner control in one delayed group,including stale primary recipients and simultaneous entry choices. The independent player prevention effect expires at the caster's actual next turn and observes prevention ordering,unpreventable damage,combat,noncombat,life loss and other recipients. Casts during End wait for the following end step. Resolving copies never exile their original spell. Admission262/273;11 gated; observation schema158 adds shield state and grouped return payloads.

Nine inherited white/blue cards have complete pinned Oracle/XMage/helper audits and four paid whole-card scenario matrices passing0.54seconds,with Clippy clean: Burdened Stoneback,Moonlit Lamenter,Clachan Festival,Pestered Wellguard,Silvergill Peddler,Wanderbrine Preacher,Wanderwine Distracter,Tanufel Rimespeaker,and Unexpected Assistance. Tests include named counter removal,Kindred,repeat activations,five fresh creatures convoking with no mana payment,exact tap triggers and mana-value cast thresholds,source/control changes,and expiration. Audit43/107;64 remain.

Dawnhand Dissident is integrated after2,975 native regressions pass82.67seconds and13 focused scenarios pass0.42seconds,with all-target Clippy clean. Paid blight/tap activations cover surveil privacy and exact linked exile; owned completion of the additional remove-three-counter casting cost composes with printed mana,X,evoke and printed additional costs in either legal nonmana order. Joint mana/counter funding and freshly returned counter donors are tested. The frozen copied-ability epoch API preserves old queued packets while separating newly gained linked abilities;14 Dawnhand matches pass0.43seconds after that union.

Maralen is integrated. Nine whole-card scenarios pass0.29seconds,39 linked regressions pass4.31seconds,10 casting-permission regressions pass0.52seconds,and all-target Clippy passes. Explicit source selection records the approving ability instance until actual casting; once-per-player usage,turn-limited exact links,dynamic union counts including noncreature Kindred,chosen Adventure face mana values,X=0,mandatory additional costs,control transfer,source departure and simultaneous entry are covered. Old and newly copied linked abilities cannot consume each other's cards.

Shadow Urchin is integrated. Seven whole-card scenarios pass0.18seconds,132 shared exile regressions pass4.06seconds,five End-Blaze death-event regressions pass0.21seconds,and all-target Clippy passes. Attacks force blight even against counter-prohibited choices; death packets preserve the complete u128 counter total before simultaneous counter cancellation/source departure. Normal paid spell/land permissions expire before the captured player's next actual end step,including permissions created during an end step. Source/control/type/destination boundaries,large named counters,timing,land quota and exact incarnations are tested.

Admission265/273;158 newly tested candidates beyond inherited107;8 gated. Observation schema159 combines frozen copied ability origins,explicit linked casting approval/use and exile turn,DeathEvent counter totals,and next-end-step permission deadlines. The265-card combined release validation follows;full-pool games,64 inherited audits,and final published reviews remain open.

Verified checkpoint265: revision a9a47743 passes all eight release phases in640.02seconds,including3,094 native tests (one ignored),9 catalogue tests,22 Foundations game suites and current ECL games,Clippy/provenance,release build and benchmark. Peak3.5GiB;1,143,128,064 completed disposable bytes cleaned. Exact required library paths/hashes and prior258 evidence are retained in the validation JSON. Freeform regression49,832decisions/0.247475027seconds=201,361.73decisions/s;clone205.46ns,restore154.59ns;initial logical state21,096bytes. This remains a regression measurement;full-pool ECL games,64 inherited audits and final published reviews remain open.

Goliath Daydreamer is integrated. Eight whole-card scenarios pass0.46seconds,54 zone regressions pass2.97seconds,17 cast-origin regressions pass0.40seconds,13 named-counter regressions pass0.22seconds,and all-target Clippy passes. Only actual hand casts gain the successful-resolution dream replacement; countering,fizzling,normal self-exile,and competing Adventure/Omen/exile outcomes are distinct. Attack resolution offers one optional free cast from owned dream-counter exile cards,with mandatory additional costs and X=0. Multiple sources and an actual Kirol-copied trigger place only one dream counter. Admission266/273;7 gated;observation schema160 adds the dream counter,public resolution marks,competing replacement outcomes and a bounded resolution-cast count.

Spinerock Tyrant is integrated after3,086 native regressions pass91.20seconds (one ignored),eight focused whole-card scenarios,and all-target Clippy/provenance. An optional exactly-one-target-instance spell copy can choose new targets; accepting grants independent wither effects to the original and copy. Unrelated later copies do not inherit that grant. Actual paid6/6 flying/wither combat,decline,countered originals,departed sources,same-object multiple target instances,and capacity rollback are covered. Admission267/273;6 gated;observation schema161 adds the optional spell-copy choice. The combined cast-filter wrapper forwards both target and origin facts.

Taster of Wares is integrated. Eight both-seat whole-card scenarios pass0.50seconds,165 white/blue scenarios pass5.31seconds,15 exile-play regressions pass0.04seconds,110 funding regressions pass3.45seconds,and all-target Clippy passes. The opponent privately selects the required live-Goblin-count reveal before the caster selects the exact exile. Source-bound permissions expire permanently on control loss/departure but survive ability loss; captured any-mana intent remains valid when the source is sacrificed during casting. CastInfo remains40bytes through a compact origin-permissions enum,with truthful serialized stash_origin/exile_any_mana flags. Shadow next-end-step deadlines are retained. Admission268/273;5 gated;observation schema162 adds the reveal and source/mana permission fields.

Mirrorform is integrated with the physical-copy core. The complete candidate passes3,063 native regressions in89.79seconds (one ignored),81 copy scenarios and all-target Clippy. Real copy overlays preserve physical/token identity,native DFC shape,external state and ability epochs; copied permanents restore native identity after departure LKI capture. Generic spell copies enter as tokens without token-creation replacement/history,including Auras,planeswalkers,DFCs,and evoke choices. Undefined copied entry choices do not invent a mana restriction witness. Dream Harvest mana-value accumulation also reads the captured copy shape consistently. Admission269/273;4 gated;observation schema163 adds sparse copy origins and the frozen unspendable mana restriction.

The remaining17 inherited white/blue cards have complete pinned source/helper audits and11 both-seat scenario matrices passing0.77seconds,with all-target Clippy clean. All26 inherited white/blue cards are audited. Combined inherited audit60/107;47 remain (black/red25 and green/colorless22).

Lavaleaper is integrated after3,120 native regressions pass91.98seconds (one ignored),ten focused scenarios pass0.08seconds,and all-target Clippy/provenance pass. All creatures gain haste; actual basic-land tap production supplies one selected type per provider/trigger occurrence,including colorless and mixed-type choices. Bonus continuations wait for real cost/entry/replacement completion,and exact funding uses the same distributions. Hot State/Object/SpellView/Announcement/Resolution/CastInfo sizes are recorded. Admission270/273;3 gated.

The generic shared-type target-completion fix resolves the Unbury full-game dead end. Two both-seat scenarios and Clippy pass. A preliminary supported266-card pool workload then completes116/116 natural games across29 deck pairs,seeds0/7 and both starters:67,131decisions,1,231owned branches,125private decisions,53.822693seconds (debug,policy/observation/privacy/periodic JSON checks included)=1,247.262decisions/s. Actual execution includes89blights,187convoke selections,oneproliferation,20counter removals,tenstack copies andeighttransforms. This remains preliminary;final273-card and themed acceptance are open.

All107/107 inherited cards now have complete source/rules audits. The final black/red25 use13 paid both-seat scenario matrices passing0.40seconds plus retained counter,cast,damage,keyword,cycling,impulse,mill and vivid interaction suites; all-target Clippy passes. Fresh evidence includes optional/stale death targets,flash and lethal upkeep completion,Kindred tribal counts,actual Treasure mana,modal damage-source lifelink,target-invalid followup suppression,optional private discard and cleanup through actual priority passes. Three existing token factories gain full source pins. The final green/land22 use16 matrices passing0.86seconds with all-target Clippy,covering actual combat/damage,Aura LKI,batch entries,counter/vivid conditions,surveil/mill recovery,tap/sorcery costs,modes,shockland choices and private basic search. No inherited typed behavior is replaced. Whole-pool integrated games,performance,full release and final published reviews remain open.

Omni-Changeling is integrated after3,157 native regressions pass94.63seconds (one ignored),29 focused copy scenarios pass0.83seconds,and all-target Clippy passes. Its owned optional copy replacement precedes other entry choices and uses fresh copied counters,life,type and Aura requirements; newly acquired copy replacements may recur,with a later copy replacing earlier exceptions. Paid/convoke/free/spell-copy entry qualifications,nested token entry,native single-face DFC shape,private APNAP choices and actual Kindle exception replacement are tested. Resolution counts now distinguish acquired ability epochs while retaining old queued packets. A creature-Aura cannot remain attached under CR303.4c. Admission271/273;2 gated;observation schema164 adds pre-entry copy choices/views and resolution-history ability origins.

The transitive supporting-definition audit covers all19 required definitions:11 tokens,seven backs andone emblem. All96 unique source pins authenticate against the frozen XMage revision. Two missing token provenance packets are now added after full token/helper reads; seven direct identity/source checks pass6.29seconds. Existing parent rules evidence is mapped in the supporting-source audit JSON. No supporting behavior gaps remain; this audit does not replace the two final independent published-head reviews.

Twilight Diviner is integrated. Eleven focused whole-card scenarios pass0.31seconds,3,182 native regressions pass95.59seconds (one ignored),and all-target Clippy/provenance pass. Qualifying simultaneous entries freeze exact other-creature references and actual graveyard cast origins; spell copies do not become casts. Non-targeting copy choice uses surviving references even after type/control changes. Once-per-turn intrinsic history includes acquired ability epochs while retaining queued old packets. Hot State/Object/SpellView/Announcement/Resolution layouts remain unchanged. Admission272/273;only Raiding Schemes remains gated;observation schema165 adds entry-batch packets,graveyard origins and trigger-use epochs.

Raiding Schemes is integrated, completing273/273 physical admission with zero unresolved or gated members. Fourteen whole-card scenarios pass1.79seconds,all716 combined ECL scenarios pass30.68seconds,and all-target Clippy passes41.25seconds. Optional independent conspire declarations and simultaneous color-sharing tap pairs use exact owned joint completion through mana/convoke/printed costs and entry choices. A bounded local integration review found premature pruning of an incoming Omni donor; the settled-entry guard and paid regression now pass. Captured intrinsic source-spell triggers support countered originals,Kirol copies,and permanent/Aura spell copies. Observation schema166 adds the conspire declaration/payment views/actions and intrinsic stack kind. Measured focused implementation interval:166 new whole-card candidates over26,826.03seconds=22.28cards/hour;inherited audits and final acceptance/review remain separately reported.

The full-game performance diagnosis is fixed: a purpose-eligible conservative quantity bound rejects impossible restricted-mana plans without exploring permutations of unusable mana or exclusive land activations. All114 funding regressions,including an actual paid restricted-mana converter in both seats,and six Growth mana-bonus regressions pass; all-target Clippy passes. Exact slow-prefix legal-action/observation calls now take roughly0.5–0.6milliseconds in debug. These are targeted timings,not full-set simulation measurements. Two terminated diagnostic attempts are explicitly retained as incomplete results. Their systemd stop exposed a runner status bug: missing command completion reports now fail closed,verified with four mocked missing/partial/completed/failed report cases without any allocation/kill probe. A separate copied-source linked-return funding legality gap remains under active repair before final acceptance.

The first strict273-card/220-game development attempt remains incomplete. All pool pairings and the Twilight theme progressed through natural outcomes,then a later mana-heavy position exposed further expensive finite searches. It was deliberately stopped for exact snapshots;the corrected runner returns1 and preserves report10f0843a3ff94beb9c893ca1d7e81046. This is diagnostic evidence,not an acceptance pass;decks,seeds and assertions remain unchanged for the next attempt.

The second funding repair also bounds eligible colors,not only total quantity: impossible Kinbinding double-white payments no longer count restricted Flamebraider white. All116 funding regressions,including actual double-white converter payment,pass3.89seconds;Clippy passes41.35seconds. Exact prefix438 now generates111 actions in0.288milliseconds and its observation in0.320milliseconds. The independent Conspire cycle audit also found that repeated Omni replacements advance fresh epochs indefinitely. An analysis-only alpha-renamed key now detects equivalent uncommitted prototypes while retaining committed history and timestamp-capacity dominance. Fourteen Conspire and31 copy-entry scenarios plus Clippy pass;the public regression repeats the same Omni choice twice with fresh real epochs,then completes the paid donor path. No gameplay replacement limit is imposed.
