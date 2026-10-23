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
`python3 scripts/validate.py --release`. The validator runs all native rules and
full-game tests with `cargo test --release --lib --bins --tests --benches`, plus Clippy,
format/link consistency, provenance tests and the existing native smoke benchmark.
Clippy checks all targets, including examples; the examples contain no tests and
are excluded from optimized test-executable builds. Without `--release`, native
tests use the debug test profile with all targets. It reports fixed inputs, coverage and cleanup and preserves every
shared library at its exact path/hash. The Freeform smoke benchmark is a limited
regression workload, not full-set or all-Standard acceptance.

[Compiler-removal proof](compiler-removal-results.json) records exact equality of
the compiled before/after definitions, all helper values and immutable globals.
The all-Standard goal remains open; finish one complete set with full-game tests
before advancing to the next set.

Each paired record pins both the full Oracle facts and its reviewed physical
metadata (including intrinsic colors). Changes to costs, types or legality in the
identity manifest require an explicit per-card provenance update; the linker
rejects accidental drift. For complete physical entries with ordinary numeric,
W/U/B/R/G/C and X mana symbols, it also verifies the fixed typed mana components
against the printed cost. Hybrid and other special symbols are explicitly skipped. Synthetic entries carry the same metadata pin.

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

Interchangeable Oracle names use `CardName::matches` or the shared internal
`card_names::same_name`; grouping/counting uses `card_names::comparison_key`.
The small pinned name-pair table follows CR201.3 and CR201.4g, including each
face separately. It affects comparisons without rewriting an object's name or
the ordinary string chosen in an action. CR201.6 cosmetic alternate titles are
not selectable names. Physical catalogue aliases still resolve to one definition;
this metadata does not admit playable behavior for cards outside the pool.

## Named counters

Live `Counters` owns sparse, canonically ordered values for incubation, bait,
page, divinity, revival and soul counters. Use `Counters::of(kind, amount)` for a
single-kind action payment, chain `.with(kind, amount)` for a mixed payment,
and use `get(kind)` to inspect any kind. Mixed payments can also be deserialized from the typed `named` list; zero, duplicate and hot-kind
entries there are rejected. Empty cold storage is omitted from JSON.

Live objects retain five inline `Copy` counter fields (40 bytes) and have no
counter-owned allocations or destructor. Named counters live in an optional
copy-on-write State buffer, sorted by card slot and qualified by exact incarnation.
No row or buffer is allocated until named counters exist. Actual zone changes
clear the old row in every zone, including exile; removing its final named counter
removes the row and releasing the last row releases the buffer. Named-only writes
leave the hot object buffer shared across owned branches.

Owned public `Counters` values, payments and frozen object views keep the same
five fields plus an optional boxed canonical named list. Use `Game::counter_count`
or the borrowed characteristic-world query inside rules code. Speculative funding
owns a copy-on-write overlay, including exact counter reservations in its key.
`Game::state_bytes()` includes initialized sparse rows and owned announcement,
replacement and entry payloads, excluding unused vector capacity consistently.

Immutable card initializers retain their existing five-field `Counters` spelling
through a private `InitialCounters` alias in generated module imports. Use
`CardExtras::fixed_entry_counters` for fixed printed counters of any typed kind;
these join the existing pre-entry counter event and its replacements. Dynamic
chosen-type entry counters use `chosen_entry_counters`. Counter storage alone
does not implement a card's remaining triggers and costs. In particular, adding a counter as an
activation cost still needs its own rules implementation and tests.

## Chosen values and fixed control effects

Use `CardExtras::entry_choice` for mandatory color or creature-type choices made
as a permanent enters. These suspend the owned entry batch before its counter
replacements and commit; they are not triggered abilities. `ChoiceColor` permits
only the five colors, and `CreatureType` validates names against the pinned type
list. Chosen values belong to an exact object incarnation, survive control
changes, and clear on zone changes. `ObjectView.entry_choice` exposes the saved
value for visible objects; `Observation.entry_value` describes a pending visible
choice. Submit `Action::ChooseEntryValue` through the normal legal-action API.

Reuse `ManaProduction::ChosenColor`, `CastFilter::SourceChosenColor` and typed
chosen static fragments rather than branching on card names. Chosen creature
subtypes participate in layer four; chosen stat bonuses use the shared actual
and speculative characteristic queries. These primitives do not supply restricted
mana spending or every effect associated with choosing a value.

`Instruction::GainControl` performs a fixed control transfer to its selected
player. It supersedes earlier temporary transfers even when the recipient already
controls the permanent. It does not implement control that lasts only while an
Aura remains attached. `CreatureDiesAttackState` tests the frozen death event,
and `MayPayControllerLife` checks affordability before offering its branch; its
program must explicitly perform the life loss and subsequent effects.


## Continuous control and attached-creature death

Use a static group beginning at `AbilityLayer::Control` with
`StaticAbilityFragment::ControlAttached` for an Aura whose controller continuously
controls its enchanted permanent. The sparse control timeline retains exact-object
baselines and fixed/EOT effects; the live controller is its derived cache. Control
assignments only create dependencies when they would change what another effect
does. Dependencies are reevaluated after each application, with timestamp ordering
for actual loops. Whole simultaneous movement batches materialize the final
controllers once, avoiding intermediate combat removal or summoning sickness.
Speculative funding recomputes its private projection after virtual departures.
An `AuraSpec` kind mask of zero means any permanent, including Battles.

`TriggerCondition::AttachedCreatureDies` belongs to the attachment and tests its
frozen exact recipient, not the recipient's own abilities. For an Aura returning
itself, use `Instruction::ReturnSourceFromFirstPublicZoneToHand`. This narrowly
opts the packet into the existing `source_move` context for its first actual
battlefield-to-graveyard/exile movement, including a later Aura SBA. It binds only
once, preserves source LKI and rejects subsequent departures/reentries. It can
also return the unchanged live source. This tracking does not rebind unrelated
death observers or infer a zone-change generation.
