# Card-test crate split experiment

On 2026-11-01, an isolated prototype reduced build-plus-test time after a card
definition edit from **24.89s to 6.29s (75%)**. A test edit fell from **25.05s to
2.34s (91%)**. An engine edit took **26.44s instead of 25.14s (5% slower)**.
These are medians of three paired samples, including test execution.

| Change before running the suite | Existing single crate | Split prototype |
| --- | ---: | ---: |
| Engine implementation | 25.14s | 26.44s |
| Arc Reactor's typed card definition | 24.89s | 6.29s |
| One scenario's Rust body | 25.05s | 2.34s |

The earlier matched benchmark measured 27.83s for the old test layout and 24.89s
for the rebuilt single-crate suite. The current paired control agrees with that
rebuilt-suite result. The split's benefit comes from reusing engine compilation
while changing cards or tests. It does not improve an uncached engine build.

## Scope and controls

Both variants ran the exact same **91 scenarios**, exercising the same **186
admitted physical cards**. Each scenario's actual game-object card-ID set matched
the earlier benchmark, not just the total card count. All other test entry points
were removed before compilation in the archived source snapshot. The full
catalogue and all **5,487 compiled definitions**, including tokens and unsupported
entries, remained present in both variants.

Both used rustc 1.98.0, optimization level 0, no debug symbols, no incremental
compilation, one Cargo job, one test thread, the shared 6 GiB memory limit, and the
same two-CPU quota. Every timed build reused the existing Cargo target and cached
third-party dependencies. **No clean build or new cold-dependency build ran.**

The experiment used compile-visible, outcome-preserving edits: a constant-valued
expression in Arc Reactor's rules, an unused local in one scenario, and a small
engine function. Inputs stayed fixed until each consuming process terminated.
Builds alternated between layouts. Identity tracing ran during preflight and was
disabled during the 18 timed runs. All 20 runs passed every scenario.

Cargo's artifact records establish that card edits reused the engine and other
card pack, while test edits reused every production crate. Engine edits rebuilt
their dependents. The measurements include that linking and rebuilding work.

## Prototype organization

The unpublished internal crates are:

- `rustmage-engine`: rule types, engine, protocol, and policy; no dependency on
  the per-card implementation crates.
- `rustmage-cards-msh`: the development set's 278 authored definitions.
- `rustmage-cards-other`: the other 5,209 definitions, kept as one control pack.
- `rustmage-card-tests`: the unchanged benchmark scenarios and fixture helpers.

The public `rustmage` package assembles an immutable catalogue and links the
pieces. The prototype uses one internal Rust link symbol to supply that catalogue
to the engine, avoiding runtime initialization and per-set engine monomorphization.
This is static linking, with no runtime plugin or dynamic card loading.

A separate consumer application declared only `rustmage` and successfully used
`Card`, `Config`, `Game`, and ordinary game actions. The Python binding's three
contract tests passed against a single `librustmage.so`. Its dynamic dependencies
were only the platform libraries `libgcc_s.so.1`, `libc.so.6`, and the ELF loader;
there were no additional Rust shared libraries to ship or load. This consumer
check was separate from the timing samples.

## Limits before production integration

This is a working benchmark prototype, not a completed production migration.
It exposes internals in an unpublished engine crate so the original scenarios can
run unchanged across the crate boundary. Production needs a narrow test-support
interface and deliberate public reexports, versioned internal packaging, and CI
cache handling that preserves compatible engine artifacts across set additions.

The fixed-roster prototype retains generated identity constants, `Card::ALL`,
and name-only metadata in the engine. Changing those inputs can still rebuild the
engine. Production must move remaining catalogue-dependent inputs out of that
compilation boundary; these results establish iteration savings for the fixed
benchmark roster. They do not establish the cost of admitting entirely new
identities, final 3,007-card CI time, or optimized gameplay throughput.

The complete card-coverage backfill remains unfinished in the existing draft PR.
The normal checkout's production crate layout is unchanged by this experiment.

## Reproduction and evidence

The machine-readable summary is [test-build-experiment.json](test-build-experiment.json).
The prototype materializer is
[`card_test_crate_split.py`](../scripts/experiments/card_test_crate_split.py), and
the paired workload runner is
[`benchmark_crate_split.py`](../scripts/experiments/benchmark_crate_split.py).
Neither participates in ordinary CI. The materializer was separately checked by
compiling its output and passing all 91 scenarios with matching per-scenario IDs.

Complete raw reports, per-scenario identities, compiler freshness records,
consumer checks, exact source archives, and the immutable shared library live in:

```text
~/.cache/rustmage-card-coverage/split-comparison-20261101-b/
```

The input archive is
`~/.cache/rustmage-card-coverage/matched-comparison-20261101-b/new-source.tar.gz`
(SHA256 `5ba4dcee4e04b49254cdcbe02aeab2a5f949257533f9afe36ad4e56a8fca7ff3`).
Restore its `new-source` directory as `monolith` in a disposable directory, then:

```sh
python3 scripts/experiments/card_test_crate_split.py \
  --source /tmp/experiment/monolith --output /tmp/experiment/split
```

From `/tmp/experiment/split`, enter the required bounded runner using absolute
paths to the experiment scripts and a fresh report directory under `~/.cache`:

```sh
python3 scripts/resource_guard.py -- python3 /path/to/benchmark_crate_split.py \
  --root /tmp/experiment \
  --previous /path/to/matched-comparison-20261101-b \
  --report /path/under/home/cache/new-comparison \
  --target-dir /path/to/existing/warm/target
```

The runner deliberately edits the disposable copies, requires their previous
jobs to finish, checks every expected scenario and card identity, and rejects
unexpected dependency recompilation or missing engine reuse. It never cleans
Cargo outputs. Preserve any active job's inputs and outputs until it terminates.
