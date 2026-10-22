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

## Current implementation checkpoint (2026-10-22)

Admission flags: **207/273**, comprising **107 inherited** and **100 newly
implemented/tested physical candidates**; **66 remain gated**. Inherited evidence
still needs audit. No final published reviews or newly merged cards. All new
names and per-card obligations appear in the linked JSON ledgers. Three tested
synthetic token recipes are outside the physical denominator.

Three isolated authors own exactly black/red, white/blue, and green/colorless
cards respectively. Their focused modules are `src/engine_ecl_black_red_tests.rs`,
`src/engine_ecl_white_blue_tests.rs`, and `src/engine_ecl_green_colorless_tests.rs`.
The integrator owns multicolor cards, shared coordination, source reconciliation,
`src/engine_ecl_core_tests.rs`, `src/engine_ecl_commands_tests.rs`, and generated
links. All five Incarnations are integrated; root now owns the seven transforming cards. Foraging Wickermaw, Harmonized
Crescendo, Thoughtweft Imbuer and Lasting Tarfire are integrated; the203-card
Clippy check needed the now integrated Brigid consumer. Both Brigid faces,
Warren Torchmaster, Shimmerwilds Growth and Kinbinding are integrated; a207-card
release validation is next.

Shared mechanics include mandatory and optional blight costs versus effects,
wither as a damage result, exact mana-funding tap witnesses, entry-time creature
history, chosen-type entry and activation packets/restricted mana/lords,
conditional flash, all five Commands and their six mode pairs, immediate blink
through owned entry continuations, private filtered optional discard,
attached-creature fight, non-target power-difference choices, and conditional
combat assignment using an equipped creature's toughness, evoke with actual mana-spent entry qualification, all-types temporary effects, vivid search/reveal quantities, exact combat-event subjects, owned layer-five color changes, creature trigger grants, and counter/untap prohibitions. Alternative casting
methods preserve printed additional mana-or-blight choices.

## Validation and measurements

The combined focused run at `bb92026c` passed **196 ECL scenarios** in7.24
seconds after linking Doran, Puca’s Eye, Hexing Squelcher, and Blossombind.
Full release validation **passed all eight phases** at `0ae35121`: formatting,
Python/provenance checks, Clippy/all-target compilation, **2,671 native tests**
(one ignored), catalogue checks, **all 22 Foundations game suites**, ECL games,
release build and benchmark. Runtime531.22 seconds; enforced shared memory peak
3.2 GiB. Report `<cache>/rustmage-validation-4cepq26r/report/result.json` records
all required libraries at exact paths/hashes; completed disposable outputs were
cleaned. The prior unrelated-attachment timestamp regression also passed its
exact targeted full-game rerun (56.22 seconds).

The release Freeform regression workload ran100 deterministic games and49,832
decisions in0.248807 seconds: **200,283.61 decisions/second**,401.92games/second;
clone208.32ns, restore181.27ns. It includes native policy/observations, excludes
JSON/model/tensor work, and is not an ECL acceptance measurement. Compiler,
hardware, revision and workload context are recorded in the validation JSON.

Ten natural ECL-only Standard games covered all colors, ten terminal outcomes,
ten blight choices, six private choices, and 100 owned raw/index branch/replay/
restore checkpoints. Debug workload: **5,719 decisions in 1.777491 seconds**,
**3,217.455 decisions/second**, including policy, observations, assertions, and
snapshots. This is subset regression evidence, not complete-set acceptance.
See the JSON for seeds, revision, compiler, hardware, and artifact reports.

The validation JSON records the measured implementation interval and current cards/hour for88 newly locally tested physical candidates. This excludes inherited certification, independent final review, and upstream merge.

## Resume and remaining work

Branch `codex/ecl-complete-set`; issue [#462](https://github.com/altf4/rustmage/issues/462).
Authenticate membership using `python3 -m scripts.set_status ECL --check`.
Finish current shared-mechanic batches, integrate, regenerate links, run combined
focused scenarios and full release validation, then extend the natural games to
verify every printed obligation. Check active jobs before touching their inputs;
poll the same process until terminal. No early or partial implementation PR.

Ordinary engineering remains for gated cards, including transform,
behold/champion, conspire, proliferate, and some copying/chosen-type effects.
These are implementation work, not external permission blockers. Authors are
working on remaining combat/type interactions, vivid consumers, resolution hand exile, and behold/champion. Full pool behavior, interactions,
performance, two independent nonauthor final published-head reviews and merge
remain open.
