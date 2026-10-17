# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,895 executable Standard identities | 3,269 identities; many reusable mechanics plus card-specific rules |
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
