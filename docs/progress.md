# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,813 executable Standard identities | 3,351 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Isolated COW object/library buffers, owned continuations, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding; optimize atomic checkpoint overhead |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | Saga PR #259 has two independent nonauthor approvals of the complete authored batch and durable scoped evidence in the [Saga review record](review-pr-259.md). Crew PR #253 and earlier milestones retain their own feature/review records. | Continue issue-managed, reviewed PRs through complete acceptance |

The previously merged composition added nine complete [Omen pairs](omens.md) to reviewed Adventure/Saga/attachment-Ward main. Its coverage was 1,810/5,164, with 3,354 gated, 87 token recipes, one emblem and observation schema 79. All1,668 native tests and all-target Clippy pass. All356 unique compiler methods pass863.099s; all236 unique fresh public methods pass1197.240s. Four same-source Standard100 workloads finish naturally with complete body vectors,585,235 parent-closed owned frames and1,170,470 denials. Exact source/library identities, preservation, provenance and independent review status are in the [Omen composition record](review-pr-263.md).

The other 56 true Adventure parents, three Omen parents and54 Saga parents
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

The Feral stage adds three complete single-graveyard bodies to reviewed Omen/Saga/Ward main: 1,813/5,164 Standard identities, 3,351 gated, 87 token factories, one emblem, nineteen Adventure helpers, ten Omen helpers and schema 79. Typed per-occurrence graveyard constraints use existing owned bindings and add no mutable relation cache. Full 1,675 native,363 compiler,239 unique public and Standard100 gates pass on the precisely scoped frozen composition; final compiler-only hardening preserves all 5,300 generated programs and has fresh focused compiler/paid acceptance. A second preserved-fingerprint Standard100 also passes 89,944 owned frames and 179,888 denials. Exact artifacts, timings, independent review status and earlier standalone scope are in the [PR267 record](review-pr-267.md).
