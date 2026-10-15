# Face-up exile and until-source-leaves returns

Exile is a distinct public zone. Its face-up objects remain visible to both
players, with their own generations. Direct targeted exile and graveyard exile
share central zone transitions; exiling a permanent is neither destruction nor
dying. Indestructible does not prevent it, and death triggers do not fire.
Face-down exile, casting permissions and re-exiling an already exiled object are
not yet admitted.

ExileUntilSourceLeaves records both exact ObjectRefs and the return zone in owned
State. If the source incarnation already left before its ability resolves, the
initial exile does nothing. Returning that physical card as a new incarnation
does not satisfy the old source condition. If the source later leaves, the paired
one-shot return occurs immediately after that departure event, without using the
stack or offering priority. It returns the exiled incarnation only, not another
object subsequently represented by the same physical slot.

Central completed zone batches dispatch their original events and then process
expired links. Multiple returns caused by simultaneous departures move together,
before any entry trigger matches the resulting battlefield. Static-source indices,
generations, sickness, counters and temporary fields follow normal transitions.
Returned permanents come under their owner's control in the current engine;
control-changing effects remain gated. SBA source deaths and sacrifice costs use
the same completed-event path. Drawing or moving unrelated library cards does
not scan/reallocate the live link records.

Tokens still cannot move again after leaving the battlefield, including before
their cessation SBA. Exiled tokens cease at the normal SBA boundary and discard
their held records; reused token slots cannot be returned by stale links. Other
exiled incarnations leaving exile also discard their return records.

Schema **13** adds public `exile_links` entries (source, exiled, return_zone) and
Zone::Exile object projections. Snapshots and clones own every link, and state
accounting includes initialized link storage. No host callback or external map is
needed to restore pending exile/return relationships.

Shared target predicates now support fixed mana-value bounds and tapped-only
conditions. They apply at announcement and resolution. Native objects have fully
implemented fixed costs; unsupported X/multiface cost semantics are still gated.
Complete texts admitted by this work include Banishing Light, Glass Casket,
Seam Rip, Stasis Snare, Henchbots, White Auracite, Soul-Guide Lantern, Angel of
Finality, Epic Downfall and Wander Off. Eighteen additional Standard definitions
bring coverage to 364/5,164, with six regression cards and 29 token factories.

Tests cover public projections, normal/early source departure, source reentry,
simultaneous returns with mutual entry triggers, source SBA death, counters and
sickness resets, tokens before/after cessation, stale held incarnations,
tapped/mana-value filters, graveyard exile, destruction distinctions and replay.
Python verifies public held records and fresh returns through clone/restore.

Wider targets, delayed returns, face-down exile, play/cast permissions, linked
abilities referring to exiled characteristics, replacement effects and remaining
card/rules mechanics are outstanding. Full native RL/search and all-Standard
acceptance remain open.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 406, 610.3, 111.8, 603 and 704.
