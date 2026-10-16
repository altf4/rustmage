# Source tap events

Issue: https://github.com/altf4/rustmage/issues/191

Coverage is **1,371/5,164** pinned Standard cards, with **3,793 unsupported**.
Six non-Standard regression identities, 57 token factories and one emblem remain
separate. The full Standard migration remains incomplete, pinned to October 15, 2026.

This stage implements the complete bodies of thirteen creatures: Attentive
Sunscribe; Mechan Navigator; Pestered Wellguard; Rescue Leopard; Seedship Agrarian;
Silvergill Peddler; Spider-Gwen, Free Spirit; Starfighter Pilot; Swarm Culler;
Volatile Wanderglyph; Wanderbrine Preacher; Wanderwine Distracter; and Wylie Duke,
Atiin Hero. Their effects cover private scry/surveil, mandatory draw-then-discard,
optional discard-then-draw, optional sacrifice-then-draw, Faerie and Lander tokens,
life gain, and temporary power reduction. Seedship retains its landfall counter
ability and the complete Lander search activation. Flying, reach, vigilance,
legendary status, colors, types, statistics and costs remain part of each full body.

The native library dispatches `SelfBecomesTapped` only on an actual untapped-to-tapped
battlefield transition. Attack declaration commits the whole attacking set before
dispatch; vigilance prevents the tap and its trigger. Selection alone does not tap.
Tap effects, attachments, source instructions, mana taps and activation costs share
the dispatcher. A simultaneous tap batch changes every member before observing
its sources, and duplicate identities trigger once. Entry tapped status does not
count as becoming tapped. Already tapped objects produce no event, and the pure
mana-funding planner never dispatches events. Torpor Orb suppresses entry events,
not tap events. These distinctions follow CR 603.2e and 701.26 in the
[pinned official rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).

Trigger packets own the source incarnation, source definition and controller. A
later departure or return does not cancel the captured effect or bind it to the
new incarnation. Tap costs capture the source before a subsequent sacrifice;
normal announcement and APNAP ordering run after the enclosing action completes.
Private choices, suspended resolutions, RNG and pending triggers remain snapshot
owned. Capacity errors restore both status changes and the action's revision.
No mutable game-state field or observation schema changes. Schema remains **52**,
C ABI **1**, control protocol **2**, and initial logical state **19,848 bytes**.
This excludes allocator/capacity, immutable definitions, observations and separately
retained snapshots.

Single-object taps avoid allocating a batch vector. Ordinary lands and other
objects without a tap observer also avoid constructing an evaluated object view.
Multi-object taps retain only sources that actually have a tap trigger and dispatch
after all status changes. Single-target taps and untaps avoid a new temporary
vector, while mass-tap effects use one identity vector.

The compiler recognizes the complete singular `Whenever this creature becomes
tapped, ...` header and authenticated self-name aliases, including legendary short
names. It does not approximate untap, enters-tapped, tap-for-mana, other-creature or
combined-event headers. The existing effect compiler must accept every residual
rule and cost. Unknown trailing text keeps the entire card unsupported.
Exactly thirteen immutable definitions change; existing executable definitions,
card identities and token factories remain unchanged.
`source-tap-admissions.json` records the complete Oracle bodies and **57 SHA-256
entries across 33 unique Java files** at pinned revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. The constructors, shared tap observer,
effects, keywords and token implementations were read in full; recorded blobs match
the pinned revision and checkout.

Twenty native tests cover both-seat full casts and every complete body, actual
attack and spell-driven taps, APNAP batches, vigilance, entry and repeat-tap exclusions,
mandatory versus optional payments, both private library destinations, rejected
sacrifice/target scopes, source departure and reentry, Lander search and landfall,
funding purity, queued private frames, stale indexed actions, clone/restore replay,
and atomic overflow. The activation tap-and-sacrifice hook test uses a synthetic
internal ability packet to isolate cost ordering; none of these thirteen cards is
admitted with an invented activated ability. Three compiler tests cover exact
headers, complete bodies, unknown residual gating and effect ordering.

`examples/standard_source_taps.rs` runs legal 60-card Standard decks with four
copies per nonbasic card. Independent public witnesses track real status changes,
source-incarnation trigger debts, each body effect, per-turn counters and temporary
power reductions, statistics and keywords, private choices and clone/restore replay.
The Python test exercises thirteen full scenarios through the C ABI with exact
resolution outcomes, private choice ownership and stale indexed-action rejection.

Two release runs of 100 games reproduced **70,443 decisions** and **57,094 replay
boundaries**. In manifest order, source casts were
`[48,42,47,38,41,53,46,58,46,41,47,49,29]`, tap triggers were
`[110,99,103,88,113,170,108,83,125,60,133,157,43]`, and verified effects were
`[110,99,103,70,113,170,92,83,103,51,133,154,43]`.
There were 1,403 true taps (774 attack taps and 629 effect taps), 585 untapped
entries, 22 Wylie vigilance declarations, 193 private choices, 34 surveilled cards
put in the graveyard, 11 mandatory-target triggers that could not be announced,
and 268,691 public stat/keyword checks. Maximum initialized logical state was
21,888 bytes. Both runs used catalogue
`afb997cc5a85525b5622b5b64281fdd8bbbc7a787a8f742b697ded928b0870e2`.
These are policy smoke workloads, not XMage parity or neural-network inference
benchmarks.

The 1,000-game native smoke benchmark measured 571,049 decisions, approximately
327,144 decisions/s and 573 games/s, 134 ns clone and 131 ns restore on this host.
Timing includes policy and observation construction, excludes model inference,
tensor encoding and JSON, and varies with concurrent host work.

Final validation results are recorded after the acceptance changes are integrated.
