# FRA set-only complete-game workload draft

The isolated draft defines six 60-card Standard decks using only physical FRA
identities, with 28 in-set basic lands, at most four copies of each other card,
all five colors, both deck seat assignments and both starting players. It uses the existing HOB
legal-action policy and its declaration-cancellation handling. Verify mode
requires natural terminal outcomes, sampled wrong-actor/owned/raw/index/restore
checks, observations for both players and full deterministic replay. Benchmark
mode separately measures construction, legal actions, gameplay and observation;
it does not include verification overhead or model inference.

The acceptance entry point first requires all 285 distinct physical Oracle
identities to resolve to the matching supported catalogue card. Missing identities
are reported; neither verification nor benchmark bypasses this gate. The tests
also check both seats for every pairing and that an incomplete pool is refused.
Mechanic telemetry requires observed prepared copies and their actual casting and
observed completing resolution packets, source-attributed empower loyalty counters, Diviner
scry/surveil triggers, scry or surveil completion and listed loyalty activations, plus
actual casts covering all five colors. These witnesses and the proposed decks
are unvalidated design targets, not claims that the policy will reach them; an
unreached witness or decision limit must be investigated and the fixed corpus
adjusted, not waived. The corpus is not exhaustive coverage of every card.

The current-pool gate and seat checks are linked into ordinary assurance. The
full-game test reports an explicit non-acceptance skip while the pool is incomplete,
and runs automatically under routine validation once all identities are supported.
The standalone verification and benchmark both enforce the same gate. Telemetry values are deduplicated existence indicators, not complete event counts.
No Cargo, games, replay or benchmark has run for this draft. Run the guarded verification and a separate
release benchmark with fixed inputs, retain evidence, and obtain the required
independent nonauthor reviews before claiming completion or throughput.
