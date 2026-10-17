# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,727 executable Standard identities | 3,437 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Isolated COW object/library buffers, owned continuations, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding; optimize atomic checkpoint overhead |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | Reviewed main through Job Select PR #246; Warp PR #247 has two independent nonauthor reviews, exact combined validation and a durable [review record](review-pr-247.md). Earlier milestones remain documented in their feature/review pages. | Continue issue-managed, reviewed PRs through complete acceptance |

The latest composed work includes sixteen complete [Job Select](job-select.md)
bodies, fourteen [Warp/shared-rule](warp.md) bodies and twenty
[actual-payment/Opus/Increment/Converge](cast-payments.md) bodies. Coverage is
1,727/5,164, with 83 token factories, one emblem and observation schema 74.
The complete native suite passes 1,599 tests; clean full compiler/data and
fresh-library public API collections pass 324 and 221 methods respectively.
The payment Standard100 run witnesses every card in both seats at low/high
payments, natural game outcomes and 104,413 owned decision replays. Initial
logical state is 20,280 bytes; maximum is 23,552 in that workload. These are
supported-card workloads, not proof of complete Standard rules or full-deck
performance. Full validation scopes and pending exact-head review are in
[the payment review](review-pr-251.md).

Parallel [owned token/counter replacements (#248)](https://github.com/altf4/rustmage/issues/248)
and [Crew/Vehicle rules (#250)](https://github.com/altf4/rustmage/issues/250)
have acceptance checkpoints awaiting composition and independent final reviews.
Their nine and twenty-four additional bodies are not included in the count above.
They must preserve affected-player choices, legal payment domains, private owned
continuations and whole-card admissions rather than narrowing the full goal.

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
