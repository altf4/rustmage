# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,974 executable Standard identities | 3,190 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Isolated COW object/library buffers, owned continuations, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding; optimize atomic checkpoint overhead |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | Saga PR #259 has two independent nonauthor approvals of the complete authored batch and durable scoped evidence in the [Saga review record](review-pr-259.md). Crew PR #253 and earlier milestones retain their own feature/review records. | Continue issue-managed, reviewed PRs through complete acceptance |

The merged Saga/Adventure/Ward baseline adds nineteen complete [Adventure pairs](adventure.md)
and ten complete [attachment/Ward bodies](attachment-ward.md) to merged Saga
main. Coverage is 1,801/5,164, with 3,363 identities still gated, 86 token factories,
one emblem and observation schema 78. All 1,659 combined native tests and all-target
Clippy pass. All three same-source Standard100 workloads finish naturally with
complete body vectors, 496,285 owned replay frames and 992,570 denials. The
complete 351 compiler/data suite passes in 476.874 seconds; all 233 unique
fresh public methods pass in 712.402 seconds. The
[composition record](review-pr-258.md) tracks exact source/library identities,
canonical preservation, meaningful cross-mechanic cases and current gate status.

The Omen parent integrates all thirteen complete Omen pairs with the three
adjacent graveyard/search consumers: 1,817/5,164 identities, 3,347 gated,
87 token recipes, one emblem, nineteen Adventure and thirteen Omen helpers,
observation schema80. The union retains chosen-spell timing, selected-face X,
same-graveyard target constraints and mandatory split-search destinations after
revealing only the selected cards.
The individual kernel records scope their frozen source evidence. Full union
validation passes: 1,703 native tests, 383 compiler/provenance methods, all 245
unique fresh public API methods and four Standard100 workloads with 366,535
owned replays / 733,070 denials. The [union review record](review-pr-275.md)
identifies the exact source/library, preservation proof and independent reviews.
Final exact-head approvals are recorded in the PR comments before merge.

Other Adventure and Saga bodies
remain part of the full goal, alongside wider Ward/Role/disguise/cloak/control
and other required semantics. Full rules enforcement across Standard, efficient
training interfaces and full-deck performance remain incomplete.

Other dependencies include wider costs/modes/events, multiple distinct target
groups, triggered/restricted/variable mana, infinite-resource handling and
replacement/prevention/layer machinery.
Remaining card-specific mechanics and native RL/search integration still need full
implementations. These priorities do not reduce the all-Standard goal.

Completion remains unproven and the goal remains active. Final acceptance must
audit every supported card/required mechanic, illegal-action paths, every pending
choice's restore/replay, hidden information, native/Python integration and full
workload performance. A zero unsupported-card counter alone will not prove rules
correctness.

The [eight-pair Adventure candidate](adventure-land-targets.md) adds accumulating turn-bound land allowances and complete typed target movement/protection variants. Its combined coverage is 1,825/5,164, with 3,339 still gated. Validation passes: 1,710 native tests, all-target Clippy, all 388 unique compiler/data/provenance checks, 247 unique fresh public API methods and 100 natural Standard games with 79,608 owned frames / 159,216 denials. Two independent nonauthor final-head reviews remain required.

The [ownership/controller candidate](ownership-control.md) composes that merged Adventure delivery and completes Korvold and the Noble Thief. Coverage is 1,826/5,164 with 3,338 gated, all 27 Adventure and 13 Omen pairs retained, and observation schema82. Frozen runtime validation passes all 1,724 native tests, all-target Clippy, 403 unique compiler/provenance methods and 249 unique public checkpoint methods. The two subsequently revised policy methods pass from an isolated final export, as do 100 natural Standard games with all sixteen inset route/seat vectors, 80,698 owned replays and 161,396 per-decision denials. Exact source/library and policy scopes, plus required independent nonauthor reviews, are tracked in the [PR 284 review record](review-pr-284.md). This initial stage leaves issue #265 open for broader changing-control bodies and layers.

