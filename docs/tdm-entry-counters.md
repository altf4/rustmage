# Entry counters and counter-placement events

This checkpoint implements shared rules for Dragonstorm Globe, Hundred-Battle
Veteran, Perennation, Magmatic Hellkite, Smile at Death, Hollowmurk Siege and
Stalwart Successor. It does not author or admit their card definitions. The
Veteran's distinct-counter-kind statistic remains the separate counter-world
checkpoint, and each card still requires its complete printed behavior tests.

## Typed interfaces

- `ControlledSubtypeEntryCounters { subtype, kind, amount }` adds a fixed quantity
  from an existing controlled provider to matching entrants. Both actual and
  prospective entry worlds use the entrant's replacement characteristics,
  including copied definitions and as-entry type choices. A provider entering in
  the same event is absent from the pre-entry world. A provider departing in the
  same event still supplies its replacement effect.
- `CardExtras::graveyard_cast_entry_counters` applies only to an actual spell
  whose cast origin was the graveyard. The permission used to cast it does not
  change that origin. A direct return has no cast origin; a cast spell copy does
  not inherit the physical card's origin. Ordinary payment, timing and countered
  spell destinations remain in the existing casting pipeline.
- `ReturnToBattlefieldWithCounters` carries an immutable mixed counter payload
  into the existing pre-entry replacement transaction. Entry observers see the
  final replaced or prohibited counters immediately.
- `SearchTargetControllerWithEntryCounters` uses the legal target's controller
  frozen at resolution, even if destruction fails or resets the card's
  controller. Its search results use the same mixed pre-entry payload. Search
  selection, failure to find, shuffle, privacy and owned continuation remain in
  the ordinary search transaction.
- `ReturnToBattlefieldThenCounters` retains the exact successful new incarnations
  through the entry batch's choices and failures. It freezes all entry observers
  first, then starts one separate ordinary counter-placement replacement batch.
  Failed returns and stale targets receive no counters. Prospective worlds
  evaluate these later replacements against the completed entry world and commit
  their final quantities together.
- `ControlledCreatureCountersPlaced { first_only }` observes each positive
  recipient placement event, regardless of counter kinds or placing player.
  Its event subject uses the existing exact `entering_object` witness and
  `Subject::Entering` instruction path; that name also serves counter-event
  recipients. Conditional named-mode printed triggers and `OncePerTurn` wrappers
  compose through ordinary trigger queuing.

## First placement

The event dispatcher derives first-event qualification from the existing public
exact-object counter-placement history across all placing players. History is
source-independent: placement on a noncreature, placement before an observer
arrives, removal of counters and a later control or in-place copy change cannot
restore first-event status. A new turn clears the history, and a true zone change
creates a different incarnation.

All positive quantities in a committed placement batch record history before
counter observers queue. Mixed kinds and repeated rows for one recipient cannot
create multiple first-event packets. Fresh entry incarnations already record
entry counters before entry observers inspect their facts; those positive entry
counter events are necessarily first. Counter prohibitions and zero placements
record nothing.

The first-event result qualifies the queued packet. Further placement in response
does not turn that historical fact into a live aggregate test. The added counter
from a Stalwart-shaped packet is a later event and cannot trigger the same
first-event ability recursively. The recipient may be the observer itself.

No new serialized history state or public observation field is introduced.
Existing shared vectors, entry batches, replacement state, clone/restore and
owned-memory accounting retain these contexts.

## Focused scenarios

`tests/tdm_entry_counters.rs` uses paid typed fixtures on existing admitted
identities. The assertions distinguish these outcomes:

| Obligation | Expected result |
| --- | --- |
| Fixed Dragon addition | One counter per existing provider, independent of existing Dragon count; opponents receive none. |
| Simultaneous providers | A new provider supplies no counter to a simultaneous entrant; a departing provider supplies its existing replacement. |
| Replacement characteristics | An as-entry Dragon type choice qualifies, including prospective entry queries; counter doublers replace the resulting quantity. |
| Graveyard casting | Actual graveyard casts receive finality; hand casts and direct returns do not; normal mana payment and failed announcements remain transactional. |
| Other permission and countering | An independent permission preserves graveyard origin; a countered spell returns to the graveyard and may be cast again; finality remains after ability loss. |
| Mixed pre-entry counters | Hexproof and indestructible exist for entry observers and keyword rules, pass through replacements, and can both be prohibited without canceling the return. |
| Later counters | Entry observers see the return before its later counters; successful results survive entry choices while failed or stale results receive none. |
| Post-entry replacements | All successful return recipients receive the replaced later quantities; prospective outcomes agree without mutating the parent. |
| Searched land | The frozen controller searches even if destruction fails, and the result enters tapped with stun; illegal targets cause no search and a zero selection still shuffles. |
| Own-creature observation | Both placing players can trigger the selected mode; opposing recipients do not qualify; mixed counters and later events respect the once-per-turn cap. |
| First placement | Mixed kinds create one packet; another placement in response does not disable it; earlier history survives late observers, control changes and in-place copies. |
| Entry/self/prohibition | Entry counters can trigger their own first-event observer; prohibited attempts do not consume first-event status. |
| Ownership/privacy | Private search payloads and suspended post-return payloads survive raw/index branches, parent disposal and restore; wrong actors and stale indices preserve state. |

Source hashes, authoritative input provenance and terminal guarded run results
are recorded separately in the evidence JSON after validation. A focused pass is
not full-set, simulation-throughput or release acceptance.
