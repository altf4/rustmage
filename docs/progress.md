# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 910 executable Standard identities | 4,254 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Complete owned-state copies, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | PR #6 foundation; PR #8 catalogue/templates; PR #10 damage/combat; PR #12 object identity/removal; PR #13 programs/triggers; PR #15 targeting; PR #17 counters; PR #18 fixed tokens; PR #20 activations; PR #22 mana/artifact tokens; PR #23 payment windows; PR #25 scry/surveil; PR #27 additive stats; PR #29 effective keywords; PR #31 blocking restrictions; PR #33 face-up exile and linked returns; PR #35 nonbasic lands; PR #37 library search; PR #39 spell modes; PR #41 distinct target groups/fight; PR #43 legend rule/state-based choices; PR #45 completed-cast triggers/prowess; PR #47 step triggers/intervening-if; PR #49 resolution-time sacrifices/conditional payments; PR #51 life-gain/creature-death events; PR #53 Equipment/continuous attached bonuses; PR #55 creature Auras/transactional noncasting entry; PR #57 milling/exact-batch recovery; PR #59 player-damage triggers/captured event amounts; PR #61 resolution mana payments/ward/restricted counters; PR #63 hand-zone cycling/typecycling; PR #65 discard/cycling events/cleanup batches; PR #67 draw events/per-turn ordinals; PR #69 connive continuations; PR #71 once-per-turn trigger limits/typed entry observers; PR #73 committed hybrid spell costs; PR #75 all-zones changeling/shared subtype matching; PR #77 continuous keyword grants/exact haste-aware funding; PR #79 pre-entry life payments/conditional land entry; PR #81 controlled-subtype activation guards/conditional mana; PR #83 all-zone characteristic/count-scaled stats; PR #85 mandatory additional spell costs, with two subagent reviews each | Continue issue-managed, reviewed PRs through complete acceptance |

The latest work adds mandatory sacrifice/discard casting costs, exact reserved
mana funding and owned continuations through cost-triggered entry choices.
Three complete definitions bring coverage to 910/5,164, with 41 token factories.
Native/Python fixtures cover legal payment, privacy, triggers and branch replay.
Full Standard acceptance remains outstanding.

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
