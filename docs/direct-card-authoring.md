# Direct Rust card authoring

Card behavior lives in `src/card_definitions/<set>/<slug>.rs`, with a sibling
provenance JSON. Use `CardRules`, typed instructions/targets/costs and shared
mechanics. `CardExtras` exposes the existing immutable mechanics directly; add a
shared field or a small bespoke effect when needed. No Oracle text or LLM is used
at runtime. The English compiler and its parser-only tests have been removed.

Every existing physical card, alternate spell face, token and emblem now has an
independently editable Rust definition. The migration mechanically relocated
already typed behavior; it does not newly certify previously gated cards. Read
the entire Oracle body and XMage implementation before changing a definition.
Mark `complete` true only after every printed obligation works and is tested.

A provenance record pins the Oracle identity/facts and, for new authored work,
the Java revision, path and SHA. Migrated records additionally identify the source
revision and original typed definition hash. Existing full-source audits remain
in `docs/*admissions*.json` and parser-free provenance tests.

`data/catalogue-identities.json` retains stable IDs, names, aliases, physical
metadata, colors, synthetic recipes and creature-type ordering. It contains no
executable card programs. `scripts/link_catalogue.py` validates pinned facts and
canonical paths and emits only module links and catalogue tables. It never
selects behavior from wording. The Rust compiler checks typed definitions and
rejects complete cards with Unsupported behavior or mismatched modal-cost tables.

The directory's set code identifies the canonical definition owner; a reprint
shares that definition. Ownership counts in the coverage report do not prove
complete membership of a set. Full-set acceptance must reconcile all printings,
including supplementary Standard-legal cards, with the pinned source inventory.

Agents edit disjoint paired files in isolated worktrees. The integrator runs
`python3 scripts/link_catalogue.py` to regenerate derived artifacts. Add meaningful
rules tests for whole-card behavior, legality/privacy, transactional failures,
queued/source departure interactions and owned raw/index branching and restore.
Require two independent nonauthor reviews of the final published PR.

Run `python3 scripts/link_catalogue.py`, then
`python3 scripts/validate.py --release`. The validator runs native rules tests,
Clippy, format/link consistency and provenance tests, plus the existing native
smoke benchmark. It reports fixed inputs, coverage and cleanup and preserves every
shared library at its exact path/hash. The Freeform smoke benchmark is a limited
regression workload, not full-set or all-Standard acceptance.

[Compiler-removal proof](compiler-removal-results.json) records exact equality of
the compiled before/after definitions, all helper values and immutable globals.
The all-Standard goal remains open; finish one complete set with full-game tests
before advancing to the next set.

Each paired record pins both the full Oracle facts and its reviewed physical
metadata (including intrinsic colors). Changes to costs, types or legality in the
identity manifest require an explicit per-card provenance update; the linker
rejects accidental drift. Synthetic entries carry the same metadata pin.
