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

The frozen manifest accounts for every identity. The [per-card obligation ledger](ecl-obligations.json) records every face and printed rules line with canonical ownership. Twenty inherited cards have passed source/rules audit;87 inherited cards remain.

## Current implementation checkpoint (2026-10-22)

Admission flags: **249/273**, comprising **107 inherited** and **142 newly
implemented/tested physical candidates**; **24 remain gated**. Twenty inherited
cards have passed source/rules audit;87 inherited cards remain. No final published reviews or newly merged cards. All new
names and per-card obligations appear in the linked JSON ledgers. Three tested
synthetic token recipes are outside the physical denominator.

Three isolated authors own exactly black/red, white/blue, and green/colorless
cards respectively. Their focused modules are `src/engine_ecl_black_red_tests.rs`,
`src/engine_ecl_white_blue_tests.rs`, and `src/engine_ecl_green_colorless_tests.rs`.
The integrator owns multicolor cards, shared coordination, source reconciliation,
`src/engine_ecl_core_tests.rs`, `src/engine_ecl_commands_tests.rs`, and generated
links. All five Incarnations are integrated; root now owns the seven transforming cards. Foraging Wickermaw, Harmonized
Crescendo, Thoughtweft Imbuer and Lasting Tarfire are integrated; the203-card
Clippy check needed the now integrated Brigid consumer. Both Brigid faces,
Warren Torchmaster, Shimmerwilds Growth and Kinbinding are integrated; a207-card
release validation reached2,737 passing native tests and two old single-face
invariant failures, now corrected and rechecked (18/18). Ashling,
Flamebraider, Flitterwing Nuisance, Lys Alana Dignitary, Retched Wretch and
Rimefire Torque are integrated. The213-card run passed all2,768 native tests;
the remaining transform-name catalogue invariant is fixed and all9 catalogue
tests pass. Subsequent integrated cards include both Trystan faces, Champions of
the Perfect, Mudbutton Cursetosser, Soulbright Seeker, Moonshadow, Temporal
Cleansing and Swat Away. Scarblade’s Malice is integrated, bringing admission to221/273. The220-card attempts found a malformed source pin and an exhaustive example match; both are fixed. The221-card run passed2,803 native tests and found one outdated Moonshadow source-index assertion, corrected in12633035. Celestial Reunion and four white/blue Behold creatures are integrated:226/273 admitted,47 gated. Champion of the Path and Champion of the Weird are integrated:228/273 admitted,45 gated. The226-card checks caught a portable-report-path error and an exhaustive example search-filter match; both are corrected. Fresh combined release validation follows.

Shared mechanics include mandatory and optional blight costs versus effects,
wither as a damage result, exact mana-funding tap witnesses, entry-time creature
history, chosen-type entry and activation packets/restricted mana/lords,
conditional flash, all five Commands and their six mode pairs, immediate blink
through owned entry continuations, private filtered optional discard,
attached-creature fight, non-target power-difference choices, and conditional
combat assignment using an equipped creature's toughness, evoke with actual mana-spent entry qualification, all-types temporary effects, vivid search/reveal quantities, exact combat-event subjects, owned layer-five color changes, creature trigger grants, and counter/untap prohibitions. Alternative casting
methods preserve printed additional mana-or-blight choices.

## Validation and measurements

The combined focused run at `bb92026c` passed **196 ECL scenarios** in7.24
seconds after linking Doran, Puca’s Eye, Hexing Squelcher, and Blossombind.
Full release validation **passed all eight phases** at `0ae35121`: formatting,
Python/provenance checks, Clippy/all-target compilation, **2,671 native tests**
(one ignored), catalogue checks, **all 22 Foundations game suites**, ECL games,
release build and benchmark. Runtime531.22 seconds; enforced shared memory peak
3.2 GiB. Report `<cache>/rustmage-validation-4cepq26r/report/result.json` records
all required libraries at exact paths/hashes; completed disposable outputs were
cleaned. The prior unrelated-attachment timestamp regression also passed its
exact targeted full-game rerun (56.22 seconds).

The release Freeform regression workload ran100 deterministic games and49,832
decisions in0.248807 seconds: **200,283.61 decisions/second**,401.92games/second;
clone208.32ns, restore181.27ns. It includes native policy/observations, excludes
JSON/model/tensor work, and is not an ECL acceptance measurement. Compiler,
hardware, revision and workload context are recorded in the validation JSON.

Ten natural ECL-only Standard games covered all colors, ten terminal outcomes,
ten blight choices, six private choices, and 100 owned raw/index branch/replay/
restore checkpoints. Debug workload: **5,719 decisions in 1.777491 seconds**,
**3,217.455 decisions/second**, including policy, observations, assertions, and
snapshots. This is subset regression evidence, not complete-set acceptance.
See the JSON for seeds, revision, compiler, hardware, and artifact reports.

The validation JSON records the measured implementation interval and current cards/hour for121 newly locally tested physical candidates. This excludes inherited certification, independent final review, and upstream merge.

## Resume and remaining work

Branch `codex/ecl-complete-set`; issue [#462](https://github.com/altf4/rustmage/issues/462).
Authenticate membership using `python3 -m scripts.set_status ECL --check`.
Finish current shared-mechanic batches, integrate, regenerate links, run combined
focused scenarios and full release validation, then extend the natural games to
verify every printed obligation. Check active jobs before touching their inputs;
poll the same process until terminal. No early or partial implementation PR.

Ordinary engineering remains for gated cards, including transform,
behold/champion, conspire, proliferate, and some copying/chosen-type effects.
These are implementation work, not external permission blockers. Authors are
working on remaining combat/type interactions, vivid consumers, resolution hand exile, and behold/champion. Full pool behavior, interactions,
performance, two independent nonauthor final published-head reviews and merge
remain open.

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
