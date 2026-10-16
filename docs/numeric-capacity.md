# Recoverable numeric capacity

Issue: https://github.com/altf4/rustmage/issues/88

A legal Magic action can exceed a finite machine representation. That is an
engine capacity error, never a win, loss or draw. `Game::apply` returns
`Error::CapacityExceeded(CapacityExceeded { resource })` and restores the entire
pre-action state and revision, including RNG, pending choices, prior side effects,
stack identities and partially changed objects. Repeating that action returns the
same error. Earlier successful decisions (including paid activation costs) remain
committed. No rules value is clamped or saturated.

The game remains usable: observe, snapshot, clone, restore an earlier checkpoint,
or select another legal action. An RL caller must record/handle the capacity
failure separately; retrying the same branch cannot progress it. This provides
recoverability, not arbitrary-precision arithmetic or an infinite-loop adjudicator.
Counters remain u64, derived stats/damage/life i128, captured damage u128, mana
u32, and identities/revisions retain their existing finite representations.

`Game::restore` now returns `Result<(), Error>`; revision exhaustion is checked
before any state change. Optional control responses use schema 2 and include
`capacity: { "resource": "..." }` on numeric exhaustion. C ABI version 1 and
observation schema 35 are unchanged. Python raises `CapacityError(EngineError)`
with `.resource`; it retains the native handle. Snapshot-ID exhaustion likewise
keeps stored checkpoints intact. Resource strings are diagnostic descriptions.

The private `CapacitySignal` unwinds only to the action transaction boundary.
Other invariant failures restore the checkpoint and resume their original panic;
they are not mislabeled as numeric exhaustion. `resume_unwind` bypasses the panic
hook for the private signal. The crate requires `panic = "unwind"` and explicitly
rejects `panic = "abort"` at compilation, because abort cannot uphold recovery.
Allocation failure/process termination are outside this numeric contract.
See [Rust's resume_unwind documentation](https://doc.rust-lang.org/std/panic/fn.resume_unwind.html).

Actions also check derived battlefield stats before committing a suspended
continuation that changed relevant state; ordinary completed resolutions check
through state-based actions. This keeps subsequent observations representable.
Virtual mana funding uses u128 temporary pools and u64 component payments so
exploring an impossible hypothetical plan cannot overflow a real u32 pool.
Saturating arithmetic there bounds optimistic pruning only, never game state.

Object arenas and the two library vectors use `Arc<Vec<T>>` copy-on-write storage.
Snapshots, clones and action checkpoints share those buffers until mutation;
other mutable vectors/continuations are copied. This is whole-buffer detachment,
not page-level COW or a claim of constant-time game cloning. Logical `state_bytes`
includes the out-of-line Vec headers, excludes allocator/reference-count metadata,
spare capacity, immutable definitions and external checkpoints, and does not
estimate physical memory savings from sharing.

## Validation and cost

Author validation: 569 Rust tests (561 unit plus eight integration), 66 release
Python tests, Clippy with warnings denied, and catalogue consistency. Boundary
fixtures cover mana, counters after a partial write, life/damage batches, draw and
spell ordinals, turn, stack/object generations, revision, suspended derived stats,
COW detachment, and actual C ABI handle reuse. Public Python play repeatedly
self-doubles Zimone until capacity fails, then checks both views, revision, retry,
clone, checkpoints and recovery. Synthetic extreme-state fixtures are distinct
from that public legal-game test. An abort-profile check fails with the intended
compile-time diagnostic.

One hundred supported Standard counter-group games complete naturally: 56,734
decisions, 18,665 independently checked public counter values, 14,029 cloned/replayed
boundaries and 18,788 maximum initialized state bytes (previously 18,764).

Local release `benchmark 1000`, same limited Freeform deck/seed workload, 571,049
decisions per run, Rust 1.98, this host, unpinned CPU:

| Measurement | Main 09f3c7d baseline | Capacity transactions, two runs |
| --- | ---: | ---: |
| Decisions/second | 555,127 | 405,502; 378,397 |
| Clone ns | 420 | 246; 276 |
| Restore ns | 452 | 122; 251 |
| Initial logical bytes | 17,560 | 17,584 |

Atomic checkpoints lower measured rollout throughput by roughly 27–32% in this
small workload; shared buffers improve measured clone/restore costs. These short
runs have noise and are evidence of a tradeoff, not a general speed claim. They
include native smoke policy and observation construction, exclude JSON, model
inference and tensor encoding, and do not compare equivalent Java workloads.
Future profiling should reduce transaction copying while retaining atomicity.

Coverage remains 922/5,164 pinned Standard identities, six regression definitions
and 41 token factories. This closes a numeric recovery gap, not full Standard,
all-format rules conformance, or the complete RL integration.
