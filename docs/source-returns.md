# Exact source returns

Issue: https://github.com/altf4/rustmage/issues/150
Pinned Standard: October 15, 2026. Eleven complete bodies bring executable coverage
to **1,212/5,164**; **3,952 remain unsupported**. Six regression identities,
55 token factories and one emblem remain separate.

## Complete bodies and provenance

`source-return-admissions.json` records pinned Oracle text and 23 SHA-256 entries
across fifteen unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. The eleven card implementations, three
shared source-return effects and explicit sorcery-timing class were audited.

| Card | Complete body |
| --- | --- |
| Abzan Devotee | 2/1; once-per-turn battlefield {1} conversion into white, black or green; graveyard {2}{B} return to hand |
| Arcanis the Omnipotent | Legendary 3/4; tap to draw three; battlefield {2}{U}{U} return to owner's hand |
| Bestial Bloodline | Enchant creature, +2/+2; graveyard {4}{G} return to hand |
| Blessed Ghoul | Hybrid {W/B} 1/1 lifelink; hybrid graveyard {2}{W/B} return to hand |
| Merchant of Many Hats | 2/2; graveyard {2}{B} return to hand |
| Project Deathlok Soldier | 1/2 artifact creature; graveyard {2}{B} return to hand |
| Reassembling Skeleton | 1/1; graveyard {1}{B} return to battlefield tapped |
| Summoned Dromedary | 4/3 vigilance; sorcery-only graveyard {1}{W} return to hand |
| Teacher's Pest | 1/1 menace; attack gains one life; graveyard {B}{G} return to battlefield tapped |
| Tunnel Rats | 2/2; graveyard {4}{B} return to battlefield tapped |
| Wingspan Stride | Enchant creature, +1/+1 and flying; battlefield {2}{U} return to owner's hand |

Other than Dromedary's explicit sorcery restriction, these return abilities have
ordinary instant-speed activation permission. Return-to-hand and return-to-
battlefield are resolution effects, never movement costs. No residual body is
omitted, and no incidental identities were admitted.

## Rules and compiler boundary

Two immutable instructions return the captured source from a specified zone to
hand, or from the graveyard to the battlefield with an explicit tapped-entry
requirement. Both require the exact captured object generation and current origin
zone. A returned/recast/reburied later incarnation cannot satisfy an older
ability. Multiple activations can wait independently; once one moves the source,
other abilities referring to the old incarnation finish without moving it again.
Paid mana is not refunded. Arcanis can return while its draw ability waits, and
that independent lower ability still draws three.

The exact source-return parser runs only inside supported activated abilities.
It authenticates the full source name or established legendary short alias and
infers graveyard origin only from an exact graveyard-return instruction. It does
not extend generic spell or trigger programs. Death, delayed and next-zone returns
need different incarnation bindings and remain gated. Unknown trailing clauses,
finality counters, external exile costs, wider attachments/type changes, X costs,
unsupported cost combinations and graveyard Exhaust combinations remain gated.
Impossible graveyard tap/sacrifice/counter/loyalty source costs are rejected.

Direct and staged activation paths share owner, zone, timing, payment and funding
authority. Hybrid equivalents remain controller-owned and locked before payment.
These abilities have no unpaid source movement cost, so an already announced
ability may legally finish paying mana after its source leaves the original zone;
its resolution then respects the old binding. This intentionally differs from a
self-exile cost, which still needs that exact source until cost commit.

Battlefield returns use the existing owned entry transaction. Forced tapped state
is combined with intrinsic entry replacements before movement, source registration
and entry events. Initial counters, attachment/life choices and suspended program
resumption therefore retain their existing machinery. Returned creatures are
summoning sick and do not count as casts or deaths. A normal cast of Skeleton,
Pest or Rats enters untapped; only their graveyard ability forces tapped entry.
Pest's actual attack observer is registered after return.

The actual new hand incarnation of a publicly returned source is added to the
existing public `known_hand` ledger, without duplicates. Opponents can observe
that card while unrelated private hand cards remain hidden. Leaving the hand
removes the fact. Clone/restore preserves this knowledge. Unifying the older
face-up targeted/batch return paths is tracked separately in
[issue #151](https://github.com/altf4/rustmage/issues/151).

Rules references: CR 113.6m (zone-specific source return), 113.7a (independent
abilities), 400.7 (new incarnation), 602 (activation and sorcery restrictions),
608 (resolution), and 614.12 (entry replacements), in the
[official rules](https://magic.wizards.com/en/rules).

## State, validation and performance

No mutable fields, observation shape or continuation variants were added. Schema
49, C ABI 1 and control protocol 2 stay unchanged, with **19,784 bytes** of initial
logical state. Active states can hold additional public hand references in the
existing ledger. Catalogue fingerprint:
`95664285e02e408361e0e9e37153edf94d5ba6122039ac31e54b3f7243556b50`.

- 931 Rust tests: 919 unit and twelve integration tests. Thirteen new methods
  cover all nine graveyard returns for both seats, wrong zones/owners, independent
  phase/turn/empty-stack restrictions, direct/staged/hybrid/land-funded costs,
  source departure during payment, repeated and stale incarnations, capacity
  rollback, actual Auras and recasting, public hand knowledge, forced versus normal
  entry, registered entry/attack observers, Arcanis draw/return and summoning
  sickness, and Abzan's original mana conversion.
- 176 compiler/data tests: exact activated source bindings, full residual
  rejection, authenticated aliases, impossible source costs, original dependency
  gates and no accidental death-trigger/spell compilation.
- 127 fresh release Python tests, including eleven new native lifecycle scenarios
  through owned payments: nine graveyard returns plus Arcanis and Wingspan on the
  battlefield. Check cost-before-effect, exact old/new references, public knowledge,
  tapped entry, complete resolution, clone/restore and stale actions.
- Catalogue consistency, formatting, Clippy with warnings denied and diff checks.

`cargo run --release --example standard_source_returns -- 100` uses three complete
sixty-card Standard deck families with actual self-mill. The coverage policy delays
attacks until turn twelve and prioritizes owned return announcements. On odd seeds
it retains Arcanis until its draw activation is paid, allowing return in response.
This is not a trained policy or metagame benchmark.

Two runs reproduce exactly: 57,128 decisions; paid commits and resolutions both
`[117,9,20,180,55,101,147,19,14,35,175]`; actual movements
`[82,6,20,133,39,69,75,19,12,34,129]` in manifest order; nine Arcanis draw-ability
resolutions; 1,490 exact cost/source/public-knowledge checks; 11,730 clone/restore
boundaries; and 21,332 maximum initialized logical state bytes. Extra resolved
abilities can correctly do nothing after another activation already moved the
source. All eleven return abilities must pay, resolve and actually move a source.
Logical byte counts exclude allocator/capacity overhead, immutable definitions,
observations and externally retained snapshots.

The existing 1,000-game native Freeform benchmark completes 571,049 decisions at
401,956 decisions/s and 704 games/s, with 130.5 ns initial-state clone and 116.8 ns
restore. This is one host sample including native smoke policy and observation
construction; it excludes model/tensor/JSON work and is not an XMage parity or
speedup claim.
