# Reality Fracture: outside-game card selection

This is an unimplemented dependency, not evidence of card support. Extrapolate
the Impossible is present in the pinned Standard Oracle inventory, but the
pinned XMage inventory lists no implementation for it. The current engine
configuration accepts main decks only; its `Zone::Outside` is also used as an
internal staging and recycling zone for tokens and spell copies. Neither that
zone nor the global catalogue is a player's sideboard.

Wizards' [Reality Fracture release notes](https://www.magic.wizards.com/en/news/feature/reality-fracture-release-notes)
state that the spell may reveal exactly two owned cards with different names;
an opponent chooses one, and the other remains outside the game. In tournament
play the available cards come from the sideboard; casual play instead permits a
personal collection. Limited sideboards include an arbitrary number of basics.

A correct implementation needs an explicitly supplied, snapshot-owned pool for
each player, with a selected rules mode defining its bounds and legality. Do not
use every catalogue card, silently invent a sideboard, expose another player's
unrevealed pool, or interpret internally staged token/copy objects as owned
outside-game cards. The current `Config` is constructed with Rust struct literals
throughout the API, so an added required field would break existing callers;
a compatible constructor or a deliberate versioned configuration migration and
matching FFI support are needed. Constructed validation must apply deck limits
across the main deck and sideboard, and any chosen policy for casual collections
and Limited basic lands must be explicit.

Resolution needs a private, cancelable selection of exactly two distinct card
names, followed by public revelation of both and an opponent-owned choice. If
fewer than two eligible distinct names exist, declining remains legal; choosing
one card alone is not a legal completion. Move only the selected physical card
into its owner's hand and leave the other in the outside-game pool. A repeated
spell must see the remaining pool and any relevant state changes. The selected
card is publicly known after revelation, while the rest of the pool and the
intermediate first selection remain private. All decision steps, ownership,
state transitions and random state must survive owned raw/index branching,
restore, replay, and disposal of the parent. Define capacity and overflow
behavior rather than sampling or imposing an undocumented hidden limit.

Acceptance requires production Rust and FFI entry points, complete card and
source provenance, and tests for empty and duplicate-only pools; decline;
nonowner rejection; opponent selection; repeated use; visibility before and
after revelation; invalid actions with transactional rollback; both player
seats; deck/sideboard legality; Limited basic lands; and owned branch/restore.
The card remains gated until these requirements, normal release validation,
and the batch's independent reviews are complete. The missing pinned XMage
source must be reconciled with the repository's provenance requirements rather
than represented as having been reviewed.

## Proposed typed boundary

Keep `Config` and `Game::new(Config)` source-compatible: introduce a separate
`OutsideGameConfig` and `Game::new_with_outside(Config, OutsideGameConfig)`.
The original constructor supplies empty pools. The new constructor validates
both pools and the main decks before allocating or exposing any game state.
For C/JSON callers add a distinct versioned construction entry point accepting
an envelope containing the existing `Config` and the outside configuration;
keep `rm_new` unchanged. Expose the new constructor through the Python binding
with an explicit optional argument, and version/document the control contract
if its accepted message shape changes. Do not add a required public `Config`
field or mutate a running game's pool through an unauthenticated request.

Use an explicit mode, such as `ConstructedSideboard`, `LimitedSideboard`, or
`CasualCollection`, rather than deriving a tournament sideboard from Freeform.
A two-player `OutsideGameConfig` owns two finite lists of physical card
identities, with stable per-entry identifiers. Constructed mode validates the
15-card bound, combined deck/sideboard card-name limits and selected format's
legality, including per-card exceptions; unsupported rules identities must fail
closed. Limited mode needs an explicit virtual supply of basic land identities
that can be materialized on demand, without an invented finite cap; casual
mode uses the supplied finite owned collection. Validate each mode's allowed
relationship with `DeckRules` instead of silently treating Freeform as either a
tournament or casual ruleset. Check copy limits by rules name, including
multi-face/name exceptions, not just raw catalogue index. Validate the pool's
cards as physical cards rather than token, emblem, or synthetic face identities.
Keep declared resource/capacity failures explicit and atomic.

