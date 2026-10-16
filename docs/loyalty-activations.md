# Loyalty activation costs, life-event emblems and a complete Standard walker

[Issue #93](https://github.com/altf4/rustmage/issues/93) now has its fixed loyalty
activation foundation as well as the preceding entry/damage/combat foundations.
Professor Dellian Fel's complete printed body is admitted. Coverage rises to
987/5,164 pinned Standard identities (4,177 unsupported), six regression definitions,
43 token factories and one separate emblem factory. The [admission record](loyalty-activation-admissions.json)
contains the complete Oracle body and both card/emblem Java source hashes at pinned
XMage revision 492369c7844b8d76479f0393ff9eb6b0b0cf30d7.

The pinned [September 25, 2026 rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
are the reference for loyalty costs/limits (606), mana classification (605.1a) and
emblems (114). A fixed signed loyalty cost is distinct from removing counters as
an ordinary activation cost. Adding, zero-changing and removing loyalty happen
when costs are paid, before resolution. Negative changes require enough counters;
positive representation overflow returns the existing recoverable capacity error
and restores the whole action, including an in-progress announcement and revision.
Targets are chosen before payment. A zero-loyalty source dies before its ability
resolves, while the independent stack ability continues. Fizzling doesn't refund
loyalty or the activation limit. Regular untargeted/tap/sacrifice/counter activations
retain their existing behavior.

The controller must have priority in their main phase with an empty stack. Tapping
and summoning sickness don't restrict loyalty costs. The once-per-turn limit is
shared across every loyalty ability of that permanent, using the existing owned
`used_abilities` mask and immutable definition mask. It is consumed on payment,
preserved across resolution and cleared by turn/zone-incarnation resets. There
is no extra dense per-object limit field. A loyalty ability that produces mana
still uses the stack; it never becomes a mana ability or a virtual funding source.
Variable loyalty costs, modifiers/extra activations, control changes and other
unimplemented printed mechanics remain gated with their whole card bodies.

| Professor Dellian Fel ability | Implemented behavior |
| --- | --- |
| +2 | Pay two loyalty counters onto the walker, then gain three life on resolution |
| 0 | Consume the shared limit, then draw one card and lose one life |
| -3 | Pay three loyalty after choosing a creature; destroy that target on resolution |
| -6 | Pay six loyalty, then create an independent life-gain emblem |

Emblems use immutable generated definitions and owned arena object references in
`Zone::Command`. They are neither tokens, cards nor permanents: no card types,
colors or mana cost; deck submission is rejected in every mode. Internal catalogue
identifiers use `@emblem/…`; they are identifiers, not in-game names. Command-zone
objects are public and their abilities function independently of the walker.
Ordinary permanent targeting/destruction doesn't reach them, they don't generate
battlefield entry/death events, they cannot change zones, and token-ceasing/reuse
logic never handles them. Creation registers the supported life-gain observer;
multiple emblems create distinct instances and triggers.

`LifeTriggerView { player, amount }` captures each positive life-gain event on
`SpellView.life_event`. It is public, exact i128 state owned by every suspended
trigger/announcement/resolution and snapshot. Later gains never overwrite it.
The emblem announces an opponent target, then makes that opponent lose precisely
the captured life amount; this is life loss, not damage. The admitted emblem family
is exact; unknown emblem text or residual clauses fail whole-card compilation.
Only factories referenced by fully accepted bodies enter the catalogue.

Observation schema **40** adds life-event context and `ObjectView.emblem` metadata.
The C ABI remains version 1 and optional control responses remain schema 2.
Snapshots retain complete game-local state. No server or JSON is involved in
native simulation. The initial benchmark payload is 19,616 logical bytes versus
19,552 before this stage, due to captured event space in owned continuations;
the dense object layout is unchanged. Metadata, spare capacity, COW reference
counts, observations and external snapshots are excluded from that payload metric.

Author verification: 632 Rust tests (621 unit and eleven integration), 110 compiler/
data tests, 76 release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. Eleven new native tests cover every cost sign/body, shared limits,
sorcery timing, source generations, zero-loyalty source death, target fizzle,
forged payments, direct and staged capacity rollback, mana classification, command
lifetime, multiple/wide captured life events and owned replay. Catalogue tests
reject generated emblems in every deck mode. Compiler tests reject incomplete
walker bodies, unsupported loyalty expressions and unknown emblem effects.

Actual public Python/C ABI play tests all four abilities with immediate payment,
clone/restore and stale revisions. A second public test creates the emblem, attacks
and kills the real admitted walker, then gains life with Moment of Craving and
replays the independent emblem's captured life-loss trigger. It also rejects
serialized emblem factories as submitted cards. The earlier synthetic future-
walker fixtures now use Vivien Reid where they previously used unsupported Dellian;
they remain clearly separate from these actual admitted gameplay tests.

One hundred `standard_planeswalkers` games use legal 60-card Standard decks with
four copies of Dellian and end naturally: 97,979 decisions, 3,952 independent public
checks, 24,532 clone/replay boundaries and 21,180 maximum initialized state bytes.
They resolve all four abilities, 317 planeswalker attacks, 60 emblems and 184 emblem
triggers. This proves the represented bodies on those fixtures,
not complete Standard rules. Other walker bodies remain unsupported until all
abilities and dependencies are implemented. The full all-Standard/RL/search goal
remains active.

A local unpinned native `benchmark 1000` sample produced 571,049 decisions at 398,632
decisions/second (698 games/second), clone 375 ns and restore 300 ns. Prior independent
samples of the preceding stage varied substantially, so this does not establish a
stable attributable speed change. Timings include smoke policy/observations and
exclude inference, encoding, JSON and a comparable Java workload.
