# Sampled token bases and exact Endure recipients

This shared checkpoint supports Abzan Monument and Warden of the Grove without
changing their gated card definitions. Evidence and exact source/run hashes are in
`tdm-dynamic-tokens-evidence.json`.

`CreateSizedToken` samples one nonnegative amount and creates one token with that
copiable base power and toughness. It uses the existing token replacement and entry
transactions. A flattened `CopyRecipe` retains the sampled base through copying;
later explicit base exceptions replace it, and P/T CDAs do not overwrite it.
Ordinary counters and later modifiers remain separate. The compact size has no
per-token heap allocation and supports the engine's nonnegative i128 stat range.

`EndureScaled` samples its amount once and identifies either the exact ability
source or the exact entering recipient. The current recipient controller chooses
and places counters; departure uses that recipient's battlefield controller LKI.
The source quantity separately uses its own current counters or final counter LKI.
A new incarnation cannot replace either old witness. Zero does nothing, and an
impossible counter placement creates the correctly sized Spirit automatically.
Fixed Endure retains its printed token factories and existing choice behavior.

The10 focused groups use real paid typed abilities and spells, including both
seats, response changes, token/counter replacements, copied values, capacity-error
rollback, actor privacy and owned raw/index/restore branches. The same assembled
inputs pass85 other TDM groups and52 relevant existing copy/token scenarios.
The147 selected groups took144.699s of compilation and0.412s of bounded scenario
execution; the cold shared engine build accounted for112.539s of compilation.
These timings describe this rules workload and are not simulation benchmarks.

Actual wire extensions still require the common schema/trace checkpoint. Full
engine and set acceptance remain open; these tests do not admit either card.
