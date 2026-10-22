# Lorwyn Eclipsed (ECL)

## Scope and baseline

Selected set: Lorwyn Eclipsed. Preserve the repository's explicit tabletop/MTGO
Standard snapshot of 2026-10-15; this run does not rotate the format.
Started 2026-10-22. Integration branch: `codex/ecl-complete-set`, base `61c61b69`.
Tracking issue: https://github.com/altf4/rustmage/issues/462. No implementation PR until all set gates pass.

Pinned XMage revision: `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.
Membership: [frozen manifest](../../data/sets/ecl.json), 273 physical identities
from 413 pinned XMage printings; zero identities unresolved against pinned facts.
Independent printing reconciliation is pending. Initial admission: 107/273;
166 gated. Admission is inherited coverage, not whole-card certification.
Oracle facts: `data/standard/cards.json`; canonical ownership:
`data/catalogue-identities.json`. Banned members remain in scope.

## Card obligations and evidence

The frozen manifest accounts for every identity. Per-card obligation/evidence
ledger is being prepared; all inherited cards require evidence audit.

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