The [Role composition](role-auras.md) brings reviewed candidate coverage to
1,862/5,164 (3,302 gated), with 37 Adventure and 13 Omen helpers, 94 token factories
and observation schema85. All 1,748 native tests and Clippy pass. The frozen
420-method compiler/provenance checkpoint and 254-method public checkpoint pass;
the subsequently added Tombstone payment method passes separately on identical
production. One hundred natural Standard games cover all 24 Role body/seat
vectors with 81,882 owned frames and 163,764 denials. The
[PR289 record](review-pr-289.md) keeps exact source scopes and final review gates
explicit. The full all-Standard, rules and training goal remains incomplete.

The [exact source-binding delivery](trigger-source-binding.md) completes Morcant's
Loyalist on actual merged Role main, bringing coverage to 1,863/5,164 (3,301 gated)
and schema 86. All 1,755 native tests, all-target Clippy and 257 unique fresh public
methods pass. The 423-method compiler collection executes 421 passing methods and
skips two opt-in provenance methods; both then execute/pass with the pinned Java
source explicitly enabled, completing all 423 unique methods across those runs.
Six actual paid Morcant cases cover both seats with Elf/Changeling recovery or no
other Elf, 5,097 owned decisions and 10,194 actor/revision denials. The new inherited
Tombstone payment method passes separately on identical production. Exact
checkpoint/export/library identities and required independent reviews remain in
the [PR 293 record](review-pr-293.md). This does not complete the all-Standard goal.


The [cast and declaration event delivery](adventure-event-predicates.md) completes
three further whole Adventure pairs on actual merged source-binding main:
1,866/5,164 executable Standard identities (3,298 gated), 40 Adventure/13 Omen,
94 token recipes and schema86. Frozen corrected source passes 1,762 native tests,
all-target Clippy, all 430 unique compiler/provenance methods, all 261 unique
fresh public API methods and 100 natural paid Standard games with actual normal
events, 81,129 owned frames and 162,258 actor/revision denials. Exact parent union
and ten-manifest pinned provenance proofs pass. Two independent reviewers approve
production; exact final published-head confirmation remains the merge gate.
The full all-Standard and training goal remains incomplete.

The [enchanted draw-step delivery](enchanted-draw.md) completes Lord Skitter's
Blessing on actual Adventure main804f539: 1,867/5,164 executable Standard
identities, 3,297 gated and schema86. Exact current-parent comparison retains
5,330 prior definitions and all helpers, aliases, colors, recipes and flags.
Fresh composition validation passes 1,766 native tests, all-target Clippy and
formatting, 24 focused compiler/provenance methods, five unique public methods
and both 100-game workloads: 153,682 owned frames and 307,364 denials combined.
The separately frozen Blessing baseline passes all427 compiler and260 public
methods; the actual Adventure parent passes430 compiler and261 public methods.
Those broad runs retain their original source scopes. Two independent nonauthor
final published-head approvals remain the delivery gate. Full Standard support
and the complete training/search goal remain incomplete.


The [complete Role consumer follow-up](role-followup.md) adds twelve whole bodies
on actual enchanted-draw main c41c060: 1,879/5,164 executable Standard identities,
3,285 gated, 40 Adventure/13 Omen, 94 token recipes and schema86. The exact-parent
proof preserves 5,319 prior programs and all prior helpers/colors/recipes; only
the twelve intended bodies and Diminisher's zero-reduction Bargain arm change.
Frozen corrected source passes all 1,775 native tests, all-target Clippy, all 441
unique compiler/provenance methods and all 266 unique fresh public API methods,
with no failures, errors or skips. One hundred natural paid Standard games
produce 82,514 owned frames and 165,028 actor/revision denials. Both independent
reviewers cleared corrected production using their own raw and adverse gameplay
controls; exact final published-head confirmations remain the delivery gate.
The full Standard and training/search goal remains incomplete.

