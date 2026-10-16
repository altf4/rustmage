# Graveyard self-exile activations

Issue: https://github.com/altf4/rustmage/issues/148
Pinned Standard: October 15, 2026. Sixteen complete bodies bring executable
coverage to **1,201/5,164**; **3,963 remain unsupported**. Six regression identities,
55 token factories and one emblem are separate.

## Complete bodies and provenance

`graveyard-activation-admissions.json` records pinned Oracle text and fifty SHA-256
entries across twenty-five unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. The sixteen card implementations,
shared self-exile cost, Renew and sorcery timing classes, and all six referenced
token classes were audited. Two token factories are new; the others are reused.

| Card | Whole body |
| --- | --- |
| Adorned Crocodile | 5/3; death creates 2/2 black Zombie Druid; Renew {B} grants one +1/+1 counter |
| Agent of Kotis | 2/1; Renew {3}{U} grants two +1/+1 counters |
| Beetle, Legacy Criminal | Legendary 3/3 flying; {1}{U} grants one +1/+1 counter and flying until end of turn |
| Bonebind Orator | 2/2; {3}{B} returns another creature card from own graveyard to hand |
| Cryotheory Adept | 2/1 prowess; {3}{U} taps target creature and adds one stun counter |
| Eternal Student | 4/2; {1}{B} creates two 1/1 white/black flying Inklings |
| Faerie Dreamthief | 1/1 flying; entry surveil 1; {2}{B} draws one and loses one life |
| Goldmeadow Nomad | 1/2; {W} creates a 1/1 green/white Kithkin |
| Gravestone Strider | 1/3 artifact creature; once-per-turn battlefield {1} any-color mana conversion; graveyard {2} exiles target card from a graveyard |
| Leering Onlooker | 1/3 flying; {2}{B}{B} creates two tapped 1/1 black flying Bats |
| Rubblebelt Maverick | 1/1; entry surveil 2; {G} grants one +1/+1 counter |
| Stoic Grove-Guide | 5/4 with hybrid spell cost; {1}{B/G} creates a 2/2 black/green Elf |
| Stone Docent | 3/1; {W} gains two life, surveils 1 |
| Suspicious Shambler | 4/2; {4}{B}{B} creates two 2/2 black Zombies |
| Theoretical Necromancer | 4/1; {3}{B} returns another creature card from own graveyard to hand |
| Venom, Evil Unleashed | Legendary 4/5 deathtouch; {2}{B} grants two +1/+1 counters and deathtouch until end of turn |

Every listed graveyard ability exiles its own source as a cost. Bonebind Orator,
Eternal Student, Faerie Dreamthief, Gravestone Strider, Leering Onlooker and
Theoretical Necromancer have ordinary instant-speed activation permission; the
other ten explicitly require sorcery timing. No other parts of the bodies are
omitted. The two additional recovery cards use the existing exact `other` target
predicate, applied to an own-graveyard creature filter.

## Cost transaction and rules

Immutable activation metadata adds a graveyard origin and self-exile cost flag.
All direct and staged actions share the existing authoritative ownership, source
zone, timing, targets, payment and funding checks. The source must remain the same
graveyard incarnation through owned target selection, hybrid-equivalent selection
and mana production. Counterfactual funding retains that source obligation even
if a mana action moves it away. Hand, battlefield, exile, library, opposing
ownership and stale source incarnations cannot satisfy the cost.

Targets are chosen first; mana is paid and the source is then exiled at cost
commit. This creates no death, cast or entry event. The activated ability retains
its exact stack ID and captured old source reference after the physical card
changes incarnation. Its effects execute only on resolution. A later return of
the physical card does not overwrite an existing ability. Ward can counter that
independent ability without refunding the paid exile cost. Source exile does not
exile a different targeted card until resolution.

Gravestone may legally target itself in its graveyard: paying the cost exiles it,
so its target is illegal on resolution. The two `another` recovery abilities
instead exclude their own source before payment and require another owned
creature card. All targets retain generation checks on resolution. Hybrid
activation choices remain controller-owned and locked before payment. Existing
battlefield conversion, hand/cycling, ordinary target selection and private
surveil continuations use the same machinery.

