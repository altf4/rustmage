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
validator also limits the Python API suite to 30 seconds and the full CI
validation workload to nine minutes. A failing timeout terminates the process
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

The standalone runner retains its target for local iteration; remove disposable
outputs only after it terminates. The full validator uses its own unique target
and cleans completed non-library outputs. Every Cargo invocation must remain
inside `scripts/resource_guard.py`; direct Cargo compilation fails closed.

Use `python3 scripts/check_sources.py` for the production metadata and source-pin
check. `--metadata-only` is an explicit local convenience when the external
XMage repository is unavailable; it does not replace full source verification
in acceptance or CI. A focused source check is faster and clearer than separate
unit tests repeatedly reconstructing the entire catalogue.
