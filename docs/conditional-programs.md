# Guarded effects and intervening-if triggers

[Issue #104](https://github.com/altf4/rustmage/issues/104) adds shared live predicates,
flat conditional programs and general forward-event intervening-if triggers.
Coverage is 1,033/5,164 pinned Standard identities (4,131 unsupported), six
regression definitions, 48 token factories and one emblem factory. The
[admission record](conditional-admissions.json) contains nineteen complete Oracle
bodies and source hashes checked against XMage revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

| Complete admissions | Behavior |
| --- | --- |
| Donatello, Turtle Techie | Artifact condition gates entry triggering and is rechecked before drawing |
| Dwynen's Elite | Another controlled Elf, including changeling/Kindred, gates entry token creation at both boundaries |
| Crypt Feaster | Menace; threshold attack trigger checks seven graveyard cards twice before temporary growth |
| Malamet Veteran; Stinging Cave Crawler | Trample/deathtouch and descend-four attack triggers using permanent cards; target counter or draw/life loss |
| Nasty Little Rabbit; Nessian Hornbeetle | Own beginning-combat counters with ferocious versus another qualifying creature; conditions are rechecked |
| Hero in Training | Unconditional entry draw, then a live other-Hero condition for life gain |
| Failed Fording; Taken by Nightmares | Targeted bounce/exile, then live Desert/enchantment condition for private surveil/scry |
| Rabbit Response | Controlled creatures gain temporary stats, then live Rabbit condition for private scry |
| Violent Urge | Temporary growth/first strike, then live graveyard card-type condition for double strike |
| You're Not Alone | Exactly one temporary bonus chosen from live controlled creature count, not two additive bonuses |
| Take Out the Trash | Damage, then live Raccoon condition for owned optional discard-and-draw |
| Rosa, Resolute White Mage | Reach; own beginning-combat target counter followed by temporary lifelink |
| Snakeskin Veil | Controlled target counter followed by temporary hexproof |
| Moment of Valor | Complete untap/growth/indestructible mode or power-qualified destruction mode |
| Spectacular Tactics; Warg Tactics | Complete controlled counter/temporary keyword modes and power/flying-qualified destruction alternatives |

An intervening-if wrapper retains the event separately from its predicate.
Forward events check the predicate before allocating a pending trigger or consuming
its once-per-turn limit. Resolution checks it again before targets/effects. A
condition false at the event cannot start triggering retroactively; a true event
can produce an ability that later does nothing. Beginning-step guards now share
this path. Wrapper traversal retains event dispatch, observer indexing, captured
contexts and trigger limits. Source departure does not inherently counter an
ability; its live condition and ordinary source/target incarnation rules still
apply. Lookback-event intervening conditions (including death/discard) remain
gated until their prior-event facts are modeled.

Ordinary effect conditions execute only when their instruction is reached. A
flat `If` skips a known block, and `Skip` selects the common continuation after an
alternative. No nested mutable interpreter stack or expression cache is needed.
The compiler keeps a later unconditional sentence outside the preceding guarded
sentence, preserves the exact existing optional-payment/if-you-do construct as
one nested block, and accepts only the supported same-target stat replacement
form. Creature pronouns bind only after a single proven creature target and a
sequence of proven focus-preserving modifiers. Token creation, zone changes and
other unproven intervening operations block later pronouns, including across
alternative branches. This conservative gate may reject unambiguous wording until
its antecedent is modeled. Unknown
antecedents, prior-cast/target ownership, event history, conditional mana semantics
and residual clauses remain rejected. The full pinned creature-type vocabulary
backs singular subtype predicates; no arbitrary subtype name is inferred.

The owned program counter survives private scry/surveil and optional discard
choices. Resuming runs the remaining body once, without repeating earlier growth,
bounce, exile or damage, and does not reevaluate an already-entered condition.
Targets are still chosen legally and revalidated; an entirely illegal target set
prevents the whole spell from resolving, including untargeted conditional tails.
Checked numeric failure restores earlier instructions, library contents, histories,
pending choices, RNG and revision through the existing action checkpoint.

State-based actions wait until the entire resolution completes, including private
choices. A lethally damaged Raccoon therefore still qualifies for Take Out the
Trash's later conditional payment; it dies after the choice and remaining spell
finish. Destroy/exile/return instructions actually move their objects immediately,
so those departures can change a later condition during the same resolution.
These distinctions follow CR 603.4, 608.2 and 704.4 in the
[official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt),
linked from [Wizards' rules index](https://magic.wizards.com/en/rules).

All predicates/programs are immutable definitions. No Game/Player/Object state
field is added. Observation schema 41, ABI 1 and control protocol 2 remain unchanged.
Source identities, private continuations, limit ledgers and counters already belong
to snapshots. Initial logical benchmark payload remains 19,648 bytes, excluding
capacity, allocator/COW metadata, statics, observations and external snapshots.

Author verification passes 704 Rust tests (692 unit and twelve integration), 127
compiler/data tests, 87 release Python tests, catalogue consistency, rustfmt and
Clippy with warnings denied. Twenty-five new native tests cover all nineteen
bodies/modes, true/false event and resolution checks, simultaneous entry, subtype/
Kindred/changeling and owner/zone membership, threshold/descend/type counts,
self-versus-other power, target legality/fizzle, cleanup, guarded alternative
selection, private continuation/replay and numeric rollback after an earlier draw.
A synthetic flat-program fixture verifies if/else/common-tail offsets around scry
without admitting a fictional card. Four binding tests cover Donatello with and
without an artifact, Rabbit's suspended scry, both replacement-growth outcomes,
and Take Out the Trash's paid private discard after damage.

One hundred legal 60-card Standard games complete naturally: 65,378 decisions,
1,959 independent public checks across 1,011 rule resolutions and 16,382 clone/
replay boundaries, with 20,836 maximum initialized state bytes. All nineteen
admissions, all three modal pairs and both conditional spell outcomes are sampled.
The checker retains public pre-resolution observations across private choices and
compares independent counts, type sets, stats, counters, keywords, damage, zones
and draw/life facts. The smoke policy delays attacks until turn 12 and alternates
mode preference by seed; blocks remain available. These policy choices do not
change engine legality or full action spaces. Focused fixtures check disappearing
conditions and private-state visibility beyond this supported-deck workload.

An unpinned 1,000-game benchmark measured 571,049 decisions at about 397,607 native
decisions/s (696.3 games/s), clone 109.6 ns and restore 88.9 ns. Host-dependent smoke
timings include policy/observation work but exclude model inference, tensor encoding
and JSON. They do not establish a speed change or XMage parity. Full Standard and
broader native RL/search acceptance remain incomplete.
