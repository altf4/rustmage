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
