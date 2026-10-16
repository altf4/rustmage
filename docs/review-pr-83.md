# Independent review: characteristic and count-scaled stats

PR: https://github.com/altf4/rustmage/pull/83
Implementation reviewed: a5ade9f846cd9e28de113d35ed3398aafb18b842.

Two independent read-only subagents reviewed this milestone:

- Rules: all 18 printed definitions, all-zone characteristics, layer 7c scaling,
  attachment-controller counts, type/token/subtype filters, source indexing,
  frozen departure information, SBA timing and complete-body compiler gates.
- Library: count recursion boundaries, characteristic/additive arithmetic,
  shared-axis evaluation, all-zone values, widened type-mask consumers,
  indexing/LKI, snapshots, the public native checker and all 18 admissions.

Both verified the implementation HEAD and reported no actionable findings. Both
independently passed 533 Rust tests (525 unit and eight integration), 95 compiler
tests and catalogue consistency. The library reviewer also passed 61 release
Python tests and 20 native games with 4,657 independent public stat checks and
1,125 cloned/replayed boundaries. Neither reviewer edited files, mutated Git or
posted externally.

Author validation passed those suites, rustfmt and Clippy with warnings denied.
One hundred supported Standard games completed naturally: 34,101 decisions,
22,117 independent public stat checks and 4,912 cloned/replayed boundaries.
Maximum initialized state was 12,088 bytes, excluding allocator metadata, spare
capacity, external snapshots and immutable definitions. The public checker skips
attached or modified creatures when verifying bare characteristic values.
Dynamic-stats.md explicitly identifies type-only and source-index structural
fixtures; these do not admit unsupported card gameplay or unattached Aura play.

Coverage is 907/5,164 Standard identities, six regression definitions and 41 token
factories. General setting/switching/dependency layers, remaining card semantics
and native RL/search integration remain incomplete. These reviews do not certify
full-format conformance or establish an XMage speed comparison.
