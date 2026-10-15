# Changeling and shared subtype matching

`Card::has_subtype` now centralizes intrinsic subtype membership. Changeling adds
every creature type in every zone, including outside the game; it does not add
land, artifact, enchantment or other unrelated subtypes. Printed subtypes remain
valid independently of changeling. Entry/death observers, continuous creature
filters, captured damage-dealer filters, sacrifice choices, library searches and
Vehicle checks all use the same predicate. Dynamic type-changing and
ability-changing effects remain gated by complete-card admission.

The immutable creature-type list is pinned in data/creature-types.json, with the
source rules version, PDF SHA-256 and apostrophe normalization. It contains the
324 creature types from CR205.3m in the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
CR702.73a defines the all-zones ability. Time Lord is a single two-word subtype;
Time and Lord are not separate types. Hyphenated types and canonical Oracle
apostrophes remain intact. The pinned Java checkout's subtype enum contains
25 additional entries absent from these rules; they are excluded from this
snapshot. Current Standard creature-type text was checked against the taxonomy;
noncreature subtypes on artifact/enchantment creatures remain separate.

The native library exports CREATURE_TYPES and is_creature_type. Lookup uses the
sorted static list; normal cards retain direct printed-subtype matching. Public
Keywords::CHANGELING uses bit 524288, within the existing u32 mask. All type facts
are immutable; no expanded subtype sets or new mutable caches enter snapshots.
The taxonomy bytes join card facts, target data and compiler code in the
catalogue fingerprint. Observation schema stays 32. Intrinsic changeling parsing
is separate from temporary keyword grants, which do not support changeling yet.

Shared exact program vocabulary also covers optional basic searches, optional
artifact/enchantment removal, optional other-owned-creature returns and up to two
graveyard targets. Declining a search does not shuffle. Accepting it opens the
existing private search; completing even a failed accepted search shuffles.
Choosing no optional target preserves independent effects; losing every chosen
target still causes the normal whole-effect fizzle. Graveshifter's optional
return still requires a legal target when its trigger is announced.

Twelve complete Standard identities are added:

| Card | Complete admitted semantics |
| --- | --- |
| Changeling Wayfinder | Changeling; optional basic-land search to hand |
| Chitinous Graspling | Changeling and reach; hybrid casting |
| Chomping Changeling | Changeling; optional artifact/enchantment destruction on entry |
| Curious Farm Animals | Death life gain; self-sacrifice activation for optional artifact/enchantment destruction |
| Environmental Scientist | Optional basic-land search to hand on entry |
| Flock Impostor | Changeling, flash/flying; optional return of another owned creature |
| Gangly Stompling | Changeling and trample; hybrid casting |
| Graveshifter | Changeling; targeted optional creature recovery to hand |
| Mischievous Sneakling | Changeling and flash; hybrid casting |
| Prideful Feastling | Changeling and lifelink; hybrid casting |
| Rooftop Percher | Changeling/flying; optional graveyard exile followed by life gain |
| Three Tree Mascot | Changeling; once-per-turn one-mana color conversion |

Coverage is 846/5,164 Standard identities, six regression definitions and 40 token
factories; no former admissions were removed. Feisty Spikeling's turn-dependent
first strike, Nameless Inversion's type removal, Barkform Harvester's library
placement, Taurean Mauler's optional counter trigger, Firdoch Core's animation,
Crib Swap's controller-bound token, Sizzling Changeling's exile permission,
Mutable Explorer's animated land token and Omni-Changeling's convoke/copy effects
remain unsupported. No partial changeling card is admitted.

Validation: 486 Rust tests (478 unit plus eight integration), 86 compiler tests,
53 rebuilt release Python tests, consistency, rustfmt and Clippy with warnings
denied. Fifteen focused engine fixtures and two type-data fixtures cover all-zone
membership, exact unrelated subtypes, two-word parsing, lords, simultaneous death
LKI, multiple subtype entries, captured damage facts, optional targets/searches,
private choices and owned replay. Subtype-search and subtype-sacrifice programs
are explicitly synthetic continuations; captured-damage fixtures dispatch frozen
event facts. They do not add any printed card admission.

One hundred supported Standard games completed naturally: 58,553 decisions,
704 observed changeling entries, 68 Wayfinder optional-search choices, 405 new
visible subtype triggers and 8,199 cloned/replayed boundaries. Maximum initialized
state was 12,904 bytes, excluding allocator metadata, spare capacity, external
snapshots and static definitions. This evidence covers supported decks, not full
Standard conformance or an XMage speed comparison. Remaining card semantics and
native RL/search integration remain outstanding.
