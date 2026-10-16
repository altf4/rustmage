# Exact targeted bounce, tap and stun programs

Issue: https://github.com/altf4/rustmage/issues/172

Eighteen complete bodies bring executable coverage to **1,303/5,164** pinned
Standard cards, with **3,861 unsupported**. Six regression identities, 56 token
factories and one emblem remain separate. No existing executable body is lost.
The format remains pinned to October 15, 2026; full Standard migration is ongoing.

| Card | Complete body |
| --- | --- |
| Bigfin Bouncer | 3/2; entry returns target opposing creature to its owner's hand |
| Bounce Off | Instant; returns target creature or Vehicle to its owner's hand |
| Constrictor Sage | 4/4; entry taps an opposing creature and adds one stun counter; Renew `{2}{U}`, exile itself from graveyard, repeats that effect only as a sorcery |
| Exclusion Mage | 2/2; entry returns target opposing creature to its owner's hand |
| Exosuit Savior | 2/2 flying; entry returns up to one other owned permanent to its owner's hand |
| Freeze in Place | Sorcery; taps an opposing creature, adds three stun counters, scries two |
| Grappling Kraken | 5/6; owned landfall taps an opposing creature and adds one stun counter |
| Iceridge Serpent | 3/3; entry returns target opposing creature to its owner's hand |
| Mental Modulation | Instant; costs `{1}` less during its controller's turn; taps target artifact or creature, then draws a card |
| Mischievous Pup | 3/1 flash; entry returns up to one other owned permanent to its owner's hand |
| Peerless Ropemaster | 4/4; entry returns up to one target tapped creature to its owner's hand |
| Proctor's Gaze | Instant; returns up to one target nonland permanent to its owner's hand, then searches for a basic land onto the battlefield tapped and shuffles |
| Rimekin Recluse | 3/2; entry returns up to one other target creature to its owner's hand |
| Ring of the Lucii | Legendary artifact; taps for `{C}{C}`; `{2}`, tap, pay 1 life taps target nonland permanent |
| Rowdy Snowballers | 2/2; entry taps an opposing creature and adds one stun counter |
| Spider-Byte, Web Warden | Legendary 2/2; entry returns up to one target nonland permanent to its owner's hand |
| Stickytongue Sentinel | 3/3 reach; entry returns up to one other owned permanent to its owner's hand |
| Tidecaller Mentor | 3/3 menace; intervening Threshold entry returns up to one target nonland permanent to its owner's hand |

`targeted-bounce-tap-admissions.json` records all eighteen complete Oracle texts
and **122 SHA-256 entries across 55 unique Java files** at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All constructors were read in full;
target classes/filters, return/tap/stun effects, entry/landfall/Threshold,
Renew costs, mana and life costs, source reduction, scry/search and intrinsic
abilities are recorded. Every recorded blob matches the pinned Git revision
and read-only checkout. No new token factory is required.

## Compiler scope and existing rules

The compiler maps exact object subjects to existing immutable target rules,
with optional cardinality kept explicit. Subjects retain controller, source
exclusion, tapped status, nonland status and creature-or-Vehicle union semantics.
Only exact return-to-owner-hand and tap/untap sentences compile. The targeted
opponent tap-plus-stun clause retains one target and supports bounded printed
counts. Existing composition admits only wholly recognized tails.

Required opponent targets remain required. An entry trigger with no legal
required target is removed under the existing trigger-placement rules; the
creature itself remains legal to cast. Optional source exclusion differs from
optional selection: Exosuit/Pup/Sentinel may choose no targets or another owned
land, but cannot choose themselves or a foreign permanent. Rimekin excludes its
source without requiring an owned recipient. Spider-Byte may target itself.
Peerless requires a tapped creature both at announcement and at resolution.
Bounce Off accepts an unanimated Vehicle as well as a creature; arbitrary
artifacts and lands do not qualify. A native structural fixture supplies an
unsupported Vehicle identity for union testing; unsupported decks still reject
that body's execution.

Proctor's Gaze with zero chosen targets still searches. If it chooses one target
and that target becomes illegal, the whole spell fails to resolve, including the
search. Mental Modulation likewise cannot draw after its sole target becomes
illegal. All revalidation uses exact incarnations; returning and reentering the
same physical card never restores an old target. Bounce records the new public
hand identity, detaches Equipment, sends an unattached Aura to the graveyard and
ceases returned tokens without leaking them into hand observations.

Tapping an already tapped target still permits stun counters and later
instructions. Each future untap attempt consumes one stun counter and leaves
the permanent tapped; the following attempt can untap it. Ward remains a stack
payment and can counter the whole targeted spell, including scry. Hexproof
prevents opposing target selection. Tidecaller's intervening Threshold is
checked at entry and again before resolution. Grappling's landfall only observes
its controller's lands.

Renew pays mana and exiles its exact graveyard source at commitment, remains
independent afterward and only activates with sorcery timing. Ring's pure mana
ability resolves immediately; its targeted tap activation pays mana/life/tap
costs and uses the stack. Funding counts only the real mana program, cannot
reuse a tapped Ring or derive mana from its targeted program, and leaves the
original state/revision unchanged.

No production Rust rule change was needed. Observation schema **51**, C ABI
**1**, control protocol **2**, owned state and snapshots remain unchanged.
Initial logical state is **19,784 bytes**; accounting excludes allocator and
capacity overhead, COW sharing, immutable definitions, observations and external
snapshots. Unknown conditional return effects, reflexive triggers, broader
permissions and unsupported residual mechanics remain gated.

## Validation

Sixteen native tests cover both seats, all eighteen actual casts and complete
programs, target absence/optionality/controller/source/zone/type distinctions,
live tapped-state and generation revalidation, private scry/search, Ward,
hexproof, cost reduction, landfall, intervening Threshold, Renew cost timing,
Ring's separate programs/funding, attachment cleanup and token disappearance.
They replay semantic transitions, reject illegal/stale actions and verify atomic
rollback when a real stun-counter overflow would otherwise leave a target tapped.
Three compiler tests cover exact subjects, cardinalities, composition, bounds,
full admissions and unknown residual rejection. The existing required-creature
cardinality representation is retained for group composition. One fresh release
Python method resolves eighteen actual scenarios, independently checks each
target effect and replays source/target/payment/private boundaries.

Two native Standard100 runs with three legal 60-card/four-copy deck families
reproduce:

- **92,952 decisions**, casts `[43,93,114,104,125,175,197,36,106,471,57,165,70,67,131,59,71,39]` in table order.
- Triggers/finished triggers `[36,0,106,77,125,0,153,31,0,471,57,0,70,0,119,59,71,6]`.
- Seven Renew activations and fourteen Ring targeted activations.
- **1,893 independently verified resolved target effects**, **180,630 public
  stat/ability pairs**, **90,172 creature-history checks**, **809 private
  choices** and **29,154 native replay boundaries**.
- **20,524 maximum initialized logical state bytes** in this workload.

Complete checks passed: **1,074 Rust tests** (1,062 unit, 12 integration),
**204 compiler/data tests**, **141 fresh release Python tests**, all-target
Clippy with warnings denied, formatting, catalogue consistency and diff checks.

The separate native 1,000-game benchmark completed 571,049 decisions at roughly
379,642 decisions/s and 665 games/s, clone 129 ns and restore 125 ns in this run.
Timing depends on the host; it includes policy and observation construction,
excludes model inference/tensors/JSON and is not an XMage parity comparison.
