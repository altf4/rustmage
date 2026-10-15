# Qualified library searches and fetches

Search is an immutable instruction with a fixed card-quality filter, maximum,
destination, tapped flag and reveal flag. Mutable Resolution owns only the selected
card IDs; the library itself is not duplicated. ChooseSearch adds one legal card,
and FinishSearch commits the selection. Even at the maximum, commit is explicit.
Every offered choice belongs to the searching player's library, matches the
filter and is distinct. Selections remain in the library and private until commit.

Qualified hidden-zone searches allow finding fewer cards, including zero, even
when matches exist. FinishSearch still shuffles the remaining library through the
game-owned RNG. Empty/no-match searches do not attempt a draw or lose the game.
Unqualified mandatory searches, searches of opponents' zones, mixed destinations,
different-name constraints and optional search restrictions remain outstanding.

All selected cards move in a completed simultaneous batch. Battlefield arrivals
apply printed entry conditions and the effect's tapped requirement before any
arrival moves. Entry triggers see the completed batch but wait until the current
resolution finishes. The library then shuffles before later instructions execute.
Self-sacrifice costs are paid before priority; a Lander token can cease and its
slot can be reused without changing the pending search's captured source.

Schema **14** adds private search_choice (filter, maximum, destination, flags,
library cards and selections) and public known_hand incarnation references. A
non-searching player sees neither the library nor partial selections. Searches
that reveal cards into a hand retain those identities as public knowledge in
objects and known_hand until that incarnation leaves the hand. Unrevealed hand
cards stay private. A later zone change removes the knowledge record; returning
the physical card to hand as a new incarnation does not reveal it again.
Future hidden hand randomization must invalidate relevant knowledge explicitly.
This models knowledge of a reveal, rather than an ongoing reveal effect.

Snapshots/clones own selections, known-hand records, RNG and continuations.
state_bytes includes initialized selection and public-knowledge storage. No
callbacks, host maps or transport session state participate in rules restoration.
Scry/surveil retain their separate private ordering choices.

Thirteen complete Standard definitions bring coverage to 512/5,164, with six
legacy regression cards and 30 token factories: Beamsaw Prospector, Burnished
Hart, Dauntless Scrapbot, Evolving Wilds, Expedition Map, Galactic Wayfarer,
Promising Vein, Sami's Curiosity, Shared Roots, Terramorphic Expanse, They Went
This Way, Vibrant Cityscape and World Map. Lander has its full predefined ability.
Residual clauses, wider search qualities, landcycling and other unimplemented
mechanics stay gated. Full Standard/RL/search acceptance remains outstanding.

Native tests cover illegal zone/owner/quality/duplicate/excess choices, partial
commit privacy, fail-to-find, empty libraries, exact shuffle and subsequent RNG,
multiple arrivals, printed entry conditions, Wastes versus typed nonbasics,
public reveals and fresh incarnations, Lander reuse, continuation order,
trigger deferral and clone/restore. Python covers both fetch-to-battlefield and
reveal-to-hand through the C ABI. standard_search exercises supported Standard
60-card decks with fetch lands, ramp, landfall and Lander tokens.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 701.20, 701.23, 701.24, 608, 603 and 614.12.
