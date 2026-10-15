# Foundation benchmark, October 15, 2026

Five samples of 10,000 complete native games each, following an excluded
1,000-game warm-up. The benchmark subprocess was pinned to logical CPU 0 on an
Intel Core Ultra 5 335. Release build with Rust 1.98.0, thin LTO, one codegen unit.

| Metric | Median | Sample range |
| --- | ---: | ---: |
| Native decisions/second | 2,013,915 | 2,000,011–2,026,981 |
| Complete games/second | 3,633 | 3,608–3,657 |
| Initial-state clone | 82 ns | 79–89 ns |
| Initial-state restore | 74 ns | 74–92 ns |
| Initial-state payload | 2,672 bytes | Same initial configuration |

Both seats used an explicit Freeform deck: 24 Mountains, 20 Lightning Bolts,
12 Goblin Pikers, and 4 Hill Giants. Each sample started seeds 0..9999, produced
5,543,265 primitive decisions, and ended every game naturally. There was no
concession cutoff. A decision may be a pass, mana activation, declaration choice,
or another primitive action, rather than a whole turn or spell.

Timing includes setup, action generation and validation, player-observation
construction and a basic deterministic native policy. It excludes model
inference, tensor encoding, JSON, Python and unsupported rules. The shared-library
Python interface has functional tests, not a throughput measurement here.

Clone/restore are hot microbenchmarks on the **initial** 120-card state only,
100,000 iterations per measurement, and should not be generalized to larger
battlefields or the future full rules engine. Payload counts initialized state
fields and vector elements, excluding spare capacity, allocator overhead, and
search-tree memory. Snapshot creation has the same owned-state copying strategy
but is not separately timed. These numbers do not establish a speedup over
XMage: its workload and implemented mechanics differ substantially.

Raw samples and source revision are in
[2026-10-15-foundation.json](benchmarks/2026-10-15-foundation.json).

Reproduce:

```sh
cargo build --release
python3 scripts/benchmark.py --games 10000 --repeats 5 --cpu 0 --output report.json
```

Choose a CPU available in your process's affinity mask, or omit `--cpu` to run
unpinned. Only the child benchmark is pinned; the caller's affinity is unchanged.
