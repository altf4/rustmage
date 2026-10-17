# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,736 executable Standard identities | 3,428 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Isolated COW object/library buffers, owned continuations, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding; optimize atomic checkpoint overhead |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | Job Select PR #246, Warp PR #247, actual-payment PR #251 and replacement PR #252 have two independent nonauthor reviews and durable scoped validation records; [replacement review](review-pr-252.md). Earlier milestones remain documented in their feature/review pages. | Continue issue-managed, reviewed PRs through complete acceptance |

The latest composed work includes twenty [actual-payment](cast-payments.md)
bodies and nine [token/counter replacement](token-counter-replacements.md) bodies,
alongside the previously reviewed Job Select and Warp stages. Coverage is
1,736/5,164, with 84 token factories, one emblem and observation schema 75.
The complete native suite passes 1,614 tests; clean full compiler/data and
fresh-library public API collections pass 330 and 224 unique methods respectively.
Replacement Standard100 witnesses all nine full bodies in both seats and real
Warp/exile/later-paid-recast routes, natural game outcomes and 187,965 owned
replays. Initial logical state is 20,280 bytes; maximum is 21,128 in that workload.
These supported-card workloads do not prove complete Standard rules or full-deck
performance. The two independent nonauthor reviews and precise evidence scopes
are recorded in [the replacement review](review-pr-252.md).

Parallel [Crew/Vehicle rules (#250)](https://github.com/altf4/rustmage/issues/250)
are being composed with both completed foundations. Its twenty-four bodies are
not included in the count above. That integration must preserve current Vehicle
kinds through targeting, counters, funding, captured damage facts and cleanup,
with owned private continuations and two final independent reviews.

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
