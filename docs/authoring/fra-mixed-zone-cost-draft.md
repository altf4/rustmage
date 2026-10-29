# Mixed-zone cost draft

The shared draft was tested at commit `dc5cc7d0`: guarded clippy passed and the
grouped test suite passed 2,762 tests. Its Sphinx, Emrakul and mixed-zone
modules are linked under assurance. Sphinx's Approach has a conditional
admission candidate with its trace recorded; Emrakul remains gated. These
results do not establish the outcome of any later in-progress run or replace
final validation and the required independent nonauthor reviews.

Sphinx's optional resolution payment calls the shared atomic zone-cost planner
for the spell and four named graveyard cards. The planner checks exact object
incarnations, duplicate components and the typed graveyard-exile prohibition
before planning replacement outcomes. Non-entry outcomes are revalidated before
commit; outcomes redirected to the battlefield use staged entry and an owned
prepayment checkpoint with an explicit unpaid-program continuation. Emrakul's
ward sacrifice payment uses the shared planner through its atomic sacrifice
helper; that helper does not use Sphinx's staged-entry continuation.

Under CR 118.11 a replacement-modified cost, including a prevented event,
remains paid. CR 614.17b prevents choosing an impossible cost and 614.17c
permits a true self-replacement to modify it before other replacements. This
preflight handles its ordinary prohibition and replacement cases but does not
model the self-replacement exception. A static replacement printed on a
departing object is not automatically a self-replacement under CR 614.15.
Do not treat the planner as complete for costs involving that exception.

The staged checkpoint records all components, including prevented ones, and
restores the unpaid continuation if an incarnation changes. Rollback transfers
the currently live zone replay into the restored state: an active replay is
retained for synchronous failure and an already-consumed replay is discarded.
The corresponding competing-replacement and stale-incarnation tests use
synthetic providers and injected stale state; no supported public path has
established those nested failures. The synthetic Sphinx redirect forces an
instant card onto the battlefield solely to test plumbing. The separate Watery
Grave fixture tests a real entry-life choice with a constructed replacement plan,
not a supported redirect provider. Replacement ordering, source and controller
changes, triggers, mana replay and transaction boundaries have focused test
coverage but still require final independent review and validation of the
published inputs.

The candidate reconciles copy-aware card-only consumers through the earlier
239f8bb5 draft and preserves a spell copy in exile until the next state-based
check. A public-action copied-Sphinx test and synthetic transient-zone tests are
linked and included in the grouped run. The battlefield-copy classification
remains unresolved; final independent review of copy privacy, replacement
interaction and the broad consumer migration is still required.

Projected funding is not complete for every prevention or replacement path.
In particular, repeatable non-tapping mana abilities whose sacrifice is
prevented can create an unbounded search; restricted mana requires a sound
purpose-aware termination argument. The experimental funding work and its
remaining limits are recorded in `fra-emrakul-exigent-draft.md`. The known
repeatable prevented-sacrifice witness uses synthetic providers; absence of an
identified real provider does not establish general termination. Admission
requires the applicable final validation and independent reviews, and any
supported path to a shared limitation must be addressed first.
