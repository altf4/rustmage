# Fast card and interaction assurance

The test suite is organized around observable card behavior and common rules
interactions. It replaces the accumulated development tests rather than
retaining a second legacy suite. Tests arrange a small deterministic game state,
perform the relevant legal actions, and assert the result. A focused scenario
should finish in milliseconds; long self-play runs and broad mutation suites
belong in explicit investigation or benchmarking commands.

Card cases should establish concrete printed outcomes: damage amount and legal
targets, movement between zones, token characteristics, counters, life changes,
activated costs and timing, or the relevant trigger. Table cases reuse one small
fixture when several cards exercise the same rules contract. Metadata and
catalogue smoke checks are useful admission checks, but they do not establish
that every printed ability or card combination works.

Interaction scenarios protect shared mechanics with a few contrasting examples:
protection and removal, countering and stack resolution, combat and keywords,
entry/death triggers, additional or alternative costs, ownership and control,
and delayed cleanup. The public-engine contracts also cover hidden observations,
illegal-action transactionality, stale action indices, independent owned
branches, and snapshot/restore behavior. Python tests concentrate on the FFI and
binding contract already exercised through the native rules engine.

When adding or changing a card, extend the smallest relevant table or scenario
with an assertion for the changed obligation. For a new mechanic, add a direct
success case and a materially different interaction or rejected action. Prefer
an actual card over an artificial effect definition. Choose cases that would
fail if the implementation regressed: merely enumerating legal actions,
successfully loading a card, or reproducing implementation constants is not a
replacement for checking the card's effect.

Keep scenario work bounded. Use fixed seeds, small decks/boards and explicit
maximum action counts. Avoid searches over random seeds, all legal branches or
large card cross-products. A necessary complete-game test must have a fixed
action ceiling and stop when the intended assertion has been established. Do
not add a new bot to reach a state that an existing scenario helper can arrange.

The native runner compiles the library test executable once and records a
flushed start/finish event for every exact test. Every scenario has a 10-second
wall-clock cap and the entire native execution suite has a 30-second cap; both
are independent of compilation. Inspect `report/native.json` for build versus
execution time and `report/native.jsonl` to locate a stalled scenario. The
Python binding suite has a five-second execution budget and a 15-second
worker timeout. The full CI validation workload has a nine-minute limit. A failing timeout terminates the process
tree before cleanup, so it cannot leave a background compiler consuming deleted
inputs.

Run normal acceptance through the resource guard automatically:

```sh
python3 scripts/link_catalogue.py
python3 scripts/validate.py --release --python smoke
```

To run the native suite alone while retaining per-test timings:

```sh
python3 scripts/run_native_tests.py \
  --target-dir "$HOME/.cache/rustmage-tests/target" \
  --report "$HOME/.cache/rustmage-tests/report/native.json"
```

The standalone runner retains its target for local iteration. The full validator
also retains compiler outputs: it uses `CARGO_TARGET_DIR` when set, otherwise a
stable worktree cache under `~/.cache/rustmage-targets/`. To share the authoring
target with native tests, Clippy and release acceptance, select it explicitly:

```sh
python3 scripts/validate.py --release --python smoke \
  --target-dir "$HOME/.cache/rustmage-tests/target"
```

Each invocation still executes every required check and writes a unique
`~/.cache/rustmage-validation-*/report/` directory. Release libraries are copied
into that report's `libraries/` directory before Python checks, and their exact
paths/hashes are retained in `result.json`. Later builds may overwrite the mutable
Cargo outputs without changing those published copies. Targets from the former
validator that contain `report/result.json` are rejected: their original library
paths remain pinned, so choose a new cache instead of overwriting them. Targets
and `XDG_CACHE_HOME` must be outside the source checkout.

Use `--fresh` only for an intentional clean-build investigation. It creates a
disposable target and cleans it after all consuming processes terminate, while
retaining reports and copied libraries. Ordinary runs never clean their reusable
target; remove genuinely disposable outputs only after their users terminate.
Every Cargo invocation must remain inside `scripts/resource_guard.py`; direct
Cargo compilation fails closed.

Use `python3 scripts/check_sources.py` for the production metadata and source-pin
check. `--metadata-only` is an explicit local convenience when the external
XMage repository is unavailable; it does not replace full source verification
in acceptance or CI. A focused source check is faster and clearer than separate
unit tests repeatedly reconstructing the entire catalogue.

## Current coverage and limits

