# Independent review: declared-attacker history and raid

PR: https://github.com/altf4/rustmage/pull/117
Final implementation reviewed: 74a058fded7af3278da1a268ac1efe020590efd2.
Base: main b2332bb170c6014300df24035f8fe818cb3dddd0.

Two independent read-only subagents reviewed the implementation. The rules
reviewer found a P2 in the first revision: attack triggers were dispatched before
the entire declared batch had been tapped and committed. A compiler-supported
intervening condition counting tapped creatures could consequently depend on
attacker selection order. The fix captures vigilance before tapping, commits
all attackers and their history, taps the complete batch, then dispatches events.
Native regressions cover both selection orders, with and without vigilance;
a compiler regression proves the affected guard is admitted. Both reviewers
re-reviewed the final revision and report no remaining actionable findings.

The library reviewer independently passed 771 Rust tests, 141 compiler/data
tests, 100 freshly built release Python tests, catalogue/diff checks and the
hundred-game native replay workload. That workload exactly reproduced 67,476
decisions, 1,837 staged payments, 349,957 public checks, 27,151 clone/replay
boundaries and 20,628 maximum initialized state bytes. Initial review also
verified all seven card and two supporting Java source hashes.

The rules reviewer independently passed 771 Rust tests, both focused compiler
regressions and the two release Python history/Chart replay tests at the final
revision. The earlier review checked all seven complete bodies and source
hashes, with the full compiler suite passing. Reviewers edited no files.

Author validation includes rustfmt, Clippy with warnings denied, 771 Rust,
141 compiler/data and 100 fresh release Python tests, catalogue consistency,
and the native workload. See [attack-history-raid.md](attack-history-raid.md)
for provenance, rules, benchmark policy, coverage and limitations.

Observation schema is 42; ABI 1 and protocol 2 remain unchanged. Initial logical
state is 19,720 bytes, plus 16 initialized bytes per unique attacker incarnation.
Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 1,091/5,164 Standard identities; 4,073 remain unsupported. The full
Standard and native RL/search acceptance goal remains incomplete and active.
