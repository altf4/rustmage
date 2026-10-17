# PR 208 independent reviews

Implementation reviewed: `5480f5893c24e4c7246e67a67e3234af9cbfb8e9` against main `092dda1101f9c795bd884986fbf4ca5277cf7262`.

Two independent nonauthors reviewed the same frozen commit in separate clean detached worktrees. Both returned clean verdicts with no actionable findings. The only subsequent change is this review record.

## Library and API review

Reviewer: `/root/selection_library_review`, exclusive worktree `source-tap-acceptance/rustmage`. This reviewer authored none of PR208. After completing this review, the agent began the separate issue209 author task in a different worktree.

Independently checked immutable predicates, actor-only private windows, exact references, deferred whole-frame commits, owned clone/rollback, all twelve full bodies, all eighteen hybrid payment equivalents and Trainer's independent offspring selection. Passed **1,299 Rust tests**, **160 fresh-release Python tests** and the **100-game Standard workload with 71,080 decisions/replays** and all documented vectors.

An external public-API adverse probe exercised **25 stage signatures**, **671 rejected actions**, **12 divergent-clone checks**, **78 deferred selections**, **six copy-window observations** and **54 hybrid-stage checks**. Wrong actors, duplicates and out-of-window objects remained rejected without altering state. Independently verified all **122 Java hashes / 29 files** and canonical twelve-replacement/one-copy delta.

Local evidence: `<temp>/pr208-library-review-{native,python,standard100,probe}.log`; probe source `<temp>/pr208-library-review-probe.py`.

## Rules and compiler review

Reviewer: `/root/selection_rules_review`, exclusive worktree `source-tap-events/rustmage`. This reviewer authored none of PR208.

Independently checked every complete body, subtype union/exclusion/conjunction semantics, noncreature Changeling, Frontier's Mount-creature restriction, complete hybrid/offspring residuals and narrow whole-body gates. Passed **1,299 Rust tests**, **248 compiler/data tests**, fresh-release Python's **26 new public scenarios**, catalogue checks and the **100-game Standard workload with 71,080 decisions/replays**. The compiler suite completed in 270.795 seconds.

**113 name-preserving adverse full-body mutations** remained gated; all **twelve unchanged controls** admitted. Verified all pinned and checkout provenance hashes, **5,243 unchanged definitions**, twelve replacements, one added copy, **5,183 stable ordinary IDs**, **5,255 preserved old color masks**, **71 unchanged prior factories** and **442 preserved old helper arms**, with four added arms.

Local evidence: `<temp>/pr208-rules-review-{verdict,rust,compiler,python,standard100,audit,negative,provenance,catalogue,release}.log`; audit scripts `<temp>/pr208-rules-review-audit.py` and `<temp>/pr208-rules-review-negative-names.py`.

## Integration validation

The integration owner independently passed 1,299 Rust, 248 compiler/data and 160 fresh-release Python tests, all-target warnings-denied Clippy, formatting, catalogue and whitespace checks. The adjacent forage workload retains all vectors and 65,729 decisions/replays. Initialized logical state remains 19,896 bytes; schema remains 56. Detailed scope, provenance and measurements are in [filtered library selection](filtered-library-selection.md).

The frozen implementation was rebased onto merged main before review. The only production-tree difference from the already-tested integration parent was its preceding PR206 review record. Both reviewers rechecked exact HEAD and clean tracked status. No correctness finding required a code change or re-review.
