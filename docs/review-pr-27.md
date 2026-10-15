# Independent review: derived additive creature stats

PR: https://github.com/altf4/rustmage/pull/27
Implementation reviewed: 09bd716 against main.

Two independent read-only subagents reviewed the milestone:

- Rules: all 40 admissions, layer-7c additions and filters, dynamic statics versus
  resolution-locked effects, source reincarnation, combat/lethal calculations,
  repeated SBAs, cleanup, simultaneous LKI and source-index replay.
- Library: source-index lifecycle and snapshot ownership, derived-stat consumers,
  whole-batch characteristic capture, asymmetric modifier migration, compiler
  admission, public projections and native/Python replay.

Both reported no actionable findings. Both passed all 140 Rust tests, 29 script
compiler/importer tests and catalogue consistency. The library reviewer also
rebuilt release, passed all 16 Python tests and completed 20 native games covering
2,747 decisions with live static sources. Neither reviewer edited files or posted
externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 40,075 decisions, 576 activations and 13,339 decisions with
live static sources. Native fixtures cover exact filters, source index,
static entrants versus locked mass sets, lord stacking/exclusion, chained deaths,
simultaneous LKI, combat, cleanup and source reincarnation. The Python fixture
checks derived anthem stats through isolated cloning and restore/replay.

A freshly built limited Freeform benchmark completed 1,000 games and 571,049
decisions at approximately 840k decisions/sec, with 7,104 accounted initial state
bytes. This smoke workload does not establish full Standard performance or a
speed comparison with Java.

Coverage is 279/5,164 Standard identities, six regression cards and 28 token
factories. Base-setting/switching/characteristic-defining values, type/color/
control/ability changes, dependencies, attachments, variable/conditional effects,
remaining card mechanics and full RL/search acceptance remain outstanding.
These reviews do not certify full Standard conformance or the entire layer system.
