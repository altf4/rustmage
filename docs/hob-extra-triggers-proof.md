# HOB additional trigger instances

Scope: complete Bifur, Melodic Rider and Wizard's Staff. Oracle bodies are frozen
in the Standard source data. Paired JSON pins record full source/helper hashes,
current official rules and explicit Bifur Java-class absence evidence. The absent
class is not represented by a fabricated Java implementation; Harmonic Prodigy,
Roaming Throne and NumberOfTriggersEvent provide the pinned reusable analogs.

| Obligation or boundary | Distinguishing proof |
| --- | --- |
| Bifur costs all three hybrid combinations, storied, entry/attack counter | Both seats; entry is the third qualifying permanent and creates two separately targeted instances; attacks create two counters |
| Dwarf permanent/current controller/current subtype, copied provider | Opponent and non-Dwarf rejected; current subtype/control and copied face acquire/lose qualification |
| Staff prowess and both sorcery equip abilities | Real paid cast/equip, Wizard-only rejection, equipment controller independent, noncreature Equipment requirement |
| Multiple providers | Two providers create three original instances, no recursive doubling; frozen prowess survives Staff departure |
| Ability removal/grants | Wasp removes printed provider; newer attachment grant survives and qualifies independently |
| LKI/hidden information | Simultaneous source/provider sacrifice creates three recruit instances from pre-event facts; private discard choices and owned raw/index/restore contract |
| Event metadata | Each ward trigger retains the same related stack identity and charges a separate payment |
| Trigger/resolution limits | Trigger-once cap remains one; optional resolution-once permits decline then accept but only one life gain |
| State/reflexive/delayed/copies | State latch once, direct reflexive scheduler once; actual Duplicator delayed instances and Echo ability copy remain unexpanded |
| Linked effects | Both Glass Casket trigger instances exile independently; all links return when source leaves |
| Snapshot protocol | Public JSON omits private witness; opaque snapshot-handle restore/fork retains pending count and publishes identical observation |
| Retained mana rollback | Actual Lotus Ring sacrifice of Staff-equipped Lookout; reverse restores attachments, keep replays frozen packet into two private recruit resolutions |

The witness is packet-owned: an unexpanded packet stores the number of additional
instances (zero means ordinary or already published). Expansion resets it before
any ordering/announcement. Copies and delayed packets therefore cannot redouble.
It survives typed snapshot/branch clones and frozen ManaEvent replay. ObjectView
and SpellView are output-only Serialize types; supported JSON restoration uses an
opaque u64 handle owning Snapshot(State), never deserializes these views.

The provider index is an optional boxed list, absent without relevant permanents.
Object-view evaluation returns immediately on that cold path. Providers are
registered/forgotten with battlefield and face providers. Qualification uses the
borrowed characteristic world without recursively constructing ObjectViews.
State byte accounting includes the optional box/list. Each private witness is a
usize; actual layouts and focused timings are recorded after the terminal run.

Validation ledger:

- First bounded guarded run: 12 executed, 7 passed, 5 failed in fixture setup
  (unselected hybrid cost, unsupported additive arbitrary subtype fixture,
  announcement correctly unavailable with no Wizard target). Compilation passed;
  retained-mana replay passed. Superseded by corrected fixtures using explicit
  cost choice, existing replacement subtype rows and rejected announcement.
- Source pins: 5,412 definitions; 1,512 immutable Java blobs verified including
  explicit Bifur absence evidence. Local linker regenerates catalogue artifacts;
  generated artifacts are excluded from the author commit.

- Second bounded run: 12/13 passed; the last fixture incorrectly interpreted
  effect_discards (already selected cards) as discard candidates. Superseded by
  direct visibility assertions for available private hand candidates.
- Final frozen Rust/JSON input digest:
  `56ebe8ff2234d7a9ddd2e2ebdc78c791aac3786fe4bd97ba83da9e1226cd3742`.
- Final guarded run: 14 new tests passed; rollback 4, storied 10, recruit 4,
  equipment 7, raw/index/restore 1 and existing Echo-copy 1 passed. This is 41
  executions /40 unique scenarios (retained rollback appears in both filters).
  Focused execution 0.02s, all seven families 0.11s; compile plus first family
  134.0797s and total guard 134.7572s. Peak memory 2,542,088,192bytes, enforced
  max 6,442,450,944bytes; no OOM or orphaned processes.
- Logs: `~/.cache/rustmage-authoring/hob-extra-triggers/final-{0..6}.log`.
  Guard report: `~/.cache/rustmage-build-reports/8c267ab50da948a29f2ccd574f345916.json`.
- Measured native layouts: ObjectView 432, SpellView 656, State 3920 bytes.
  Baseline delta will be measured by the integrator's already planned run;
  no extra compilation was performed just for layout evidence. Only two private
  usize witnesses and one optional boxed-index pointer were added. They are
  covered by existing object/packet inline byte accounting plus explicit index
  heap accounting. Whole-pool simulation throughput is integration acceptance,
  not inferred from these small rules scenarios.
- Final source verification repeated successfully (1,512 Java blobs), owned Rust
  formatting and diff whitespace checks passed. A broad first formatting check
  accidentally included generated catalogue linkage and reported its deliberately
  generated format; owned-file check excludes that artifact and passes.


Integration review and layout receipt:

- Independent nonauthors `hob_review_contracts` and `hob_review_rules` both approved
  exact author commit `a031e9c78a0f0e7ce87bbc2a6d095f691a3a386c`, including
  all printed obligations, full source/absence pins, packet/LKI/owned rollback
  and actual terminal 40-unique-scenario evidence. No actionable findings.
- Official CR603.2d explicitly excludes delayed/reflexive abilities created by
  object abilities. State-trigger exclusion also matches pinned
  `TriggeredAbilities.java` and `NumberOfTriggersEvent.java` and CR603.8.
- An already planned nearby integration test measured pre-multiplicity layouts:
  ObjectView432, SpellView640, State3888 bytes. With this core they are432/656/3920:
  actual deltas0/+16/+32 bytes, including alignment. Nearby baseline includes the
  four-effect/Fili immutable rules and Woodland leaf; this is not a separate
  raw-faca layout build. Log `<cache>/rustmage-authoring/hob-complete-set/schema155-pre-multiplicity-1/focused.log`.
- Final integrated inherited-schema audit, strict release acceptance and measured
  smoke performance remain pending; focused timings are not whole-set performance.
