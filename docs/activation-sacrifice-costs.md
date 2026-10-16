# External sacrifice activation costs

Issue: https://github.com/altf4/rustmage/issues/144
Pinned Standard: October 15, 2026. Nineteen whole bodies bring executable coverage
to **1,181/5,164**; **3,983 remain unsupported**. Six regression identities,
**53 token factories** and one emblem are separate. Two new factories are the
blue 1/1 Fish without abilities and the colorless 3/3 artifact Golem.

## Whole bodies and provenance

`activation-sacrifice-admissions.json` records exact pinned Oracle text and
43 SHA-256 entries across 25 unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All nineteen card classes, the shared
sacrifice cost, four token classes and Mountaincycling were read.

| Card | Complete body |
| --- | --- |
| Acolyte of Aclazotz | Tap/sacrifice another creature or artifact, opponent loses one/controller gains one |
| Ahriman | Flying/deathtouch; three mana/sacrifice another creature or artifact, draw one |
| Cauldron of Essence | Each own creature death drains one; sorcery-speed tap/mana/sacrifice creature, targeted recovery |
| Fountainport | Colorless mana; tap/mana/sacrifice token to draw; paid-life Fish and separate Treasure creation |
| Hardened Tactician | One mana/sacrifice any token, draw one |
| Hungry Ghoul | One mana/sacrifice another creature, source +1/+1 counter |
| Kingpin's Enforcers | Lifelink; mana/sacrifice artifact or creature, draw one; may sacrifice itself |
| Legion Extruder | Targeted two-damage ETB; tap/mana/sacrifice another artifact, create artifact Golem |
| Marwyn, the Clearcutter | Legendary; tap/mana/sacrifice artifact or land, draw one |
| Ripchain Razorkin | Reach; mana/sacrifice land, draw one |
| Seismic Monstrosaur | Trample; mana/sacrifice land to draw; independent Mountaincycling |
| Slagdrill Scrapper | Artifact creature; tap/mana/sacrifice another artifact or land, draw one |
| Sweettooth Witch | Food ETB; mana/sacrifice Food, targeted player loses two |
| Umbral Collar Zealot | Sacrifice another creature or artifact, private surveil one |
| Unburied Earthcarver | Mana/sacrifice another creature, source +1/+1 counter |
| Unrooted Ancestor | Flash; mana/sacrifice another creature, source indestructible until end of turn then tap it |
| Vampiric Rites | Mana/sacrifice creature, gain one and draw one |
| Vanguard of the Rose | Mana/sacrifice another creature or artifact, source indestructible until end of turn then tap it |
| Vito's Inquisitor | Mana/sacrifice another creature or artifact, source counter and menace until end of turn |

The artifact-or-land union adds two bodies to the initial seventeen-card intake.
Wider dependencies remain gated: mixed creature-or-token predicates, suspected/
sacrificed-object event bindings, Rooms, granted/triggered mana, variable costs,
exile permissions, copies, control changes, layers and token replacement effects.
Unknown residual clauses reject the complete body. External-sacrifice mana
abilities and combined external sacrifice plus source sacrifice/life/counter/
loyalty costs remain unsupported. Existing discard plus source-sacrifice support
is retained.

## Payment, reservation and state

An external single-object sacrifice uses the existing owned additional-cost
continuation: targets first, exact resource selection next, mana/source costs and
sacrifice on payment, then the completed activation on the stack. Direct activation
forms cannot bypass this choice. The resource stays on the battlefield while costs
are being selected/funded and leaves only when payment commits.

Candidate checks enforce current object generation, controller, battlefield zone,
kind union, token flag, Food/subtype predicate and the printed “another” restriction.
A source may be selected when its filter allows it; Kingpin's Enforcers is the
admitted example. Other-object costs reject that source. Out-of-range/stale/foreign
objects and wrong zones/kinds/subtypes/token status are rejected atomically.

The existing virtual funding search reserves the chosen object. A reserved land
may tap to produce mana and then be sacrificed. A reserved Treasure cannot be
consumed for mana and then sacrificed again. Other Treasures remain usable. The
enclosing source's own tap obligation is protected separately, including
Fountainport's independent colorless ability: it cannot tap for mana when that
would leave its draw ability's tap cost unpaid.

