# Per-set progress and resumption

Maintain one versioned `docs/sets/<lowercase-code>.md` and one GitHub tracking
issue per selected set. Link them to `data/sets/<code>.json`; do not copy or
manually maintain a second membership manifest. Reuse existing records and
acceptance evidence. Keep work for different sets in separate ledgers even when
they share engine changes or canonical reprint definitions.

Use the following structure, adapting the tables to the set's size. This is a
record of facts, not a checklist to mark complete on intent alone.

## Scope and baseline

Record the set name/code, target date/format, issue URL, default/integration
branch and commit, starting timestamp, source revisions and membership hash.
State full physical identity count, printing reconciliation, supplemental/starter
products, banned members, missing sources and the initial admission count.
Preserve the denominator throughout the run. Any reviewed membership correction
needs its old/new counts and source evidence.

## Card obligations and evidence

| Oracle identity / name | Canonical definition | Obligations and missing mechanics | State | Tests and source evidence | PR / reviewed SHA |
| --- | --- | --- | --- | --- | --- |

Every member must be accounted for. A separate linked machine-readable ledger is
fine for large sets. Record why existing tests cover a reprint rather than
reauthoring it. Distinguish these states:

- Unresolved source or identity: membership remains counted.
- Gated: some behavior or its evidence is missing.
- Implementing: named owner, worktree/branch and dependency.
- Tested candidate: all obligations covered locally; review/integration pending.
- Reviewed: two eligible reviews at the specified published head.
- Merged and verified: included at the recorded integration revision with passing
  validation. This still does not by itself certify full-set games.

`set_status` reports admission flags, not those evidence states. Report both.
For each blocker name the exact missing rule/source/access, the affected cards,
what can proceed independently and the next action that would resolve it.

## Batches, reviews and game coverage

Record each batch's owned paths, base revision, shared mechanics, source pins,
PR URL, review identities/SHAs, findings/resolutions and integration commit.
Reviewers must be independent of all authored ancestors included in their review.
A substantive diff change requires fresh relevant review evidence.

Record focused tests and full validator exit statuses, report locations/hashes,
and known failures. Keep an execution matrix for full games: legal deck lists,
colors, seeds, observed mechanics, decision branches, completed outcomes and
replay/privacy/owned-state checks. Cross-reference targeted rules scenarios for
obligations the games did not exercise. Never substitute deck inclusion counts
for execution evidence.

## Measurements

Record the start/end times, number of newly verified whole cards and cards/hour;
separate existing coverage and unmerged work. For simulation measurements record
revision, compiler/profile, hardware, deck/policy/seed workload, sample count,
wall time and the units actually emitted. Identify whether a rate includes
observation encoding, cloning or assertion overhead. Keep the Freeform smoke
measurement separate from this set's gameplay and search workloads.

## Resume here

Keep this section current before handing off or ending a session:

- Current commit/branch, dirty owned files, active PRs and integration dependencies.
- The next concrete implementation/test/review action, plus unclaimed next batch.
- Source pins and fetch commands; necessary source artifacts and hashes.
- Active process/session/unit IDs, command, immutable input commit and log/target
  locations. Check whether they still exist on this host before resuming. An old
  session ID is not portable; do not assume a process finished after disconnect.
- Required shared libraries and exact hashes/paths **on the producing host**.
  A new computer rebuilds missing libraries from the recorded source; it does
  not fabricate an old path or move a library still used on the original host.
- Completed validation/reviews, unresolved findings and missing permissions.
- Exact command or prompt to resume the selected set.

Keep bulky logs under `~/.cache` and link portable summaries in the ledger/PR.
Before changing machines, preserve necessary evidence in repository records or
accessible CI/PR artifacts: a private absolute cache path alone is not a handoff.
Never delete an active output, useful uncommitted work or a required library.

When all acceptance gates pass, record the accepted revision and finish the
selected-set task. Do not silently start another set or close the overarching
all-Standard goal.
