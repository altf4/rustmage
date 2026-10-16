# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,077 executable Standard identities | 4,087 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Isolated COW object/library buffers, owned continuations, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding; optimize atomic checkpoint overhead |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | PR #6 foundation; PR #8 catalogue/templates; PR #10 damage/combat; PR #12 object identity/removal; PR #13 programs/triggers; PR #15 targeting; PR #17 counters; PR #18 fixed tokens; PR #20 activations; PR #22 mana/artifact tokens; PR #23 payment windows; PR #25 scry/surveil; PR #27 additive stats; PR #29 effective keywords; PR #31 blocking restrictions; PR #33 face-up exile and linked returns; PR #35 nonbasic lands; PR #37 library search; PR #39 spell modes; PR #41 distinct target groups/fight; PR #43 legend rule/state-based choices; PR #45 completed-cast triggers/prowess; PR #47 step triggers/intervening-if; PR #49 resolution-time sacrifices/conditional payments; PR #51 life-gain/creature-death events; PR #53 Equipment/continuous attached bonuses; PR #55 creature Auras/transactional noncasting entry; PR #57 milling/exact-batch recovery; PR #59 player-damage triggers/captured event amounts; PR #61 resolution mana payments/ward/restricted counters; PR #63 hand-zone cycling/typecycling; PR #65 discard/cycling events/cleanup batches; PR #67 draw events/per-turn ordinals; PR #69 connive continuations; PR #71 once-per-turn trigger limits/typed entry observers; PR #73 committed hybrid spell costs; PR #75 all-zones changeling/shared subtype matching; PR #77 continuous keyword grants/exact haste-aware funding; PR #79 pre-entry life payments/conditional land entry; PR #81 controlled-subtype activation guards/conditional mana; PR #83 all-zone characteristic/count-scaled stats; PR #85 mandatory additional spell costs; PR #87 simultaneous counter groups/distinct recipients/counter doubling, with two subagent reviews each; PR #89 recoverable numeric capacity/atomic action checkpoints/COW buffers, also independently reviewed twice; PR #91 live removal target unions/predicates, color facts and this-turn damage histories, reviewed twice; PR #94 simultaneous battlefield group removal/damage and exact result continuations, reviewed twice; PR #95 loyalty counters and planeswalker damage foundation, reviewed twice; PR #96 exact planeswalker combat defenders and departure/damage routing, reviewed twice; PR #97 fixed loyalty activations, persistent life-event emblems and complete Dellian body, reviewed twice; PR #99 exact scaled token batches, per-turn life-gain totals and eleven complete admissions, reviewed twice; PR #101 exact scaled damage/life quantities and eight complete admissions, reviewed twice; PR #103 distinct characteristic quantities, scaled draw/growth/drain and eight complete bodies, reviewed twice; PR #105 guarded programs, intervening-if checks and nineteen complete bodies, reviewed twice; PR #107 restricted counterspell characteristics/type unions and seven complete bodies, reviewed twice; PR #109 exact two-brid equivalents/commitment and three complete bodies, reviewed twice; PR #111 locked generic spell reductions and twenty complete bodies, with a compiler admission finding resolved and re-reviewed twice; PR #113 count-scaled reductions/affinity and fourteen complete bodies, reviewed twice | Continue issue-managed, reviewed PRs through complete acceptance |

The latest work adds count-scaled generic reductions and affinity using exact
owned battlefield/graveyard/exile counts, subtype unions and distinct colors.
Fourteen complete bodies bring coverage to 1,077/5,164, with 49 token factories
and one emblem. See [scaled-spell-reductions.md](scaled-spell-reductions.md) for
pinned provenance, rules, native/Python rollback evidence and workload limits.
Hand/unknown-history/target-dependent reductions, taxes and remaining body
dependencies stay gated. Full Standard and broader native RL/search acceptance
remain outstanding.

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
