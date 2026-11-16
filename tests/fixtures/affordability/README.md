These unmodified request traces reproduce [issue #684](https://github.com/altf4/rustmage/issues/684), captured against revision `0ddbc78533db881eee69004ee319aa409a84c2df`.

- `enigma-requests.jsonl`: 740 requests; the final P0 observation on turn 12 stalled while checking The Enigma Jewel's craft cost. SHA-256: `dd6ccd886c8e88f2296f3a3a8c404036edaea1de3d999bb86185e6b85e24b784`.
- `groves-bounty-requests.jsonl`: 731 requests, including repeated scalar-X affordability probes. SHA-256: `655d93d93f62cbb6210daa20164d07fab2bd2d2e49370e51b9097dc1c4775599`.

The first record embeds the exact decks, seed and starting player. Its historical `library` field is provenance only; tests use the current compiled engine. Both outside-game pools are null.

Run the complete traces and small deterministic search-expansion/payment regressions with:

```sh
python3 scripts/resource_guard.py -- cargo test --test affordability_performance
```

The small negative cases assert zero expanded states instead of a machine-dependent elapsed-time threshold. Positive cases execute Jewel's restricted-mana payment, exile four materials, transform, and compare owned raw/index branches and restoration. Linked-source and attachment cases retain the full search.
