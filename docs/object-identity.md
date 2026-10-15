# Object identity and zone transitions

A `CardId` identifies a physical card slot. `ObjectRef { card, generation }`
identifies that card's incarnation in a particular zone of a game branch. Every
actual zone change increments its generation. Reordering within a zone does not
create a new object. References are scoped to a game and snapshot, not process-wide
identities. Restore reproduces generations and still increments the external
revision used to reject stale action indices.

All production zone mutations use `Game::move_object`. It maintains library
membership, removes departing stack entries and combat memberships, clears tapped
state, marked damage, deathtouch marks and temporary growth, and initializes
summoning sickness for a new permanent. A surviving attacker remains blocked when
its last blocker leaves. The returned `ZoneChange` owns a pre-transition object
view and the new reference. Hidden transition values are not added to observations.

Spells retain both their own stack reference and their target's selected reference.
Resolution rechecks incarnation, zone and current creature status. A creature that
leaves and returns cannot receive an earlier Giant Growth. A counterspell cannot
counter a different incarnation of a spell using the same physical card slot.
Damage sources retain incarnation alongside captured controller and keyword flags,
so separate incarnations remain separate sources when grouping life-gain events.

The exact full-text templates for `Return target creature to its owner's hand.`
and `Destroy target creature.` add Unsummon, Fell, Murder and Bilbo's Deadly Slice
from the pinned Standard pool. Templates require a normal instant/sorcery with a
fully implemented fixed mana cost. No admitted permanent currently has a relevant
destruction or zone-change replacement, indestructible or regeneration.

Observation schema **4** adds `ObjectView.reference`, `SpellView.object`,
`SpellView.target_object`, `DamageSource.reference`, and `Observation.blocking`
(current blocking creatures, including those whose attackers have left). Action JSON continues to
use physical card IDs for submitted choices; only validated casts bind internal
references. C function signatures and protocol response schema remain unchanged.
The catalogue fingerprint changes because compiler admissions change.

This is groundwork, not a complete event/trigger engine. Returned events are not
yet dispatched or retained in a trigger queue. Simultaneous leave-trigger discovery,
zone-following exceptions to rule 400.7, stack-to-permanent characteristic changes,
control layers, tokens and reanimation remain gated behind future implementations.
The LKI view covers currently implemented characteristics, not those future systems.

Rules basis: [Comprehensive Rules, effective September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
400.7 (new objects), 403.4 (new permanents), 506.4 and 509.1h (combat membership),
608.2b (target legality). Regression tests cover stale targets after return/recast,
transient-state reset and owned LKI, all current zone routes/library indexing,
blocking and source identity, combat-step preservation after removal, real
Unsummon responses through both internal fixtures and public gameplay, mana/timing/targets for all
four spells, and restoring pending targets. Shared-library tests cover complete
Standard removal games and replaying a cast through snapshot/restore.
