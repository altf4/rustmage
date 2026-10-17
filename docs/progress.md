# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,772 executable Standard identities | 3,392 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Isolated COW object/library buffers, owned continuations, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding; optimize atomic checkpoint overhead |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | Saga PR #259 has two independent nonauthor approvals of the complete authored batch and durable scoped evidence in the [Saga review record](review-pr-259.md). Crew PR #253 and earlier milestones retain their own feature/review records. | Continue issue-managed, reviewed PRs through complete acceptance |

The Saga batch adds twelve complete [single-face Saga bodies](sagas.md) to the
reviewed Crew, actual-payment and replacement foundations. Coverage is
1,772/5,164, with 86 token factories, one emblem and observation schema 77.
All 1,636 native tests pass. A complete 336-method compiler run passed before
review found overly broad parenthetical removal; exact reminder authentication
and 50 additional negative controls now close that admission hole. The complete
337-method compiler/data suite passes in 537.691 seconds.

The optimized Saga Standard100 workload completes naturally with every chapter
and final sacrifice verified in both seats, printed creature bodies/keywords
and actual creature-Saga attacks. It checks 102,849 owned replays and dropped-parent
children, 205,698 wrong-actor/stale-revision denials and 36 private choice frames.
Initial logical state is 20,312 bytes; maximum is 21,192. The complete 226-method public baseline passes in 792.491s on the earlier
immutable fingerprint; all 24 paid Saga cases also pass in 65.999s on the final
metadata-only fingerprint. Two independent nonauthor reviewers approved the
complete authored batch at e0afa05; the final published head is checked again
before guarded merge, as recorded in the [Saga review record](review-pr-259.md).
These workloads do not prove complete Standard rules or full-deck performance.

[Adventure spell faces and owned normal-play exile permissions (#254)](https://github.com/altf4/rustmage/issues/254)
are now being implemented in an exclusive worktree. The shared foundation and
nineteen audited complete pairs include a land normal face; the fifty-six other
Adventure bodies and thirteen distinct Omen bodies remain full-goal requirements.

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
