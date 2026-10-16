# Planeswalker combat declarations and damage routing

This is the combat portion of [issue #93](https://github.com/altf4/rustmage/issues/93).
It builds on loyalty damage from PR #95. Coverage remains 986/5,164 Standard
identities (4,178 unsupported), six regression definitions and 43 token factories.
No walker body is admitted: legal loyalty activations and complete printed bodies
remain outstanding. The explicit future-walker fixtures exercise this kernel only.

The pinned [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
are the rule reference: 508.1b for defender choice, 506.4/506.4c for removal from
combat, 510.1b–c for unblocked/blocked damage, and 702.19b/d/f for ordinary trample.
Attacking a walker is not spell/ability targeting and ignores hexproof; no target
announcement or ward trigger is created. A creature still obeys controller, zone,
tap, summoning-sickness, haste and defender restrictions. Vigilance preserves its
untapped state. No new targeting shortcut bypasses the shared legal-action visitor.

Native `Action::Attack { card }` declares an attack on the opponent.
`Action::AttackPlaneswalker { card, defender: ObjectRef }` selects an exact opposing
battlefield walker incarnation. Forged/non-walker/own/stale references are rejected
without changing state or revision. Each creature can be selected once. Independent
attackers may choose different walkers or the opponent in the same declaration.

`CombatDefender` is `Player(Player)` or `Planeswalker(ObjectRef)`.
Owned `AttackDeclaration { card, defender: Option<CombatDefender>, blocked }`
records both pending and committed selections. `None` means the attacked permanent
left combat; it does not mean the defending player. Selected declarations remain
private to the choosing player until `FinishAttackers`. Committed declarations are
public. Observation schema **39** adds `attack_declarations`; `attackers` still lists
creature IDs. The optional control response remains schema 2 and ABI remains 1.
For example, a selected walker's JSON defender is
`{"Planeswalker":{"card":42,"generation":1}}`; a departed defender is `null`.

Zone transitions clear exact references to a departed defending walker, retaining
the creature's attacking/blocked status and combat relationships. Returning the
same card creates a new incarnation and never reattaches old attacks. An unblocked
attacker then assigns no damage. Blocked attackers still exchange damage with
surviving blockers. Ordinary trample can spill past lethal blocker assignments to
the attacked walker, but never to its controller. Without a remaining defender,
all available damage stays with surviving blockers. Walker loyalty dying in the
first-strike step leaves no defender for the normal step, including double strike.
Damage-to-loyalty, event amounts, lifelink, deferred SBAs and batch histories compose
with the existing source-aware damage kernel. Trample over planeswalkers, battles,
control/type changes, phasing and card-specific attack restrictions remain gated
with their unsupported bodies and require their own implementations.

Every defender is owned snapshot state. Damage continuations carry the same exact
defender, including arbitrary legal interior i128 damage assignments. Logical
state accounting includes the larger selected/committed declaration elements and
damage continuations. The candidate walker set is collected once per attack-action
visit, avoiding an arena scan for each creature; no vector allocation occurs when
the defending set is empty. The old initial benchmark state stays 19,552 logical
bytes. Public rollouts below use 21,516 maximum initialized bytes versus 21,276 in
the preceding same-deck stage. Allocator metadata, spare capacity, COW reference
counts, observations and external snapshots are outside that payload metric.

This stage also fixes a missed loyalty-counter fact in zone-change last-known
information. `DamageFacts::from_view` now uses the same all-kind counter total as
live facts, so a creature with only loyalty counters remains counter-qualified
when its source leaves. A focused transition fixture checks both representations.

Author validation: 620 Rust tests (610 unit and ten integration), 74 release Python
tests, catalogue consistency, rustfmt and Clippy with warnings denied. Eleven new
native tests cover separate defenders for either active player, pending privacy,
invalid-action atomicity, creature constraints/vigilance, blockers, departure and
return, trample, first/double strike, huge numeric assignments, exact clone/restore
and the last-known counter fact. The actual C ABI fixture uses an explicitly
injected unsupported future walker and checks legal serialized attack, rejected
stale reference, pending privacy, clone, restore and loyalty damage. The Python
fixture uses admitted Bear Cub decks and independently checks typed player
attacks, privacy, forgery rejection, public commitment and snapshot/replay.

One hundred supported Standard damage-body games completed naturally with 145,800
decisions, 3,746 independent public checks and 35,870 clone/replay boundaries. These
public games validate existing creature/player behavior; they do not establish
playable walker bodies or full Standard acceptance. The all-Standard/RL/search
goal remains active, and issue #93 remains open for the activation and card work.

An isolated local unpinned release `benchmark 1000` sample produced 571,049 decisions
at 401,446 decisions/second (703 games/second), clone 372 ns and restore 290 ns.
An earlier sample run alongside the public rollouts produced 388,683 decisions/
second, clone 285 ns and restore 272 ns. The preceding stage recorded clone/restore
near 111/108 ns, so these samples do not establish snapshot performance stability
or an attributable change. The initial state payload is unchanged; fuller profiling
remains required. These limited smoke timings include policy/observations and
exclude model inference, encoding, JSON and a comparable Java workload.
The independent library reviewer later measured 396,011 decisions/second, clone
89.25 ns and restore 86.44 ns in the same benchmark. The slower snapshot sample did
not reproduce; no stable regression or speedup is established by these samples.