Represent pool entries separately from `Object` and `Zone::Outside`: the latter
is an implementation staging location and outside the game is not a game zone
(CR 400.11). An outside entry does not receive an in-game object ID until the
chosen physical card enters the game. On successful admission, allocate its
normal object with the choosing player as owner (CR 108.3), put it into that
player's hand, remove only that physical entry from the pool, and use the
engine's public knowledge tracker for the revealed card. A virtual basic is
materialized without depleting a finite supply. The unchosen card remains in
the same outside pool. Do not issue a fabricated zone-to-zone change event for
an outside-game card merely by reusing internal `Zone::Outside`; explicitly
review replacement and observation semantics for bringing a card into a hand.

The resolving spell owns a resumable typed selection state with controller,
choice phase and stable private entry IDs. Offer decline before committing any
selection; otherwise privately select two distinct eligible entries whose rules
names differ, reject wrong-actor, duplicate-entry and equal-name actions, and
allow a first selection to be cancelled while still private. Only after a valid
pair is complete reveal both together, retain their public identities and ask
the opponent to choose one. There is no priority or opportunity to respond
between these resolution choices. Opponent selection is mandatory once a pair
is revealed; the earlier optional decision cannot be retracted after revelation.
Use owned entry IDs and revalidate them when applying each action. No partial
pool removal, token allocation, or disclosure occurs on an invalid action.

The pool owner’s observation may contain their available pool
and first private selection; CR 723.4 does not expose outside-game cards to a controller of that player. the other player's projection must contain neither
identities, selection progress nor action IDs that identify unrevealed entries.
After revelation both viewers see the pair; the opponent alone receives the
choice actions. Keep the revealed pair in public knowledge and preserve knowledge
of the selected card in hand. Ensure action enumeration, errors, counts, traces,
clones and raw/indexed snapshots cannot leak the rest of either pool. Existing
clones and rollback own all pool and continuation data; a parent may be disposed
without invalidating descendants. Measure and account for pool and selection
allocations in existing memory/capacity reporting.

Public actions and observations are wire changes and require centralized schema
and protocol review, a new observation version when the wire actually changes,
and a conservative projection that rejects unrepresentable outside-game state,
including hidden pools, suspended choices, nested packets and rollback. Test the
new Rust constructor and FFI/Python entry point alongside old calls, and retain
fixed-input validation and two independent nonauthor reviews. This design is a
proposal, not an implementation or a change to the card's support status.

