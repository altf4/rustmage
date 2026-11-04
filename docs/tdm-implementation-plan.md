# Tarkir: Dragonstorm implementation

Tracking issue: [#570](https://github.com/altf4/rustmage/issues/570).
Implementation branch: `codex/tdm-complete-set`.
Baseline: `c87b55079c540edde1c49cc725a1e1475bebf68b`.
Pinned XMage: `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

The frozen pool has **276 physical identities**, deduplicating432 printings,
plus13 paired Omen spell faces. Cori-Steel Cutter remains banned in the frozen
Standard snapshot; its behavior and games use separately labelled Freeform scope.

All276 physical definitions have local whole-card outcome evidence:139 new
implementations and137 previously admitted definitions verified. All13 Omen pairs
and the Reliquary Dragon token are complete. The printed proof ledger is frozen
at SHA256 `b6716c0d6f18774988997e1183caaf5b0f977d2049406f56da7dbfaecec2942a`.
See [the integration ledger](tdm-card-integration-evidence.json) and
[the sixteen-family matrix](tdm-set-coverage-matrix.json), which binds265 named
proofs to exact source and retained positive receipts. These are card/scenario
proofs; full-set game, performance, release and published-PR acceptance remain open.

Engine86 was accepted at `2660c86112362973c195bee4c1b85e1a968f9f3a` with
full9/9 release validation,4490 native cases across108 targets,43 infrastructure
contracts,10 Python binding cases and two independent pure nonauthor reviews.
All three author caches were warmed before dependent card work. No card or shared
engine source was changed during its consuming jobs.

The first fixed52-game run completed three pairings and stopped four when the
only legal action became Concede after a deferred Devotee mana-color choice.
The harness rejected those artificial terminals. Independent review confirmed
**E:PAID_MANA_USE_RECEIPT**, the87th unweighted audit group: a post-cost funding
world added the chosen mana without recording the paid child's once-per-turn
use, so it imagined a second conversion. The real committed use then made the
parent payment impossible. This is an engine fundability defect. The wrong color
is not intrinsically forbidden by Magic; an action API promising completion must
preserve that promise or implement the required rollback.

Work has returned to the engine phase for a test-first minimal receipt projection
repair. Cover once/exhaust, exact incarnation and acquisition, borrowed/self-granted
use, nested parents and genuinely repeatable converters, plus representative raw,
index, observation, transactional rejection and owned restore contracts. Preserve
real observer timing; do not replay costs or change decks/seeds to evade the bug.
Finish focused contracts and two independent nonauthor reviews before resuming
dependent acceptance. Preserve every printed card definition and its provenance.

Two preliminary pure reviewers read every276 physical Rust definition, all13
Omen bodies, complete Oracle text and all603 unique pinned primary/helper Java
blobs; neither found a translation mismatch. They also independently approved
exactly63 final schema201 trace additions, preserving all3240 older rows. The
accepted regression baseline now has3303 rows; final engine acceptance must
verify it again. [Trace evidence](tdm-final-trace-additions.json).

Remaining acceptance uses the exact frozen decks/seeds in
`examples/support/tdm_game_decks.json`:48 Standard games and4 separate Cutter
Freeform games,4096 actions each, natural terminal outcomes, all colors and eight
mechanic payoffs, complete raw replay and meaningful owned search roots. The
harness source and physical manifest/catalogue/printed-ledger hashes authenticate
receipts. No seed search, deck mutation, partial-pool override or card-aware bot
is permitted. Benchmark the same verified Standard48x3 and Freeform4x3 separately,
with construction, legal actions, shared policy and actor-observation costs stated.

Reconcile the README accepted-branch union without dropping previously accepted
ECL/TMT coverage, record measured implementation throughput with elapsed interval
and scope, regenerate linkage, freeze final inputs and run
`python3 scripts/validate.py --release --python smoke` with the existing guarded
cache. Open exactly one completed-set PR after full-set acceptance, then obtain
two fresh independent pure nonauthor reviews of its final published head.

All Cargo remains guarded: one build, one job/test thread and6GiB across worktrees.
Retain reports under the persistent cache and preserve every required immutable
shared library at its exact path/hash. Never change consuming inputs or delete
active outputs. The complete-set and broader Standard goals remain open.
