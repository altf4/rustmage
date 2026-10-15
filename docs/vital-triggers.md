# Life-gain and creature-death events

Explicit gain-life instructions and lifelink damage use a shared life-gain hook.
Zero gain produces no event. Damage from one lifelink source to several recipients
in one batch produces one event; different lifelink sources produce separate
events. First-strike and normal combat damage are separate batches, with trigger
resolution and priority between them. Dispatch precedes state-based actions, so
an observer dealt lethal damage in the batch still sees that batch's life gain.
Queued triggers enter the existing APNAP ordering/target-declaration machinery;
they do not resolve during the event or interrupt a resolving instruction.

Each player's `life_gain_events_this_turn` counts positive gain events, including
when no observer exists. Both players' counters reset at the beginning of every
turn. Exact first-gain triggers inspect this history, and own-turn conditions also
inspect the active player. A source entering or returning after the first gain
cannot retroactively observe it or reset the player's history. Observation schema
21 exposes the public counts. Snapshots own this history and both source indexes.
`state_bytes()` includes initialized index storage. Death-source capture now uses
an index instead of scanning every object slot for each departure.

Creature-death events inspect the battlefield immediately before the whole
movement batch. A dying source sees other creatures that die simultaneously, and
a condition without “another” can see its own death. Exact filters distinguish
controller/opponent/any player, other incarnations, nontokens and supported single
subtypes. Tokens can die before ceasing to exist. Exile, hand-to-graveyard changes
and noncreature permanent departures do not produce creature-death events. Source
keywords and references retain last-known information; a returned source is a new
incarnation. Linked returns and state-based actions keep their existing ordering.

Compiler additions recognize only complete event/effect clauses. They also handle
explicit source-name entries/counters, source names shortened at a legendary
card's comma, and whitespace left by removing reminder text. Event-subject
pronouns are not inferred to mean the source. Separate exact effects cover fixed
opponent life loss and source damage to its controller followed by a fixed draw.
All other text still has to compile before a definition becomes executable.

Twenty-three complete Standard definitions become executable:

| Cards | Newly available semantics |
| --- | --- |
| Ajani's Pridemate; Pest Mascot; Tigra, Feline Fury | Life-gain source counters |
| Marauding Blight-Priest; Drogskol Reaver | Life-gain life loss/draw; double-strike lifelink |
| Cat Collector; Vanguard Seraph | First-gain history, own-turn restriction, token/surveil continuations |
| Agent Venom; Midnight Reaper; High-Society Hunter | Filtered nontoken deaths, source damage/draw, attack sacrifice payment |
| Boggart Cursecrafter; Voracious Vermin | Subtype/other-creature death events, token deaths |
| Sephiroth, Planet's Heir | Opponent deaths following entry debuffs |
| Pulse Tracker; Engine Rat; Shroudstomper | Fixed opponent life-loss effects |
| Dori, Bearer of Friends; Glacier Godmaw; Professional Wrestler; Provisions Merchant | Complete entry/token text with trailing reminder whitespace |
| Gladiolus Amicitia; Starling, Aerial Ally; Thrór's Map | Explicit source-name entry effects and existing search/target continuations |

Coverage is 634/5,164 Standard identities, six regression cards and 35 token
factories. Variable gain/draw amounts, life-loss triggers, replacement/prevention,
once-per-ability turn restrictions, one-or-more death batches, wider death filters,
control changes, granted/copied triggered abilities and residual card semantics
still require implementations. Cards containing those unsupported clauses remain
unavailable even when another clause uses these events.

Native fixtures check APNAP action replay, source indexes across every admitted
definition, history reset/late entrants, lifelink grouping, double-strike windows,
simultaneous observer deaths, exact filters, departed source LKI, illegal sacrifice
rejection and chained triggers. Python exercises both event families through the
shared library with clone/restore. The supported Standard example completes games
and replays event/choice boundaries. These checks do not establish full Standard
acceptance or an XMage speed comparison.

Rules: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
CR119.9, 201.5/201.5c, 603.3/603.10a, 700.4 and 702.15.
