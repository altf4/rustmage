# Aetherdrift implementation

All 276 Oracle identities in the frozen DFT pool are implemented and passed full-set acceptance. Cosmetic variants (553 printings), basics and reprints count once per Oracle identity. The initial branch admitted 119 members; 157 additional identities now have complete typed behavior and independently expected outcomes.

Definitions are directly authored Rust with paired Oracle/XMage provenance. The complete source translations, inherited definitions and affected shared mechanics received two independent nonauthor audits. The published PR head receives a separate final review. No English compiler or runtime language model participates in gameplay.

Shared rules remain in `crates/engine`; card behavior remains in generated per-set compilation boundaries and card scenarios in separate integration targets. Engine changes and discovered dependencies were batched, tested and reviewed before dependent card authoring resumed. All builds used the shared resource guard and stable warm cache.

Shared mechanics include speed and max-speed layers, exhaust events and reuse, crew/saddle/exert payment facts, Embalm grants, linked and simultaneous entry/copy behavior, private exile/play rights, frozen source outputs and exact nested mana-payment continuations, controller-qualified target groups, source-independent delayed effects and additional combat scheduling. The complete dependency inventory and historical checkpoints are in `aetherdrift-engine-checkpoint.md` and the freeze receipts.

Acceptance evidence:

- `aetherdrift-focused-card-evidence.json`: all 276 definition hashes, printed-obligation mappings and passing card outcomes; 340 focused card/provenance cases.
- `aetherdrift-engine-freeze.json`: 387 focused shared-engine cases, strict trace compatibility, two independent source/runtime approvals and warm-cache proof.
- `aetherdrift-game-acceptance.json`: 80 fixed legal DFT-only games, 67,835 actions replayed, 3,364 owned branches, natural terminal outcomes, five spell colors and all 15 required mechanic witnesses. A separate same 80-case benchmark excludes replay/owned checks and explicitly records its profile and workload.
- `aetherdrift-acceptance.json`: full release validator, 3,777 native cases, tooling/source/format checks, production library, Freeform simulation sample, Python binding smoke and immutable library hashes.
- `aetherdrift-readme-count-audit.json`: reproducible 3,438/5,164 accepted-union count and 9/20 completed sets, retaining accepted upstream ECL/TMT support.
- `aetherdrift-authoring-throughput.json`: measured 157 newly completed local candidates across the recorded parallel authoring/review/engine-reentry interval, including waiting rather than claiming isolated typing speed.

The completed-set effort uses one branch and one PR. No card obligation or engine blocker remains. The broader all-Standard objective remains open; the DFT workload does not certify every Standard card or choice.
