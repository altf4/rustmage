# Independent review: activated abilities and permanent programs

PR: https://github.com/altf4/rustmage/pull/20
Implementation reviewed: e965f4a; actor-only payment view follow-up efcd29a.

Two independent read-only subagents reviewed this milestone:

- Rules: all 29 new admissions, mana/tap/sickness/haste/self-sacrifice,
  fixed life/counter costs, mixed counter allocations, sorcery/use restrictions,
  deferred commitments, source LKI, independent ability stack identities,
  permanent entry and entry counters.
- Library: cost/target generation, atomic rejection, owned announcement payments,
  usage masks and reset, source incarnations, snapshots, schema 9 privacy,
  native/C/Python integration, whole-text compiler admission and hot paths.

Both reported no actionable findings. Both separately checked the actor-only
pending payment follow-up and confirmed no new issues. The additional view
preserves the choosing player's knowledge of their uncommitted mana/counter
choices without exposing them to opponents.

Validation passed: 103 Rust tests, 24 compiler/importer tests, catalogue
consistency, release build and 12 Python binding tests. The library reviewer
also completed 20 native games with 141 activations. Author checks included
rustfmt and Clippy with warnings denied, plus 100 native supported Standard
games: 48,687 decisions and 892 activations.

Tests exercise tap/sickness/haste, exact payment rejection, source sacrifice and
return, spell-only counter targets, source-counter identity, graveyard selection
before sacrifice, permanent entry/draw/activation, suspended discard replay,
entry counters, any-kind/mixed counter costs, deferred target payments and
actor-only projections. Fixed life/use restrictions have direct kernel and
compiler checks; wider restrictions remain gated.

Executable coverage is 171/5,164 Standard identities. This completes the bounded
counter/fixed-token foundations from #16, while broader counter/token mechanics,
mana abilities, other costs/activation zones/modes, replacements/layers, remaining
card semantics and full RL performance acceptance remain outstanding.