Source tap/mana costs are committed before external sacrifice, allowing a selected
source to leave after those costs. Existing external discard still precedes
source sacrifice, retaining its battlefield observers. Unsupported composite
payment orders remain gated. No priority is granted between costs. If a sacrifice
synchronously returns an Aura and suspends for entry attachment, the owned
announcement marks the cost paid and clears its payment so resumption cannot
repeat it. Numeric capacity failure restores all mana, taps, objects, continuations
and revision.

Sacrificed creatures generate ordinary death events. Cauldron targets a card
already in the graveyard before choosing its sacrifice; the newly sacrificed
creature cannot retroactively become that target. Its payment death queues the
drain trigger above the completed recovery activation. Countering an activation
does not refund its costs. Source counter/keyword effects resolve later; temporary
keywords keep their existing end-of-turn lifetime. Surveil and cycling retain
private owned library continuations.

No mutable state or public fields were added. Observation schema stays **49**,
C ABI **1**, control protocol **2**, and initial logical state **19,784 bytes**.
AdditionalCostView already represents both sacrifice and discard. Catalogue IDs
are local to fingerprint
`f461f669d1c4448f7ce038416c33546baf942ac7c0399bf9e8ee170dbb5a718e`.
Snapshots own every target, selected object, pending payment and private choice;
restoration advances the revision and rejects stale indexed actions.

Rules reference: CR 118, 400.7, 601.2, 602.2, 603, 608 and the sacrifice rules in
the September 25, 2026 text linked by the
[official rules page](https://magic.wizards.com/en/rules).

## Verification and workload

- **894 Rust tests**: 882 unit and twelve integration. Eleven new methods cover
  all nineteen activated bodies in both player seats; exact filters and identity;
  selected source sacrifice; land/Treasure reservations; Fountainport's competing
  tap obligation; all four Fountainport abilities; Witch/Extruder entry and token
  bodies; Cauldron target/payment/death ordering; private surveil and Mountaincycling;
  linked Aura return suspension; overflow rollback and structural countering.
- **170 compiler/data tests** include exact scalar costs and typed filters,
  artifact/land unions, whole-body admissions and residual rejection, source cost
  distinction, wider composite and external-sacrifice mana gates.
- **122 fresh release Python tests**, including nineteen staged native C ABI
  scenarios with owned selected resources, exact mana payment, complete activation
  resolution, source sacrifice/growth, clone/restore and stale rejection.
- Format, all-target Clippy with warnings denied, catalogue consistency and diff
  checks pass.

`cargo run --release --example standard_activation_sacrifice -- 100` uses three
legal 60-card Standard deck families covering tokens/artifacts, graveyard/land
resources and white/black sacrifice outlets. It requires all nineteen activation
commits and resolved bodies, source sacrifices and private choices. The policy
holds Seismic on even seeds, permits cycling on odd seeds, delays attacking until
turn twelve and prioritizes a Witch activation until its first committed Food
payment. This keeps competing outlets from consuming every Food before that body
is exercised. These are workload preferences; the engine action space is unchanged.

Two runs reproduced **156,060 decisions, 4,103 owned selections/committed
sacrifices, 53 source sacrifices, 21,767 cost/privacy checks, 428 private surveil
boundaries, 69,608 clone/restore replays and 21,544 maximum logical initialized
bytes**. Commit counts in table order were:
`[106,139,66,2,32,180,55,648,104,63,2,69,1,428,52,184,1593,50,329]`.
Completed body counts were:
`[106,139,63,2,32,180,55,648,104,63,2,69,1,428,52,184,1584,50,329]`.
Some games end with further activations still on the stack. The checker verifies
selected-object departure, exact mana subtraction, source taps, privacy and replay;
focused native/Python scenarios verify the individual effects and reservations.
A structural stack-removal fixture models a countered activation without claiming
an ability-counter spell has been admitted. A synthetic exile link isolates
existing synchronous Aura entry after cost payment.

An unpinned 1,000-game initial Freeform smoke run recorded 571,049 decisions,
400,283 decisions/s, 700.96 games/s, 131.68 ns clone and 127.65 ns restore. It
includes native decisions/observations and excludes inference, tensors, JSON and
Python. This is a bounded smoke measurement, not Java parity or full Standard
throughput. Logical bytes exclude capacity, allocator overhead, immutable
metadata, observations and external snapshots.

Full Standard and broader native RL/search acceptance remain incomplete.
