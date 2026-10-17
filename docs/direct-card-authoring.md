# Direct Rust card authoring

Card behavior lives in `src/card_definitions/<set>/<slug>.rs`, with a sibling
provenance JSON. Use `CardRules`, typed instructions/targets/costs and shared
mechanics. Write keyword masks with named constants such as `Keywords::MENACE`,
`Keywords::NONE` and `.union(...)` for combinations; avoid numeric constructors. `CardExtras` exposes the existing immutable mechanics directly; add a
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

## Portable source locations

Use the active checkout for repository files; do not record a developer's absolute
home or workspace paths in checked-in files. XMage source audits use
`RUSTMAGE_XMAGE_SOURCE` when configured. Otherwise they look for a sibling `mage`
checkout, including beside the primary repository when run from a linked Git
worktree. The `set_status` CLI also accepts an explicit `--xmage` path. A source
checkout is required for provenance checks; relocating Rustmage must not silently
skip those checks.

Historical evidence uses `<cache>`, `<temp>`, `<worktree>`, `<checkout>` and `<xmage>` as
location placeholders. These identify the original artifact suffixes without
binding the repository to one machine. Artifact hashes and measurements remain
unchanged. Actual libraries remain at their existing paths; the notation change
does not move or delete artifacts.

## Rules names and catalogue identities

`Card::name()` is a shared string containing a definition's actual rules name;
`Card::identity()` is its unambiguous catalogue/serialization key. Different token
recipes can share one rules name, and fixed offspring copies keep their source's
name. Do not compare factory keys when a rule asks about names.

Live object-name rules use the central characteristic-world name query, including
speculative funding worlds. `ObjectView.name` freezes the effective string for
observations and last-known information; `ObjectView::name()` reads that captured
value. Names are shared string references, with no allocated string in each live
object. Future name-changing effects must extend the characteristic query rather
than changing physical metadata or recomputing a departed object's name. This
query does not itself admit new copy, rename or named-ability suppression cards.

## Named counters

Live `Counters` owns sparse, canonically ordered values for incubation, bait,
page, divinity, revival and soul counters. Use `Counters::of(kind, amount)` for a
single-kind action payment and `get(kind)` to inspect any kind. Mixed payments
can also be deserialized from the typed `named` list; zero, duplicate and hot-kind
entries there are rejected. Empty cold storage is omitted from JSON.

The five existing counter fields stay inline. The additional runtime storage is
one nullable pointer regardless of how many named kinds are supported; it owns a
boxed vector header and its element buffer only when a named kind is present.
Removing the last named counter releases both. `Game::state_bytes()` includes
initialized sparse payloads in objects, announcements, replacement and entry
batches, consistently excluding unused vector capacity. Snapshots and frozen
object views clone that payload; feasibility queries borrow their goals and
available counter sets.

Immutable card initializers retain their existing five-field `Counters` spelling
through a private `InitialCounters` alias in generated module imports. New named
entry counters use typed entry-counter rules or the surrounding entry effect;
this storage change does not admit the six Foundations consumers or implement
their remaining triggers and costs. In particular, adding a counter as an
activation cost still needs its own rules implementation and tests.