The draft permits only decline when another player controls the pool owner.
CR 723.4 hides outside cards from the controlling player; Wizards' [Mindslaver
ruling in the Outlaws of Thunder Junction release notes](https://media.wizards.com/2024/downloads/OTJ_Release_Notes_nwYBwey2D91L/EN_MTGOTJ_ReleaseNotes_20240307.pdf)
says that player cannot have the controlled player choose a card from outside
the game. Tournament rule 3.16 also forbids accessing that player's sideboard.
Publicly known outside cards remain visible, but do not change that restriction.

## Outside-to-hand event audit

The [June 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260619.pdf)
state in CR 400.11 that outside the game is not a zone, and in CR 108.3b that a
sideboard card is owned by the player who started with it there. CR 403.4 says
an object entering a zone is a new object. Therefore the draft allocates a new
physical hand object for the pool owner and records the revealed hand identity;
it does not manufacture a prior in-game `Zone::Outside` object or assert a
battlefield-to-hand return. CR 402.1 distinguishes cards put in hand from
cards drawn. The action test verifies no draw count or second-draw trigger,
no false battlefield-return trigger, continued public knowledge, and an actual
subsequent cast, with Rest in Peace in play and snapshot replay.

This is a supported-path conclusion, not a proof that all replacement effects
or observers are implemented. General replacement effects act on events under
CR 614 and must be evaluated if a future rule can replace bringing an outside
card into a hand; `engine_zone_replacements` currently models moves from an
existing `ObjectView` in an in-game zone. A correct extension needs a distinct
pre-event identity and typed origin for an outside card, owner-based affected
player, replacement ordering/suspension, and a single transactional commit. It
must not call the existing staging-zone move machinery as if `Outside` were a
real zone. Whether an observer phrased as putting a publicly visible card into
a hand from anywhere triggers for an outside card requires source-backed review;
no such applicable observer or hand-entry replacement provider was identified
in the currently supported Standard rules. Keep the definition gated until
this distinction, public schema changes, tests and independent reviews pass.

## Schema 189 candidate

The isolated composition starts at the published schema-188 baseline and adds
the outside-pool fields and actions as schema 189. The prior-schema audit first
rejects outside-game owned state, rollback, retained packets, new card identities,
and outside wire fields or actions before applying the existing 188-to-187
projection. Pre-existing schema-188 behavior remains representable. This is a
conservative exception, not a claimed trace audit: the linked tests and complete
historical trace comparison have not run. General outside-to-hand replacement
semantics and the missing pinned XMage class remain blockers; Extrapolate remains
gated.

## Owned outside-entry plan draft

The isolated plan/commit foundation retains the selected pool slot (or virtual
basic), owner, card, choice-instance ID and pool mutation version in owned state before committing.
A stale slot, changed choice or version rejects without removing a card or
changing public knowledge. Pool removal increments its owner's version; future
pool mutations must do the same. The plan has no in-game ObjectRef and the
current normal path commits it immediately. This is not a replacement-event
scheduler or evidence that outside-to-hand replacements and observers work.
Extrapolate remains gated pending those rules, validation and source provenance.

The isolated scheduler exposes a public competing-provider choice and rechecks
live battlefield providers after each replacement. Synthetic providers exercise
prevention, redirect, repeat selection and snapshot restoration; no printed
outside-entry providers have been identified. The scheduler now stages typed
resolving-effect self-replacements before battlefield providers, but generic
outside-entry observers remain incomplete.

## Isolated scheduler limitations

The experimental outside-entry scheduler carries a physical choice identity,
not an in-game origin. Its provider lookup currently visits battlefield static
sources only. An incoming card's ordinary battlefield ability must not be
assumed active merely because it is selected from outside the game: CR 113.6
and 400.11c govern when abilities function there. Specially worded effects
that function outside the game need explicit source-backed modeling; none have
been identified for this event. Do not blanket-scan all abilities printed on
the incoming card or manufacture an ObjectRef for it.

Separately, CR 614.15 and 616.1a define a self-replacement as an effect of the
resolving spell or ability that replaces its own effect, and require it to be
applied before ordinary competing replacements. The isolated scheduler owns
instruction-supplied self-replacement templates and their resolving stack-item
identity, choices and applied indices. It re-evaluates applicability after each
transformation and retains those choices across snapshots. This foundation is
limited to destination changes and prevention; no printed admission instruction
requiring it has been identified, and its synthetic tests remain unvalidated.
Do not claim complete replacement support.

Rules source: [Magic Comprehensive Rules, June 19, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260619.pdf).

The draft records a typed completed outside admission without an invented
source zone or object. Only successful admission has a new object identity;
prevention leaves the card outside and does not set exile history. A card
actually put into exile sets the existing turn history, matching the condition
that a card was put into exile. CR 400.11 and 603.6 do not justify fabricating
a zone-change event from `Zone::Outside`. Whether a generic “whenever a card
is exiled” observer sees this hypothetical replacement remains unresolved; no
such observer is dispatched by the draft. Existing return-to-hand and
zone-change observers do not constitute that event model.
The experimental schema 192 guard conservatively rejects all states retaining
outside-game ownership, entry batches and active zone-rule checkpoints when
projecting to 191, including cast rollback checkpoints; passive zone rules remain
representable. This remains broader than necessary until future behavior is proven.
