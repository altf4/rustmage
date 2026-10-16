# Controlled-subtype activation restrictions

The native library now checks immutable `ActivationGuard` definitions when an
ability starts. `ControlSubtypeAny` requires at least one owned battlefield
permanent with one of the listed intrinsic subtypes. It shares `Card::has_subtype`
with the rest of the engine: changeling qualifies as a Dragon, while it does not
add basic land types. Tapped or summoning-sick prerequisite permanents still
qualify. Other players' permanents and cards in other zones do not.

The ten Verge lands retain their unconditional primary mana ability and unlock
their second ability only with the required land type. A typed nonbasic land
qualifies; a matching basic supertype is not required. Mox Jasper requires a
Dragon and produces one of the five colors, never colorless. These are mana
abilities and use no stack. Standard deck admission still enforces the ordinary
four-copy limit, including the zero-mana Mox.

| Card | Primary mana | Conditional mana | Prerequisite: at least one |
| --- | --- | --- | --- |
| Blazemire Verge | B | R | Swamp or Mountain |
| Bleachbone Verge | B | W | Plains or Swamp |
| Floodfarm Verge | W | U | Plains or Island |
| Gloomlake Verge | U | B | Island or Swamp |
| Hushwood Verge | G | W | Forest or Plains |
| Riverpyre Verge | R | U | Island or Mountain |
| Sunbillow Verge | W | R | Mountain or Plains |
| Thornspire Verge | R | G | Mountain or Forest |
| Wastewood Verge | G | B | Swamp or Forest |
| Willowrush Verge | U | G | Forest or Island |
| Mox Jasper | — | W, U, B, R or G | Dragon |

The compact mana graph retains relevant owned subtype witnesses even when they
produce no mana themselves. It recomputes prerequisites after each virtual move,
including sacrifices, and filters both real moves and its optimistic bound with
the shared predicate. The admitted pure mana steps can only remove these subtype
facts, so an initially absent guard cannot become true during the payment plan.
Card definition, ownership and presence already distinguish the canonical keys;
no new mutable game field, keyword cache or observation schema is needed. Schema
remains 33. Witness collection allocates no subtype vector storage on unguarded
boards, and unneeded nonmana witnesses are excluded from compact resources.

A starting restriction is distinct from an unpaid cost. Restrictions are checked
before an announcement's mana window. Once an ability has legally started,
losing a prerequisite while paying does not undo that activation or create a new
resolution condition. The compact enclosing source obligations therefore check
source costs without rechecking the guard. Each additional mana ability started
inside the window still checks its own guard against the current board. This
follows CR602.1b, 602.2b/601.2e–h and 602.5 in the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
The source-obligation and synthetic nonmana-definition fixtures distinguish this
rule from complete new printed-card admissions.

The exact compiler grammar recognizes one named controlled subtype or either of
two named subtypes, from the existing singular tribe vocabulary and five basic
land types. It preserves every residual card clause and the existing timing and
once-per-turn checks. Broader counts, tapped prerequisites, conjunctions, derived
power/toughness conditions, different controllers and unrecognized restrictions
remain gated. A guard is not an exhaustible resource: unbounded free mana loops
remain rejected. No partial Baxter Building, Bloom Tender, Springleaf Drum or
other unsupported card is admitted.

Validation: 518 Rust tests (510 unit plus eight integration), 92 compiler tests,
59 rebuilt release Python tests, consistency, rustfmt and Clippy with warnings
denied. Runtime fixtures cover all ten land cycles against all five basic types,
both controllers and several zones, exact mana colors, typed nonbasics,
changeling/token witnesses, forged actions, paid spell announcements and owned
clone/restore. Small-board native mana-action enumeration independently checks
compact reachability across guarded/unavailable/opposing prerequisite cases and
multiple costs; private graph fixtures check virtual witness removal and keys.

`standard_activation_guards` uses all ten Verges and Mox Jasper in two legal
supported 60-card Standard decks. One hundred games completed naturally with
49,525 decisions, 2,342 direct secondary Verge mana activations, 406 direct Mox
activations and 7,720 cloned/replayed boundaries. Maximum initialized state was
12,628 bytes, excluding allocator metadata, spare capacity, external snapshots
and immutable definitions. These smoke-workload counts are not a Java speed
comparison or proof of full Standard conformance.

Eleven complete definitions advance coverage to 889/5,164 Standard identities,
with six regression cards and 41 token factories. General rules/layers, remaining
card semantics, packed/batched RL encoding, information-set MCTS and native
training integration remain incomplete.