The [target exclusions and two-mode combinations](target-exclusions.md) add seven
complete bodies on actual Role main ce4b0c1: 1,886/5,164 Standard identities,
3,278 gated, schema86. Exact-parent comparison retains all 5,324 other definitions
and all helpers/colors/aliases/recipes/flags. Fresh composed validation passes
1,781 native tests, all-target Clippy, 27 affected compiler/provenance methods,
five unique public methods and two 100-game workloads with 172,449 owned frames
and 344,898 actor/revision denials. Earlier frozen branches retain their own full
439/263 and 441/266 compiler/public scope; these are not fresh composed broad runs.
Two exact final-head independent nonauthor confirmations are the delivery gate.

The [Role move-result and paid-flash batch](role-semantic-kernels.md) completes
six bodies on actual main8108: 1,892/5,164 Standard identities, 3,272 gated,
schema87. Strict comparisons preserve all 5,325 other main definitions and every
parent helper/color/recipe/alias/flag. Frozen c5af333 passes 1,784 native tests,
all-target Clippy, 448 unique compiler/provenance methods and 268 fresh API
methods with no skips; 100 natural Standard games produce 84,835 owned frames
and 169,670 actor/revision denials. Later actual-main composition3e681b6 passes
fresh 1,790 native tests, Clippy, 36 affected compiler methods and 42 paid natural
Standard cases, retaining all authored six and seven parent programs exactly.
The broad 448/268 and 100-game evidence keeps its c5 source scope. Two independent
NONAUTHOR final-head confirmations remain required before merge; full Standard
and the wider training/search goal remain incomplete.


The [temporary creature death grants](temporary-death-grants.md) add three whole
spells on actual merged main 16f0604: 1,895/5,164 Standard identities, 3,269 gated,
schema 88. Exact-parent comparison preserves 5,328 other definitions and all 882
helpers/colors/aliases/recipes/flags. Frozen ecedec2 passes 1,804 native tests,
Clippy/formatting and 100 natural Standard games: 79,647 owned frames and 159,294
actor/revision denials. Its full 460 compiler run has 459 passes and one stale
inventory failure; its full 271 public run has 268 passing methods and three
stale-schema methods (49 subtest failures). These broad runs are not relabeled
green. After metadata-only correction and exact production/generator/card-data
identity proof, fresh 10 affected compiler methods and four public methods
(50 inherited Role cases plus all three grant spells in both seats) pass without
errors or skips. Policy logic is unchanged except explicit schema assertion
metadata. Both independent reviewers clear production; exact final-head
confirmations remain required before merge. Five further inventoried death-grant
bodies, the rest of Standard and the wider training/search goal remain incomplete.


PR #305 completes Curse of the Werefox on actual merged main `e5ebe649`:
1,896/5,164 Standard identities execute, 3,268 remain gated, schema89. Exact
preservation changes only that complete body, retaining 5,330 definitions and
all 882 existing helpers, colors, flags, aliases and recipes. Its sparse owned
Role-success continuation captures the original creature without retargeting;
the opponent is chosen at the separate reflexive announcement. Doubled Roles
queue one event; replacement-added Frogs do not qualify. The combined API exposes
exact Role recipient bindings and leaves extra Frog groups unbound.

Frozen composition `c865dcb` passes 1,802 unit plus 12 integration tests,
all-target Clippy, formatting/generation, 36 affected compiler/provenance methods,
and eleven fresh paid own/parent public methods (206.688 seconds). All 100 fresh
fixed natural Standard games finish with 81,976 owned frames and 163,952 denials
in 176.758 seconds; all 838 inputs stay fixed. Library SHA256 is
`9fbc0236ba5352666b1c7f88f061ede2819a6cd5b73dc85e9c037673c56b4a9e`.
Earlier full 453-method compiler and 270-method API gates passed at frozen
`2cdb597`; their original source scope is retained, not relabeled as fresh
composition runs. Initial new integration fixture failures and their test-only
corrections remain recorded in [the delivery record](role-success-fight.md).
Two independent NONAUTHOR exact-final-head approvals remain the merge gate.
Full Standard/training support remains unfinished.

