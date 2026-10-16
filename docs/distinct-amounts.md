# Distinct characteristics and scaled draw, growth and drain

[Issue #102](https://github.com/altf4/rustmage/issues/102) adds immutable distinct
color/card-type expressions and shared scaled effects. Coverage is 1,014/5,164
pinned Standard identities, with 4,150 unsupported, six regression definitions,
48 token factories and one emblem factory. The [admission record](distinct-amount-admissions.json)
contains all eight complete Oracle bodies and source hashes verified against
XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

| Complete admission | Behavior |
| --- | --- |
| Explosive Prodigy | Entry damage to an opposing creature uses the live controlled-permanent color union |
| Frontline Rush | Modal two real Goblins or a temporary targeted bonus using the live controlled creature count |
| Glister Bairn | Hybrid cost; own beginning of combat gives another controlled target a temporary color-count bonus |
| Luminollusk | Deathtouch; entry gains the color count as one life-gain event |
| Peer Past the Veil | Privately select the complete hand for simultaneous discard, then draw the distinct card types in the controller's graveyard |
| Shimmercreep | Menace; entry freezes the color count, opponents lose that life and controller gains it once |
| Shinestriker | Flying; entry draws the live color count as individual successful draws |
| Squawkroaster | Double strike; characteristic power continuously equals the color count in every zone |

Color quantities union all relevant color bits, so a hybrid/multicolored permanent
can supply several colors and duplicates count once. Colorless is not an extra
color; producing colored mana does not itself color a land or Treasure. Printed
card colors, token colors and both hybrid halves use the existing intrinsic
characteristic tables. Color-changing layers remain gated with their incomplete
card bodies. These expressions use earlier-layer facts and do not recurse through
derived power/toughness. Card-type quantities use a separate union of type bits,
retain Kindred and compound types, honor owner/zone filters, and exclude tokens
when the expression specifies cards. Supertypes and subtypes are not card types.
The existing all-graveyard characteristic expression remains supported.

Resolution amounts evaluate when their instruction executes (CR 608.2h).
Glister/Frontline store the resulting additive bonus rather than a live count,
so later color changes do not resize that temporary bonus; ordinary cleanup
removes it. Squawkroaster instead uses the live characteristic evaluator in every
zone (CR 604.3 and 613). Both use the usual counter/continuous-stat pipeline.
Normal source incarnation, target generation, ownership, hexproof, ward and
mode legality remain enforced by the shared engine.

Scaled draw calls the ordinary individual-draw pipeline (CR 121.2–4). Successful
draw ordinals/observers are exact; failed draws do not trigger successful-draw
abilities and retain the normal state-based loss. Work is bounded by available
cards plus one failed attempt because no admitted replacement or failed-draw
observer distinguishes further failed attempts. Replacement/deck-out modifiers
remain gated. Peer completes the existing owned private discard continuation
before evaluating graveyard types, so the discarded cards count and the resolving
spell itself does not. Discard and draw triggers wait until resolution completes.
A snapshot midway through private selection owns the selected cards, program
counter and remaining work; replay does not repeat a completed discard batch.

Life drain uses one frozen amount for the opponent's loss and the controller's
single gain. It deals no damage, has no lifelink and creates no damage history.
A zero amount neither gains life nor creates a life-gain event. Checked signed
conversions/multiplication/stat additions, life totals and draw counters remain
inside the existing action checkpoint. Overflow after life loss or after the last
private discard restores both players, cards, pending choices and revision.
No mutable expression cache or observation field is introduced. Observation 41,
ABI 1 and control protocol 2 are unchanged; initial logical benchmark state remains
19,648 bytes, excluding capacity, allocator/COW metadata, statics, observations
and external snapshots.

Rules were checked against the [official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt),
including CR 105 (colors), 205.2 (card types), 608.2 (resolution), 121 (draws) and
514 (cleanup), linked from [Wizards' rules index](https://magic.wizards.com/en/rules).
Vivid is stripped only as a nonrules ability word; every remaining clause must
compile. Unsupported variable target counts, conditional costs, color-changing
choices, reveal-until selection, mana-spent counts, X mana costs and other body
dependencies still gate complete cards, including the remaining Vivid cards.

Author validation passes 679 Rust tests (667 unit and twelve integration), 121
compiler/data tests, 83 release Python tests, catalogue consistency, rustfmt and
Clippy with warnings denied. Seventeen new native regressions cover five-color/
hybrid/multicolor/token unions, colorless mana sources, ownership/zones, compound/
Kindred card types, every body/mode, live versus frozen stats, cleanup, source
removal and zero quantities, individual draw events/decking, privacy, complete
clone/reusable restore, and numeric rollback at private/final-resolution boundaries.
Two new binding tests contain three public fixtures for Shinestriker, Shimmercreep
and Peer's suspended private batch; all check native clone/restore and independent
public quantities.

One hundred legal 60-card Standard games complete naturally: 58,753 decisions,
1,388 independent public facts across 628 quantity effects and 14,728 clone/replay
boundaries, with 21,820 maximum initialized state bytes. All eight admissions'
quantity behaviors are sampled. The example uses public color facts, printed
type-line sets, life/draw histories, damage and stat differences independently of
the internal expression evaluator. It delays attacks until turn 18, declines
blocks and alternates Frontline mode preference by seed to exercise expensive
entries and both modes. These are smoke-policy choices; engine legality and full
action spaces are unchanged. Focused tests cover additional behaviors/privacy.

An unpinned 1,000-game benchmark measured 571,049 decisions at roughly 404,094
native decisions/s (707.6 games/s), clone 97.1 ns and restore 92.5 ns. These noisy
host-dependent smoke timings include policy/observation work but exclude model
inference, tensor encoding and JSON; they do not prove a speed change or XMage
throughput parity. Full Standard and native RL/search acceptance remain incomplete.
