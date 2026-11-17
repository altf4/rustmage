These unmodified request traces reproduce [issue #684](https://github.com/altf4/rustmage/issues/684), captured against revision `0ddbc78533db881eee69004ee319aa409a84c2df`.

- `enigma-requests.jsonl`: 740 requests; the final P0 observation on turn 12 stalled while checking The Enigma Jewel's craft cost. SHA-256: `dd6ccd886c8e88f2296f3a3a8c404036edaea1de3d999bb86185e6b85e24b784`.
- `groves-bounty-requests.jsonl`: 731 requests, including repeated scalar-X affordability probes. SHA-256: `655d93d93f62cbb6210daa20164d07fab2bd2d2e49370e51b9097dc1c4775599`.

The first record embeds the exact decks, seed and starting player. Its historical `library` field is provenance only; the historical index traces must not be replayed against the changed action contract. Both outside-game pools are null.

Run the complete traces and small deterministic search-expansion/payment regressions with:

```sh
python3 scripts/resource_guard.py -- cargo test --test affordability_performance
```

The `*-semantic.jsonl` files translate each historical indexed step through the
preserved baseline library at source `3a2ac6bd054304ca1620eac903c16cf49e789217`,
SHA-256 `f3efbda1919d3c24395172d8f2af45aa506b381f45fa61046c86fc29cdd11cff`.
They store explicit actions and the baseline final public boundary; the regression
replays these actions under `staged_attempts_v1` and compares that boundary.
`scripts/translate_action_trace.py` requires the baseline hash and refuses to
overwrite an output. The original captures are unchanged.

The small negative cases admit attempts and assert zero projected payment moves,
steps, and continuation plans instead of a machine-dependent elapsed-time
threshold. Positive cases execute Jewel's restricted-mana payment, exile four
materials, transform, and compare owned raw/index branches and restoration.
Linked-source and attachment selections are provisional and perform no future
payment search.

These two captures do not cover the seven remaining timeout cases in the
November 16 collection comparison. That comparison contains receipts rather
than executable traces. Their capture/minimization and a collection panel with a
caller trained for the new contract remain separate benchmark work; these tests
do not establish a completed-games-per-minute improvement.
