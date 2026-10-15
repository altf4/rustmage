# Legend rule and state-based choices

The native engine enforces CR704.5j for admitted legendary permanents. A player
controlling multiple legendary permanents with the same name chooses one to keep;
the others go to their owners' graveyards. Different names and different
controllers do not conflict. This event does not destroy or sacrifice anything,
and indestructible cannot prevent it. A player may keep a legend already dying
from another state-based action, in which case every copy may leave.

`Action::KeepLegend { object }` selects an exact incarnation. The active player
makes all their name-group choices before the other player. Within their groups,
a player may choose in any order. Choices and earlier selections are public under
CR101.4. No priority action is offered while choices are pending. Concession is
still available. Observation schema 17 exposes `legend_choices` and the
`legend_rule` decision to both viewers, with action indices only for the actor.

The engine freezes deaths, counter-pair cancellation, token cessation and player
losses before asking for a legend choice. These actions commit together after
all choices. Departure information is captured before movement/cancellation;
linked returns and subsequent state-based passes happen before pending triggers
are placed or priority resumes. A returned object gets a new incarnation and is
not modified by the earlier counter-cancellation batch. Cleanup preserves the
suspended decision and gives the required extra priority cycle after expiration
causes state-based actions.

The continuation, selections and resume point belong to each game/snapshot. Only
suspended legend events allocate a boxed continuation; ordinary checks scan the
object arena once and commit without that allocation. `state_bytes()` includes
initialized continuation storage. It is a storage estimate, not allocator capacity
or a throughput measurement. `standard_legends` exercises native Standard games
with clone/restore at every legend choice; Python tests cover the C ABI path.

Ten complete legendary definitions are now executable, bringing Standard coverage
to 547/5,164. Names, supertypes and control are immutable for admitted cards;
name/type/control changing effects, copies, planeswalkers and legendary instant/
sorcery casting restrictions remain gated. A supported body never permits
unsupported residual text. This milestone does not establish full rules or full
Standard conformance.

Rules source: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
CR101.4, 514.3 and 704, particularly 704.5j and 704.7.
