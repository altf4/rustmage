# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,770 executable Standard identities | 3,394 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Isolated COW object/library buffers, owned continuations, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding; optimize atomic checkpoint overhead |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | Job Select PR #246, Warp PR #247, actual-payment PR #251 and replacement PR #252 have two independent nonauthor reviews and durable scoped validation records; [replacement review](review-pr-252.md). Earlier milestones remain documented in their feature/review pages. | Continue issue-managed, reviewed PRs through complete acceptance |

The latest isolated author batch adds ten complete [attachment Ward](attachment-ward.md)
bodies to the merged1760-card Crew/payment/replacement foundation. This branch
therefore covers1770/5164 Standard identities with84 token factories,one emblem
and schema77. The unchanged rule runtime passes1635 native tests and228 unique
public methods; strict reminder admission adds focused compiler controls while
preserving all previous generated programs. Its legal100-game workload validates
all ten complete effect vectors, Ward payment/decline and owned replay. Final08c native,56paid-case and repeated100-game checks pass; two independent
exact-head review confirmations are required before merging. This batch does not compose the parallel Adventure/Saga branches;
that integration requires its own canonical and full regression evidence.

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