The compiler accepts only the exact self-exile phrase and, for the Renew label,
requires the supported source cost and explicit sorcery restriction. Composite
exile costs involving tap, sacrifice, external discard/sacrifice, life or counter
removal remain gated pending full ordering support. Graveyard mana abilities are
also gated because funding search currently executes mana abilities from the
battlefield. Keyword counters, speed, discover, empower, copies, conditional
permissions and unknown residual text keep affected whole bodies unsupported.
The initial fourteen-body probe gained exactly Bonebind Orator and Theoretical
Necromancer through the shared `another` filter; no incidental bodies were added.

Rules references: CR 113.6m (zone-specific ability), 113.7a (ability independent
of source), 400.7 (new incarnation), 601.2b–i/602.2 (announcement, targets and costs),
602.5d (sorcery timing), 608.2b (illegal targets), in the
[official rules](https://magic.wizards.com/en/rules).
[Official Tarkir release notes](https://magic.wizards.com/en/news/feature/tarkir-dragonstorm-release-notes)
describe Renew as graveyard activation with an exile cost and confirm that normal
priority and timing apply to a newly arrived graveyard card.

## State, replay and validation

No mutable state, observation fields or owned continuation variants were added.
Schema 49, C ABI 1 and control protocol 2 remain unchanged, as does **19,784 bytes**
of initial logical state. The immutable catalogue fingerprint is
`d626deeb0f374a35b92ae26a41606578ea72b18e5bd69db5b095b87810cafd93`.
Snapshots retain complete costs, targets, exact identities, private library
choices, cast/death histories and RNG. Restore invalidates stale indexed actions.
Capacity errors restore the complete pre-action state and revision, including
mana and exile source movement. A later resolution capacity error preserves the
already paid cost and restores the pre-resolution continuation.

- 918 Rust tests: 906 unit and twelve integration tests. Fifteen new methods cover
  all sixteen graveyard bodies for both seats; zones/ownership; phase, turn and
  empty-stack timing; no-mana funding; direct/staged/hybrid payments; exact source
  and target generations; private surveil; full token recipes; actual creature
  casting, entry/death triggers and prowess; temporary keyword expiry; Ward;
  independent same-slot abilities; and capacity rollback/replay.
- 174 compiler/data tests: exact costs/origins/labels, composite and mana gates,
  `another` predicates, all pinned bodies, and whole-body residual negatives.
- 125 fresh release Python tests, including sixteen new real-library staged
  payment scenarios. Actual self-mill/cleanup puts sources in the graveyard;
  tests check old-reference capture, exact mana consumption and exile payment,
  full resolution, private choices, clone/restore and stale indexed actions.
- Catalogue consistency, formatting, Clippy with warnings denied and diff checks.

`cargo run --release --example standard_graveyard_activations -- 100` uses two
complete sixty-card Standard deck families with all sixteen bodies and real
Deathcap Marionette self-mill. The smoke policy delays attacks until turn twelve
and prioritizes legal graveyard announcements to exercise the owned cost path.
This is a coverage workload, not a trained player or metagame benchmark.

Two runs reproduce exactly: 51,529 decisions; cost commits and resolutions both
`[26,43,20,55,41,51,45,31,45,10,43,19,28,9,32,24]` in manifest order; 1,044 exact
cost/source checks; 285 private library choices; 9,496 clone/restore replay
boundaries; and 22,024 maximum initialized logical state bytes. The workload
requires every new ability to commit and resolve. Logical byte counts exclude
allocator/capacity overhead, immutable definitions, observations and externally
retained snapshots.

Three warmed 1,000-game native Freeform benchmark samples (571,049 decisions per
sample) have medians of 399,036 decisions/s, 699 games/s, 130.2 ns clone and
123.7 ns restore at the initial 19,784-byte state. The unpinned host sample range
is 397,201–400,878 decisions/s. These include smoke policy and observation
construction and exclude model/tensor/JSON work. They measure the working source
before commit and are not an XMage parity comparison or speedup claim.
