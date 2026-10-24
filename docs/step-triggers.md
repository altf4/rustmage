# Step events and intervening-if conditions

The native engine queues beginning-of-step triggers when it enters upkeep, draw,
main phases, beginning/end of combat or the end step. Conditions can match the
source controller's turn, an opponent's turn or each turn. Entering a permanent
during an already-running step never retroactively creates its beginning trigger.
A skipped draw step has no beginning event. The ordinary draw, state-based actions,
APNAP trigger ordering and target declarations finish before priority resumes.

An intervening-if clause immediately following the trigger event is evaluated
both at that event and before the ability resolves (CR603.4/608.2a). A false
condition suppresses the trigger or makes its resolution do nothing. The engine
does not turn later conditional effect text into this kind of condition.

Exact predicates cover tapped source state, committed attack history, current
controlled creature/land/tapped-creature counts, derived creature power and total
power/toughness, hand/life thresholds and total spells cast during the turn. Many
predicates remain vocabulary for cards whose other text is unsupported. Current
power/toughness queries use the shared counter, temporary-growth and continuous
modifier calculation. An empty battlefield has no greatest creature power;
its total creature power is zero.

Raid records committed attacker declarations, including vigilant attackers.
Selecting an attacker alone does not set the history bit. Removing every attacker
or ending combat does not erase that turn's history. Both players' history resets
at the start of the next turn, alongside the existing cast counts. Observation
schema 19 exposes `players[].attacked_this_turn`.

Survival checks the source's current tapped state while its exact incarnation
remains on the battlefield. After departure it uses the tapped state captured at
departure; a returned incarnation is independent (CR608.2h). Stack views expose
`source_tapped`, which is captured trigger/activation information and updated at
departure, rather than a live mirror of the source's state. The zone-transition
hook updates queued, ordering, announcing and resolving items as well as the stack.

The live step-source index, history, last-known fields and every suspended choice
are owned by the game and its snapshots. `state_bytes()` includes initialized
index storage. Native tests cover condition changes during responses, competing
end-step triggers, derived-power and land thresholds, source departure/return,
late entries, private/optional target continuations, raid and token creation before
state-based loss. The Standard example replays step boundaries and pending choices;
Python verifies a survival trigger, public history and branch/restore through the
native library.

Supporting exact instructions add filtered creature untapping (respecting stun),
atomic fixed draw/life-loss and fixed life-loss/token sequences, and optional
`have target creature get` modifiers with required target declarations.

Seventeen complete definitions bring Standard coverage to 599/5,164, plus six
regression cards and 35 token factories. Delayed triggers, additional/skipped
phases beyond the existing first-draw rule, conditional costs, step-trigger modes,
wider history and unsupported residual text remain gated. This does not establish
full Standard or RL/search acceptance.

`StepGuard::AttachedControllerTurn` qualifies an attachment using its exact
battlefield incarnation and the exact attached permanent's current controller.
The live and speculative funding worlds use the same predicate. For a beginning
trigger whose wording names the enchanted permanent's controller, wrap the step
in `TriggerCondition::When` and keep the inner `AtStep` guard `Always`; attachment
or control changes after the event must not become a resolution recheck.

Super Intelligence uses this guard with upkeep/any-player and draws through the
existing `PlayerSet::Active`. The upkeep player stays active while that queued
packet remains pending, stacked or resolving. Legal turn advancement waits for
an empty stack, while ending the turn removes the packet. Consequently a later
attachment change, control transfer, source departure or recipient departure
cannot redirect the already queued draw. This narrow prerequisite adds no wire
state and admits only the complete Super Intelligence representative. Its
both-seat paid, privacy, owned replay and funding evidence is recorded in
[msh-super-intelligence-evidence.json](msh-super-intelligence-evidence.json).

Rules: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
CR603, 608.2a/h, 504 and 513.
