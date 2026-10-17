# Direct Rust card authoring

New cards are authored in `src/card_definitions/<slug>.rs`. Write their complete
behavior using `CardRules`, existing typed instructions/targets/costs, and shared
engine mechanics. Add a small typed effect when needed; bespoke effects are also
appropriate. Gameplay never reads Oracle text or invokes an LLM.

Each card has a sibling JSON provenance record: Oracle identity, hash of the full
pinned physical facts, `complete`, and the Java revision/path/hash. The small
`scripts/direct_cards.py` linker validates identity/facts, discovers files and emits
module links. It does not translate behavior. Incomplete definitions stay gated.
Existing working templates remain available for cards they already implement.

For an authored card the catalogue generator supplies immutable physical metadata
and links its Rust constants before reaching the English compiler. All selected
mode combinations and their additional costs are explicit Rust. Rust compile-time
checks reject Unsupported behavior and mismatched modal cost tables. Add fields to
`CardRules` only when a shared mechanic actually needs them.

Agents can edit disjoint paired files in separate worktrees. Regenerate derived
files during integration rather than resolving generated text by hand. Add focused
rules tests that exercise every printed obligation, legal/illegal actions, hidden
views and owned branching/restore. Independent subagents review the complete card
translations and interactions; coverage is not earned by partial bodies.

Run `python3 scripts/generate_catalogue.py`, then
`python3 scripts/validate.py --release`. The latter runs full native rules tests,
Clippy/format/consistency and direct-definition/provenance checks; it also runs the
existing native smoke benchmark. Each run reports commands, source stability,
complete-card coverage, performance logs and cleanup in one JSON artifact. It
retains shared-library paths/hashes and cleans terminal disposable build output.
The smoke benchmark is a limited Freeform deck, not Standard-wide acceptance.

The representative batch directly implements Explosive Derailment, Jailbreak
Scheme, Requisition Raid and Rustler Rampage (16 legal combinations, both seats).
It adds selected-player creature counter and stun-aware untap batches; all other
behavior reuses the existing engine. Main and candidate coverage are reported
separately until final validation/review/merge. The all-Standard goal remains open.


Initial acceptance on actual land-entry main passes1,936 native tests, Clippy,
format/generation checks, complete-source provenance and three public API methods.
All947 tracked inputs stay fixed. The validator cleans543,899,648 bytes after its
jobs close and preserves all shared libraries. The first development compile
caught an include-file comment error; a later new fixture referenced a card name
absent from the catalogue. Both are corrected before this passing run. Independent
review additionally hardened cost-table shape and stale-facts rejection.
[Machine-readable results](direct-authoring-results.json) record throughput,
performance scope and strict preservation of every5,330 unrelated definition,
930 inherited helper arms and97 token recipes. Final publication changes only
instructions, documentation and reporting; independent final reviews gate merge.
