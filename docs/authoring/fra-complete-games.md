# FRA set-only complete-game workload draft

The isolated draft defines six 60-card Standard decks using only physical FRA
identities, with 28 in-set basic lands, at most four copies of each other card,
all five colors and pairings in both starting seats. It uses the existing HOB
legal-action policy and its declaration-cancellation handling. Verify mode
requires natural terminal outcomes, sampled wrong-actor/owned/raw/index/restore
checks, observations for both players and full deterministic replay. Benchmark
mode separately measures construction, legal actions, gameplay and observation;
it does not include verification overhead or model inference.

The acceptance entry point first requires all 285 distinct physical Oracle
identities to resolve to the matching supported catalogue card. Missing identities
are reported; neither verification nor benchmark bypasses this gate. The tests
also check both seats for every pairing and that an incomplete pool is refused.
Mechanic telemetry requires observed prepared copies, source-attributed empower loyalty counters, selected Fatehold triggers,
scry or surveil completion and listed planeswalker ability activations, plus
actual casts covering all five colors. These witnesses and the proposed decks
are unvalidated design targets, not claims that the policy will reach them; an
unreached witness or decision limit must be investigated and the fixed corpus
adjusted, not waived. The corpus is not exhaustive coverage of every card.

The current-pool gate and seat checks are linked into ordinary assurance. The
full-game test is explicitly ignored until the pool is complete; final acceptance
requires explicitly running that ignored test with its full-pool gate enabled.
The standalone verification and benchmark both enforce the same gate. No Cargo,
games, replay or benchmark has run for this draft. Run the guarded verification and a separate
release benchmark with fixed inputs, retain evidence, and obtain the required
independent nonauthor reviews before claiming completion or throughput.
