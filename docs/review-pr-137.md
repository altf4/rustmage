# PR #137 independent reviews

PR: https://github.com/altf4/rustmage/pull/137
Issue: https://github.com/altf4/rustmage/issues/136
Base main: `5845756cde2b163a33ac69fd2fab120ebbfbdbb3`
Initial source: `366bd7272d8d9490c9d3cbfaff1820b20d944cd2`
Corrected and cleared source: `9b028ce51bb4c8822bab5cc32e984c30351c2159`

Two independent read-only review subagents audited and re-reviewed the exact
source. Neither edited the checkout or made Git mutations.

## Findings and corrections

Both reviewers found a P2 in new replacement parsing: a multi-sentence alternative
ending in "instead" could pull an earlier independent conditional sentence into
the replacement branch. The parser now requires exactly one alternative sentence.
Three negative statement/full-card regressions reject ambiguous draw/life/scry and
global-wipe examples while the printed Withering Curse body still compiles.

The library reviewer independently reproduced a second P2 in an isolated copy:
Dream Beavers' gain-to-scry continuation could commit overflowing conditional
stats because the suspended-state dirty gate omitted player life/history changes.
The gate now compares life totals and the positive gain predicate. Four regression
cases cover real Dream Beavers and synthetic net-zero-life gain history, each with
near-capacity source/global bonuses. They require CapacityExceeded and exact state/
revision rollback through repeated attempts, clone/restore and safe observations.

## Corrected-source rules review

Cleared both corrections with no remaining actionable findings. Audited positive/
wide/net life history, continuous source/global stats/grants, controller predicates,
resolution versus intervening-if timing, mandatory targets, branch semantics,
whole bodies and compiler residual gates. Thirteen Oracle bodies and fifteen
unique pinned Java hashes match. Independently passed 859 Rust tests, both focused
compiler methods, catalogue and diff checks. Initial audit also passed all 161
compiler/data tests and the three new release Python methods. Entry timing agrees
with official Secrets of Strixhaven release notes.

## Corrected-source library review

Cleared both original independent reproductions and all new rollback cases with
no remaining findings. Audited authoritative decisions, private continuations,
atomic rollback/revision, virtual funding predicates, live derived stats, compiler
admission and workload checks. Independently passed all 859 Rust, 161 compiler/
data and 117 freshly built release Python tests, catalogue/diff checks, and all 27
provenance entries across fifteen unique Java files. Reproduced 100 native games:
61,031 decisions, 167,852 history/body checks, 317 gain-transition stat checks, 204
private surveil choices, 26,183 clone/restore replays and 20,720 maximum logical
initialized bytes.

Only this review record and the progress delivery row follow the cleared source.
Full Standard and broader native RL/search acceptance remain incomplete.
