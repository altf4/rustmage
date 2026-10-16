# Entry-trigger suppression

Issue: https://github.com/altf4/rustmage/issues/182

Three complete bodies bring coverage to **1,344/5,164** Standard cards,
with **3,820 unsupported**, pinned to October 15, 2026. Six non-Standard
regression identities, 57 token factories and one emblem remain separate.
The full Standard migration remains incomplete.

| Card | Complete implementation |
| --- | --- |
| Torpor Orb | Two-mana artifact; creatures entering cause no abilities to trigger |
| Doorkeeper Thrull | 1W 1/2 with flash and flying; artifacts and creatures entering cause no abilities to trigger |
| Karn, Argent Defender | Two-mana legendary artifact creature, 1/3 Golem; artifacts and creatures entering cause no abilities to trigger |

`entry-suppression-admissions.json` records the three complete Oracle bodies and
**16 SHA-256 entries across ten unique Java files** at revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All constructors, the complete
`DontCauseTriggerEffect`, its rule-modifying base, the battlefield static ability
and flash/flying implementations were read. Torpor Orb's default creature filter
was checked in `StaticFilters` and `FilterCreaturePermanent`. Recorded blobs match the pinned revision and checkout.

## Event semantics

A definition-only kind mask identifies creatures (1) or artifacts/creatures (17).
The existing battlefield static-source index registers these sources and removes
them on departure. No mutable state field, per-game flag, observation schema or
ABI is added. Schema remains **51**, C ABI **1**, control protocol **2**.

The engine computes the union once per entry batch, after every entrant and its
static abilities have been installed. Each matching entry is prevented from
creating any triggered ability, regardless of the observer's controller, type,
self/other status or target requirements. Artifact creatures match the union once.
Simultaneously entering suppressors therefore apply to the whole batch.

Previously queued triggers still resolve. Leaving removes the prohibition only
for future events; returning creates a new incarnation. A blocked event does not
allocate a stack identity or consume a once-per-turn trigger use. Death halves,
attacks, steps and casts retain their own dispatch paths. Noncreature artifact
entries work under Torpor Orb, while Thrull and Karn block them. Ordinary lands
and enchantments continue triggering, and token entries use the same rules.
Entry counters and tapped entry remain replacements and apply normally.

These semantics agree with the official [Torpor Orb release notes](https://magic.wizards.com/en/news/feature/outlaws-of-thunder-junction-release-notes)
and [Doorkeeper Thrull release notes](https://magic.wizards.com/en/news/feature/murders-at-karlov-manor-release-notes).
The current catalogue gates type-changing, copying, control-changing and ability
removal; adding those mechanics will require their evaluated characteristics to
feed this index and mask as well.

## Complete-body compilation

A closed extractor recognizes only the two exact prohibitions, with the exact
optional words “the battlefield.” It retains all remaining text for the existing
whole-body compilers. The mask is emitted only when the complete card is admitted.
Unknown subjects, controller restrictions, dying clauses and trailing effects do
not silently disappear. Normal-permanent/layout and cost checks still apply.
Only these three executable definitions change; existing executable definitions,
card identities and token identities are unchanged.

## Validation and native workload

Full checks passed: **1,138 Rust tests**, **217 compiler/data tests**, **146
release Python tests**, catalogue consistency, formatting, all-target Clippy
with warnings denied and diff checks.

Fourteen native tests cover both seats, complete casts, both kind masks, all
observers, simultaneous ordering and mixed batches, real flash responses to queued
triggers, source departure/reentry and clone isolation, entry replacements,
death/attack/step/cast triggers, token batches, once-per-turn use, other continuous
effects and capacity rollback. Public actions check clone and restore equality
and stale action-index rejection. Two compiler tests cover exact extraction,
residual preservation and complete-card negative compilation. The release Python
test runs nine source/entrant scenarios with cloned/restored observations, exact
trigger presence/absence and allowed draw/life/token effects.

Run the legal 60-card/four-copy workload:

```sh
cargo run --release --example standard_entry_suppression -- 100
```

Three deck families mix each suppressor with Helpful Hunter, Mephitic Draught,
Pactdoll Terror, Hopeful Vigil, Callous Inspector, Diregraf Ghoul, Burdened
Stoneback and Fear of Lost Teeth, plus 24 lands including Tranquil Cove. The
policy varies early/late suppressor casts and leaves activated counter removal
unused so independent stat/keyword witnesses have fixed expected bases.

An independent ledger reconstructs expected entry and death triggers from public
object transitions and card names, then reconciles their exact source incarnations
and ability indices with newly visible stack, waiting and announcement entries.
It checks missing as well as unexpected triggers at priority boundaries. This
witness does not read engine trigger conditions or suppression definitions.

Two deterministic runs reproduced:

- **56,153 decisions**, suppressor casts **[65, 56, 62]** in table order;
- blocked entry triggers **[107, 115, 75]**, allowed **[293, 226, 242]**;
- **264 death triggers**, **1,343 land**, **1,471 creature** and **569 artifact entries**;
- **211,594 public stat/ability pairs**, **35,224 clone/restore replay boundaries**;
- **21,164 maximum initialized logical bytes**.

The 1,000-game native smoke benchmark measured 571,049 decisions, approximately
357,499 decisions/s and 626 games/s, 132 ns clone and 127 ns restore on this host.
Initial logical state remains **19,784 bytes**, excluding allocator/capacity,
immutable definitions, observations and separately retained snapshots. Timing is
host dependent and includes the smoke policy and observation construction; no
model inference, tensor encoding or JSON is included. This is not an XMage parity
or speed comparison.

Catalogue fingerprint:
`1a8bcb368a9208f8d2aa2c9def82b8f33c2e2cd89b308fe0c0339d690a769a85`.