The [simultaneous return kernel](battlefield-returns.md) composes actual Werefox main and admits complete Aetherize and River’s Rebuke: 1,898/5,164 identities, 3,266 gated, schema89. Fresh composed validation passes 1,819 native tests, all-target Clippy/fmt, thirty affected compiler methods, six paid parent/own API methods and 100 unretried natural Standard games (82,844 owned frames / 165,688 denials). The full original 452 compiler/269 API suites retain their original baseline scope. The feature record documents the historical-preservation test correction, exact source/library identities and pending independent final-head approvals.

The [continuous combat restrictions](combat-restrictions.md) add five complete bodies on actual return main: 1,903/5,164 executable identities, 3,261 gated, schema89. The frozen composition passes 1,824 native tests, all-target Clippy/fmt, 35 affected compiler methods, seven paid own/parent API methods and 100 unretried naturally terminal Standard games (93,895 owned frames / 187,790 denials). Exact source/library identities, earlier failed policy workloads, generic raw guards, strict parent preservation and required independent final-head approvals are recorded in the feature document.


The [temporary would-die exile replacement batch](death-exile-plan.md) adds eleven
whole bodies on actual main5d103e7: 1,914/5,164 executable Standard identities,
3,250 gated and observation schema92. The exact-parent proof preserves all 5,320
other definitions, all 888 prior helper arms, colors, aliases and 94 token recipes.
Frozen runtime validation passes 1,836 native tests, all-target Clippy, formatting,
276 unique fresh public API methods and 100 natural Standard games with 73,375
owned frames and 146,750 wrong-actor/stale-revision denials. Thirty paid natural
Standard cases cover every mode vector in both seats. The full 483-method compiler
run has 480 passes and three inherited stale PR309 fixture failures; their
assertions are corrected separately; all 276 fresh affected compiler methods
pass in 537.037 seconds. These failed historical assertions are not relabeled as a
passing full compiler run. Seven complete-body dependencies remain gated and
tracked in [issue323](https://github.com/altf4/rustmage/issues/323). Two independent
NONAUTHOR exact-final-head approvals are required before merge. The all-Standard
rules and training goal remains incomplete.


## Shared Gift delivery — PR #312 / issue #310

Seventeen complete Gift bodies compose onto actual main
`8dd395286661963d1d1d98a2d384f2f5ab62d620`: 1,931 executable Standard identities,
3,233 gated, schema94, 94 token recipes and one emblem. The
[Gift record](gift-kernels.md) describes promise-before-target legality, all four
gifts, exact cast/entry provenance, gift-first resolution, owner spell returns,
protection and casting prohibition, pinned 23-card Java audit and six concrete
remaining whole-body dependencies. Strict preservation retains 5,314 unrelated
definitions and all 888 parent helpers.

Frozen composed source `e484dbcb787f57c24a10a309c0ad35f84f10fd85` passes all
1,848 native tests, all-target Clippy, formatting, generator verification, 50
affected compiler methods (23.283s), and five unique public methods (210.011s,
68 Gift cases, 30 death-exile/mode cases, 24 Role cases and inset alias guards),
without skips. Fresh composed Standard100 finishes naturally: 74,676 owned
frames, 149,452 denials, 155.576s. All 866 frozen inputs and the exact schema94
`87d27d0258d2` immutable library hash are verified. Source721's native failure
was solely a new fixture expecting a card instead of Blooming Blast's printed
Treasure; the correction changes only that fixture, with production/generator/
data/policies identical. The fresh full native suite proves the corrected case.

Historical schema90 breadth remains scoped to source28 on actual5d:
277 unique public methods pass with no skips; the full480 compiler run remains
477 passes and three stale-fixture failures. Exactly those three catalogue
expectations were corrected after all jobs ended, then their affected checks and
the composed 50-method gate pass. No historical broad run is relabeled as a
current-parent full run. Two independent whole-batch preliminary reviews find
no issues; final exact-head confirmations are required before merge. All jobs
are terminal, completed outputs cleaned, shared libraries retained at their
original paths/hashes, and frozen source exports hash-verified into recoverable
archives. Full Standard remains unfinished.


The [temporary death consumers and Suspect batch](temporary-death-consumers.md)
composes actual Gift/death-exile main02d8352 and adds nine complete bodies:
1,940/5,164 executable Standard identities, 3,224 gated, schema95. Strict union
preserves 5,322 other definitions and all906 parent helpers/old94 recipes, adding
only the declared Detective factory and two helpers. All nine complete programs
remain identical to the initial reviewed471 delivery; historical/current Gift,
death-exile, combat and own sentinels retain exact independent scopes.

Fresh composed production4a8 passes1,862 native tests, all-target Clippy/fmt,
generator check and43 affected compiler methods. Final policy5a5 differs only in
two Rabid readiness lines, with every other tracked byte identical; its20 paid
natural Standard cases and entire unchanged fixed100 pass (90,675 owned replay
frames/181,350 denials). The feature record preserves both historical failed100
workloads, precise policy diagnostics, old481 compiler475/6fixture outcome and
fresh affected corrections. Original277 API scope remains schema91/471, not a
fresh schema95 broad run. All jobs are terminal and completed build non-SO outputs
are cleaned with every shared library retained. Two independent NONAUTHOR exact
final-head reviews are required before merge. Fifteen inventoried full consumers,
including complete Tiered Vincent, remain gated; issue311 and full Standard stay open.


## Timestamped base-stat layers — PR321

This delivery adds ten complete bodies to the actual merged Suspect/Gift parent,
bringing executable coverage to1,950/5,164, with3,214 gated,95 token recipes and
observation schema96. Shared sparse exact-incarnation effects enforce7a→7b→7c,
Cursed Role timestamps, live static predicates, fixed resolution recipients and
cleanup/departure. Owned branches, replay, restore and public projections remain
independent. Full56-card source/dependency inventory remains in the audit;48
remaining base-stat bodies are tracked in issue324, and full Standard is unfinished.

Composedf373 evidence:1870 native tests/Clippy/fmt/generator,50 affected compiler
methods,30 paid natural cases, three corrected split-search API methods and the
complete fixed100 workload all pass. The100 workload has81,625 owned replay frames
and163,250 denials. New paid cross-rule cases cover zero-toughness state actions
using death-exile replacement before death-return, and7-power dying LKI with
owner-return/Suspect clearing old base layers. Final actual-parent preservation is
checked separately. See base-stat-layers.md for exact source/library hashes and
the historical failed schema assertions and missing-Git harness scope.

## Private land placement — PR327

Actual schema96 main19e gains exactly five complete bodies: Embrace the Paradox,
H.E.R.B.I.E. Scout Unit, all three Insidious Fungus modes, Lessons from Life and
both Nicanzil explored-kind clauses. Current coverage is1,955/5,164,3,209 gated,
schema97/95 token recipes. Strict preservation retains5,327 other definitions
and all911 prior helpers. The complete12-body/86-Java-file audit retains seven
unfinished full bodies under issue330; full Standard remains incomplete.

Runtime71d passes1,881 native tests/all-target Clippy/fmt/gen; immutable60a69
retains its exact path/hash. Corrected policy95fed changes no runtime/generator/
card data:37 affected compiler/provenance methods, all24 paid cases and all100
fixed games pass (68,728 owned frames/137,456 denials). Seven parent/private-hand/
modal/schema-routing public methods also pass at71d. The failed historical497
compiler run and initial68-game workload retain their precise scopes; neither
is relabelled green. [The delivery record](hand-land-placement.md) identifies
all source/library identities, corrected fixtures/policy and remaining guards.
Both introduced activated and inherited triggered modal string-marker forgeries
are fixed with parser-owned bindings. An independently found pre-existing generic
triggered-modal reminder gap remains a separate issue261 obligation.


## Resolved signed base-stat expressions — PR328

Actual schema97 maina938 gains five complete bodies: Genemorph Imago, Moon Girl
and Devil Dinosaur, Belligerent Yearling, PuPu UFO and Atomic Microsizer. Coverage
is1,960/5,164 (3,204 gated), schema98/95 recipes. Generic signed fixed7b snapshots
read current controller resources or exact subject current/departure power; real
hand-land entry/equip/optional targets reuse the existing kernels. Strict5 changes
preserve5,327 definitions/all911 helpers and add only Atomic's equipment helper.
Five full Java bodies/direct imports pin144 records/105 unique files. Base-family
remaining43 and hand-land remaining6 retain whole-body gates; full Standard remains
incomplete. Audit metadata is never permission input.

Production3943 passes1,887native/Clippy/fmt/gen, immutable396f schema98 stays exact.
Policy13cdb changes no production; all12 natural paid vectors pass (9,488 owned
frames/18,976 denials),100fixed games pass (77,531owned/155,062denials),22affected
compiler methods pass and22rerun after metadata-only audit corrections pass.
Every896-file manifest/library is fixed, all jobs closed and terminal non-SO builds
cleaned. Earlier prototype and new fixture failures retain their actual failed
scopes in [the complete delivery record](resolved-base-stat-expressions.md).
Two independent NONAUTHOR exact-final-head reviews remain required before merge.

The [Tiered delivery](tiered-costs.md) composes actual schema98 main and adds all six complete Tiered bodies/eighteen mutually exclusive modes, plus complete Boros Charm and Epic Fight. Coverage is1,968/5,164 (3,196 gated),95 token recipes and schema99. The strict union preserves all5,324 other definitions, all912 inherited helpers and every color/alias/inline flag/token recipe; only six Tiered-cost helper arms are added. Final validation passes1,900 native tests, all-target Clippy/fmt/generator,40 affected compiler/provenance methods,50 paid natural Standard vectors in three public methods (91.757s) and100 fixed unretried Standard games (177.315s;81,365 owned frames/162,730 denials). Exact production/library identities, historical prototype failures and final independent-review requirements remain explicit in the feature record. This does not complete the full Standard goal.


## Live static stat effects — PR #332 / issue #331

Four complete bodies compose actual Tiered main: Maha, Its Feathers Night;
Archon of the Wild Rose; Aettir and Priwen; Burden of Proof. Coverage is1,972/5,164
(3,192 gated), schema100. Sparse owned state evaluates current controllers/life/
Aura membership, independent7b axes,7c bonuses and real blocking restrictions;
simultaneous sources receive controller-chosen APNAP timestamps. Strict preservation
retains5,328 unrelated definitions,918 inherited helpers and95 token recipes.
Final frozen production ca735501 passes1,911 native tests, Clippy/fmt/generator,
45 affected compiler/provenance methods,12 paid natural Standard vectors20.167s
and100 fixed unretried Standard games186.170s (84,430 owned frames/168,860 denials).
All910 execution inputs and the optimized library hash remain unchanged. Independent
whole-four review and exact-final-head approval gate merging; the full source and
historical failure scopes remain in [the delivery record](live-base-stat-effects.md).


## Joint hand/graveyard land selection — PR #339 / issue #336

Complete Worldsoul’s Rage and Bonny Pall, Clearcutter compose actual schema100
main, bringing coverage to1,974/5,164 (3,190 gated),96 token recipes and schema101.
Owned optional choices hide hand identities, validate exact references and stage
all selected lands into one replacement-aware entry batch without consuming the
normal land quota. Beau uses live controller-relative layer7a characteristics and
the printed token name for the legend rule. The delivery record explains the
intentional correction of Java’s layer7b assignment against Oracle/CR604.3a.
Strict preservation retains5,330 other definitions,924 inherited helpers and all95
old recipes, adding only Beau and the two owned helpers. The complete six-body
audit leaves four other bodies gated and tracked.

Frozen production passes1,918 native tests, all-target Clippy/fmt/generator and
41 affected compiler/provenance methods. Final public policy passes14 paid vectors;
100 fixed unretried natural games pass117.008s with60,866 owned frames/121,732
denials. Four parent methods pass166.203s across88 paid cases. All918 final inputs
and the immutable library remain fixed; policy-only setup corrections and earlier
failed scopes remain explicit in [the delivery record](joint-land-selection.md).
All jobs are closed and non-SO build outputs cleaned. Two independent exact-final
reviews gate merging; the full Standard migration remains incomplete.
