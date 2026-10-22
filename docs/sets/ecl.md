# Lorwyn Eclipsed (ECL)

## Scope and baseline

Selected set: Lorwyn Eclipsed. Preserve the repository's explicit tabletop/MTGO
Standard snapshot of 2026-10-15; this run does not rotate the format.
Started 2026-10-22. Integration branch: `codex/ecl-complete-set`, base `61c61b69`.
Tracking issue: https://github.com/altf4/rustmage/issues/462. No implementation PR until all set gates pass.

Pinned XMage revision: `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.
Membership: [frozen manifest](../../data/sets/ecl.json), 273 physical identities
from 413 pinned XMage printings; zero identities unresolved against pinned facts.
Independent Scryfall set reconciliation matches all 273 physical identities (2026-10-22); see [reconciliation evidence](ecl-membership-reconciliation.json). Initial admission: 107/273;
166 gated. Admission is inherited coverage, not whole-card certification.
Oracle facts: `data/standard/cards.json`; canonical ownership:
`data/catalogue-identities.json`. Banned members remain in scope.

## Card obligations and evidence

The frozen manifest accounts for every identity. The [per-card obligation ledger](ecl-obligations.json) records every face and printed rules line with canonical ownership. All inherited cards still require evidence audit.

## Batches, reviews and game coverage

Parallel author ownership (isolated worktrees, base `61c61b69`):

| Owner | Cards | Tests | State |
| --- | --- | --- | --- |
| ecl_black_red | Exactly black or red | `tests/ecl_black_red.rs` | Source audit |
| ecl_white_blue | Exactly white or blue | `tests/ecl_white_blue.rs` | Source audit |
| ecl_green_colorless | Exactly green or colorless | `tests/ecl_green_colorless.rs` | Source audit |
| Integrator | Multicolor, source reconciliation, generated links | To be recorded | Source audit |

Shared engine edits require explicit ownership coordination. No final reviews,
set-only game acceptance, or set simulation measurements yet.

## Measurements

Newly verified whole cards: 0. Implementation throughput pending a measured
interval. Set simulation throughput unmeasured. Build resource preflight passed
under the mandatory guard (6 GiB shared cgroup), without allocation probes.

## Resume here

Integration branch `codex/ecl-complete-set`; initial source pins above. Authenticate
membership with `python3 -m scripts.set_status ECL --check`. Reconcile independent
physical membership before certifying denominator. Finish author batches, integrate,
regenerate links, run focused tests and `python3 scripts/validate.py --release`.
Do not open a partial PR. No active validation or required shared libraries yet.

## Initial integrated implementation checkpoint (2026-10-22)

Current admission flags: **159/273**, comprising **107 inherited** and **52 newly
implemented/tested physical cards**. The other **114 remain gated**. No inherited
card has been newly certified merely from its admission flag. All 52 new names
and measurements are in [validation evidence](ecl-validation.json); the per-card
obligation ledger distinguishes them from inherited coverage. Three new token
recipes (Shapeshifter, Treefolk, Mutavault) are tested dependencies, outside the
physical membership denominator.

Shared mechanics now include non-targeting blight costs versus effects, exact tap
witnesses during mana funding, entry-time creature history, source-counter guards,
shared creature-type targets, temporary type replacement, immediate blink through
owned entry continuations, target-controller exile/counter follow-ups, preserved
card types under creature Auras, chosen-player creature/token groups, live block
requirements, vivid mana production, attached all-creature-types effects and
filtered temporary keyword groups. Four Commands cover all six printed mode pairs.

All 72 integrated focused scenarios pass. Ten natural ECL-only Standard games
cover every color, ten terminal outcomes, ten executed blight choices, six private
choices and 100 owned raw/index branch/replay/restore checkpoints. The initial
measured debug workload performed 5,719 decisions in 1.777491 seconds (3,217.455
choices/second including observations, assertions and snapshots). These games
are initial regression evidence, not a claim to full-set gameplay acceptance.

Full release validation is the next gate. Final set reviews, upstream merge and
full-pool acceptance remain pending. Ordinary engineering remains for the gated
cards, including transform, evoke, behold/champion, conspire, proliferate, some
copy effects and chosen-type interactions; these are work to implement, not
missing-permission blockers. Author worktrees continue independently on wither,
filtered private discard and chosen-type consumers.
