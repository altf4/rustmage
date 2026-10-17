# Shared Gift protocol — issue #310

PR #312 adds seventeen complete Gift consumers to actual main
`8dd395286661963d1d1d98a2d384f2f5ab62d620`. The composed catalogue implements
1,931 of 5,164 pinned Standard identities; 3,233 remain gated. Observation schema
94 distinguishes this delivery from the original schema-90 Gift checkpoint.
All 94 token recipes and the emblem remain intact. This completes the implemented
Gift family below, rather than the full Standard migration.

## Complete bodies and rules

The complete consumers are Bilbo's Gambit, Blooming Blast, Consumed by Greed,
Crumb and Get It, Dawn's Truce, Dewdrop Cure, Into the Flood Maw, Long River's Pull,
Longstalk Brawl, Mind Spiral, Nocturnal Hunger, Peerless Recycling, Sazacap's Brew,
Scrapshooter, Valley Rally, Wear Down, and Wildfire Howl.

Casting exposes a public, optional, non-targeted opponent beneficiary before
any targets or payments. Promised and declined declarations select separate
complete target plans, including promise-only target groups and changed target
cardinalities. Initial availability checks either fully fundable declaration;
subsequent choices and payment enforce the selected declaration. Sazacap's Brew's
mandatory caster discard remains an additional casting cost.

A real resolving instant or sorcery gives its promised card, Food, Treasure, or
tapped 1/1 blue Fish before its other instructions. A countered spell or one whose
entire target set is illegal gives no gift. Partial target legality preserves the
central resolution rule. Token replacements suspend and resume gift delivery
exactly once under the beneficiary's control. Permanent Gift entry and printed
entry abilities are separate packets, retaining the cast's exact source,
beneficiary, controller and actual public entry incarnation. Uncast entry or a
later incarnation cannot inherit that promise; departing sources retain bound
controller and LKI context.

Spell returns route the physical card to its owner's hand, including Adventure
and Omen faces, without resolving it or creating resolution-only permissions.
Dawn's Truce applies player hexproof and a snapshot of the caster's current
permanents. Bilbo's Gambit's casting prohibition covers all casting origins,
methods and faces while allowing otherwise legal abilities, Plot and land plays.
Both temporary effects expire in cleanup. Sparse owned contexts participate in
clone, snapshot, restore and owned heap accounting.

Generic full-body lowering authenticates original raw text, including the exact
Gift reminder appropriate to its gift kind. Printed names and the audit inventory
never grant admission. Unknown parentheses, borrowed reminders, executable tails,
malformed costs, unsupported layouts and incomplete bodies remain rejected.

## Pinned source and preservation

`gift-admissions.json` records all 23 complete physical metadata records and pins
full Java implementations, resolvable direct imports, wildcard-package sources,
and shared Gift protocol files at Java revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`: 532 records, 198 distinct files.
It is audit evidence, not compiler permission or a claim of transitive Java closure.

`gift-main-preservation.json` proves actual main `8dd3952` changes exactly these
seventeen ordinary bodies, preserving 5,314 other definitions, every one of the
888 parent helper arms, aliases, colors, flags, token recipes and emblem. The
seventeen programs and eighteen new helper arms match the earlier tested Gift
checkpoint. `gift-original-preservation.json` retains the original strict proof
against `5d103e7`. Parent death-exile and combat proofs retain their historical
strict unions and separately compare their delivered programs to current ones;
their allowed change sets are not expanded for Gift.

## Validation and exact scope

The composed execution source is `e484dbcb787f57c24a10a309c0ad35f84f10fd85`.
All 866 tracked inputs were frozen for native/compiler checks and copied into an
immutable export for public acceptance. Production, generator, data and policy
bytes equal `721fb951560e5326a4b75d2d9e7600fa12897bec`; their only difference is a
new native fixture's expected promised Treasure. The optimized library remains:

`<cache>/rustmage-immutable/gift312-721fb95-2bff-schema94-87d27d0258d2.so`

SHA-256: `87d27d0258d2f2bd811d206e54a699472a51567941445927542b5888f452a716`.
Catalogue: `2bff2197796a4c74703b5736b161a099dd1fd88cb385fb45fde0c78f0dca462c`.

- Fresh composed native: 1,836 unit and 12 integration tests pass; all-target
  Clippy, formatting and generator verification pass.
- Fresh composed compiler: 50 affected methods pass without skips, including all
  Gift, all seven parent death-exile controls, historical/current preservation,
  parent Role/temporary-death/intrinsic/raw guards and three corrected catalogue
  expectations. Runtime: 23.283 seconds.
- Fresh composed public: five unique methods pass without skips in 210.011
  seconds: 68 paid Gift cases, 30 parent death-exile/mode cases, 24 inherited
  Role cases and four rejected inset deck aliases.
- Fresh composed Standard100: all 100 games finish naturally with all 68 Gift
  vectors, 74,676 owned replay frames and 149,452 denials (155.576 seconds).

Paid native cases exercise all seventeen bodies in both seats with promise and
decline, foreign ownership, every gift kind, announcement legality, real
counter/fizzle/partial-target responses, mandatory discard, token replacement,
  protection snapshots, casting prohibition and cleanup. The added composed
  witness pays Fake Your Own Death and nonlethal Pinecone Strike on a foreign-owned
  creature, then promised Blooming Blast: Treasure is delivered, damage follows
  its controller, the marked creature goes to exile, and no death-return/Treasure
  tail occurs. Every action checks owned clones after parent drop, restore,
  both observations and rejected wrong-actor/stale revisions.

Historical breadth remains scoped to the original `28d8b9d` source on `5d103e7`,
using schema 90 and immutable `gift312-eb55a2f-1997-schema90-fad2ff7751f3.so`:
277 unique public methods pass without skips (1,849.476 seconds), all 100 Standard
workloads finish naturally (74,676 owned frames, 149,452 denials), and the full
480-method compiler run has **477 passes and three stale-fixture failures**.
Those failures expected newly admitted Mouser Mark III/Technodrome to be gated,
and expected eleven additional-cost helpers before Sazacap added the twelfth.
After all live jobs ended, exactly those expectations were corrected; the exact
three methods plus all four Gift methods passed, then the composed 50-method gate
passed. The old full compiler run is not reported as green. Earlier native
`721fb95` passed 1,835 unit tests and failed only the new fixture's mistaken card
gift expectation; the test-only correction is covered by the fresh native suite.
Prototype fixture failures and the explicitly canceled incorrect-Git-context
compiler harness are not acceptance evidence.

Independent reviews cover the whole seventeen-body batch, generic raw boundary
controls, exact canonical preservation and novel paid interactions. Their final
exact-head confirmations are required before merge. Completed build outputs are
cleaned immediately, retaining every shared library at its original path and hash.

## Remaining complete bodies

Six pinned consumers remain gated on explicit missing semantics:

- Coiling Rebirth: copy-except characteristics, issue #314.
- Cruelclaw's Heist: lasting foreign exile-cast and any-mana permission, issue #315.
- Kitnap: changing-control layers, issue #265.
- Parting Gust: exact delayed exile-return with entry counter, issue #316.
- Starfall Invocation: selection from actual simultaneous destruction results,
  issue #317.
- Starforged Sword: ordered flying-removal layers, issue #318.

No partial version of those bodies is admitted. Full Standard remains unfinished.