PR #468 established the existing 54 independent Oracle card cases, focused
interaction cases and shared engine contracts. The Marvel Super Heroes rebuild
extends those helpers with four card-family modules and one representative
engine-contract module. It removes the previous MSH development tests and game
corpora. The per-card `msh_*_obligations.json` ledgers distinguish printed rules,
header facts, concrete behavior assertions and shared keyword proofs. See
[`msh-bounded-assurance-rebuild.json`](msh-bounded-assurance-rebuild.json) for the
frozen membership and replacement inventory.

The broad regression table now executes all 2,584 candidate supported physical
cards, including 2,578 Standard cards, using fixed small main-phase, stack or
combat fixtures. Every candidate must be offered and finish its declaration and
resolution. The longest accepted trace uses 15 actions; the limit is 64 and the
fixture object ceiling is 192. A missing cast, unresolved decision or exceeded
budget fails. No card is silently skipped.

`src/assurance/oracle_cases.json` records independently pinned Oracle text, stats,
keyword expectations, damage/draw/pump values and total printed cost for the
simple families. Central interaction cases exercise keyword rules, colored
costs, stack responses, target invalidation, ward, aura suppression, trample and
deathtouch, counters/tokens/replacement effects, simultaneous deaths, layers,
private searches and adventure/omen origins. One fixed complete game has a
256-action ceiling. Expensive owned/raw/index/restore checks run at five
representative decision roots rather than on every card action.

`src/assurance/card_traces.json` captures the unchanged production engine's
single-card cast/entry behavior, including action-journal and both-player final
observation digests, named fixture, action count, status and final zone. These
are regression baselines, not independent proof that every printed obligation
works. In particular they do not exhaust each card's activated/attack/upkeep/death
abilities, all modes or combinations. Those remain reasons to add focused
independent scenarios when changing the relevant behavior. Admission and trace
counts must never be presented as complete all-pool rules certification. The remaining 2,586 Standard identities are still gated in this candidate.
Admission counts and this bounded rewrite do not certify the entire Standard
pool or every cross-card interaction.

The eight Python contracts finish well below their five-second execution budget
and 15-second worker timeout. Infrastructure contracts check actual cgroup limits,
fail-closed compilation, command status, timeout termination, detached-child
cleanup, cache reuse, source drift and immutable library snapshots without OOM
allocation probes. Ignored native
cases execute, and zero/skipped Python or script suites fail acceptance.

For an intentional cast/entry baseline change, first add an independent assertion
for the changed rule. Then record a candidate under the persistent cache:

```sh
RUSTMAGE_RECORD_CARD_TRACES="$HOME/.cache/rustmage-card-traces-candidate.json" \
  python3 scripts/resource_guard.py -- cargo test --lib \
  engine::assurance::regressions::supported_cards_retain_bounded_cast_and_entry_traces \
  -- --exact
```

The recording run still compares with the committed baseline and fails on drift.
After the process terminates, inspect the changed rows and copy the reviewed
candidate to `src/assurance/card_traces.json`. Ordinary validation never updates
baselines. Set `CARGO_TARGET_DIR` under `~/.cache` when doing local development
builds, and keep logs/reports there. Refreshing fingerprints alone is insufficient
review evidence for a changed card rule.

For a wire-schema migration, the same explicit recording command may additionally set
`RUSTMAGE_RECORD_CARD_TRACES_PREVIOUS_SCHEMA` to the former version. It writes a
separate `.previous-schema.json` candidate with the requested observation version projected
back. Compare all inherited full traces with the committed baseline before accepting
wire-only fingerprint changes. The recording run still asserts the ordinary current
baseline and fails on drift; this option does not weaken normal validation.

Power-up wire migrations can be audited with the recording-only
`RUSTMAGE_RECORD_CARD_TRACES_PREVIOUS_SCHEMA=137` option alongside
`RUSTMAGE_RECORD_CARD_TRACES=<cache>/candidate.json`. The previous-wire export
removes only the new `ObjectView` entry-turn/use-history fields and retains
all inherited fields and their serialization order. Ordinary validation still
compares the full current traces strictly, even while recording audit output.

Restricted-mana schema 140 audits retain all existing Power-up fields. Record an
explicit prior-139 candidate when comparing with the second MSH cohort. A
separate prior-138 export can compare the older inherited subset while projecting
only the schema-139 counter-history addition. Power-up fields are removed only
for explicitly requested versions below 138; ordinary validation is unprojected.

The modal-face schema141 cohort records prior140 with every existing Power-up,
counter-placement history and keyword field intact. New optional face/transform
fields are omitted on inherited roots, so the previous-schema export changes only
that version and must exactly match all2453 inherited full traces. That development test module was retired by the bounded MSH rebuild; current
Monica interactions are in `src/assurance/msh_c.rs`. The historical physical
trace and immutable comparison hashes remain in `msh-faces-core-evidence.json`.
