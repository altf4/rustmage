# Independent review: life-loss history and guarded card bodies

PR: https://github.com/altf4/rustmage/pull/123
Final implementation reviewed: 3ff5a3f8bbb1da9af817e64493ca0d6a75c761e0.
Base: main 6bbb0b85c2a71f1afdda692d74477a21e98a527e.

Two independent read-only subagents reviewed the initial source and corrected
revision. Both report no remaining actionable findings and edited no files,
mutated no Git state and posted nowhere externally.

Initial review found a P2: a shock-land life payment in the same entry batch
could incorrectly enable Frilled Sparkshooter's conditional counter replacement.
The official simultaneous-entry ruling evaluates other entry conditions before
shock-land choices. The fix captures every incoming counter quantity before
choices, owns it in EntryMove, and applies the frozen result on commit. ETB
triggers and later separate entries still see the completed payment. Both
reviewers confirmed the fix; eight prior-history/payment/order cases distinguish
Frilled replacement from Collector ETB and later entries, with pending-choice
snapshot/fork/restore/replay. Redirected Aura entries clear captured counters
when they move elsewhere. Existing size-based pending-move byte accounting
includes the extra 32 bytes per move; the initial state is unchanged by the fix.

The rules reviewer checked all concrete loss routes, exact positive totals,
net gains, controllers, pre-entry versus trigger timing, Vampire permanent scope,
all three complete bodies and residual gates. All six pinned Java hashes match.
Independently passed 800 Rust, 147 compiler/data, three public Python regression
tests and catalogue consistency on the initial source; final re-review passed
13 matching life-loss and 83 entry-related tests plus diff checks.

The library reviewer audited all production life decreases (only counterfactual
FundingState retains a direct virtual subtraction), owned totals/reset, capacity
rollback, choice flags, snapshots/privacy/schema and target/cost legality.
Independently passed 800 Rust, 147 compiler/data and 106 release Python tests,
catalogue/diff checks and all six hashes initially. On the final revision,
independently repeated all 800 Rust, 106 release Python, diff checks and the
native hundred-game workload. Metrics exactly match: 48,564 decisions, 145,692
public checks, 20,660 clone/restore/replay boundaries and 20,668 maximum logical
initialized state bytes, with all three entries, both entry branches and both
guarded trigger kinds starting resolution.

Author validation also includes formatting and Clippy with warnings denied.
See [life-loss-history.md](life-loss-history.md) for pinned provenance, rule
sources, validation, workload policy, state accounting and performance limits.
Only this record and progress bookkeeping follow the reviewed source.
Coverage is 1,102/5,164 Standard identities, with 4,062 unsupported. Observation
44 adds each player's exact public current-turn loss total; ABI 1 and protocol
2 remain unchanged. Measured initial logical state is 19,752 bytes. The full
Standard and native RL/search goal remains incomplete and active.
