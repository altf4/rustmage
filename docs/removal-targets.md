# Live removal targets

Issue: https://github.com/altf4/rustmage/issues/90

This stage admits 45 whole Standard definitions, bringing executable coverage to
967/5,164 (4,197 unsupported), six regression definitions and 43 token factories.
[The admission record](removal-target-admissions.json) lists every printed body,
Java source path and SHA-256 at pinned XMage revision
492369c7844b8d76479f0393ff9eb6b0b0cf30d7. Pinned Oracle facts remain authoritative;
the Java implementation is a reference, not a runtime dependency.

Immutable `TargetRule::Union` and `Filtered` compose existing target rules with
live predicates. Qualifiers apply to their branch: Broken Wings can target any
artifact/enchantment, or a creature with flying; an artifact need not have flying.
All branches retain zone, controller, hexproof and bound-generation checks. The
same matching code is used at announcement and resolution, so untapping a tapped
target, lowering its power or losing flying can make the target illegal. If all
chosen targets are illegal, the entire resolution (including life/scry tails)
fizzles; indestructible instead prevents destruction of a still-legal target and
does not suppress that tail.

New predicates cover attacking/blocking, power/toughness and either-stat bounds,
color unions, token status, intrinsic subtypes/non-outlaws, nonbasic land and
this-turn damage history. Blocking uses the persistent blocking set, even if its
attacker left combat. Outlaws are Assassins, Mercenaries, Pirates, Rogues and
Warlocks; intrinsic changeling qualifies. Other subtype/ability/type/color changes
remain gated until their continuous-effect machinery is implemented.

Two owned per-object flags distinguish **dealt** damage from **was dealt** damage.
Only positive applied damage sets them. A captured departed source does not mark
its new generation. Cleanup damage removal preserves both flags during any extra
priority cycle; the next turn, zone transitions and ceased tokens reset them.
This distinction is required for Rooftop/Stingblade Assassin versus Red Guardian.
A regression creates a cleanup state-based death, then casts Rooftop Assassin
while its legal victim has zero marked damage but a true history flag. Snapshots,
clones and transaction rollback own these flags. Observation schema **36** exposes
`damaged_this_turn` and `dealt_damage_this_turn` on visible objects; ABI 1 and
optional control-response schema 2 remain unchanged.

`Card::colors()` reads an immutable byte per definition (W/U/B/R/G bits), including
both hybrid halves and token colors. It uses pinned colors, not color identity,
the selected hybrid payment, or one alternative mana cost. Unsupported multiple-
face identities expose front-face color facts only. Missing, malformed or
duplicate color facts fail compilation even if the input checksum is refreshed.
A native test checks every pinned identity and token factory against these facts.

The compiler only accepts exact complete target clauses with complete existing
programs/tails. This unlocks destroy/exile spells, all modes of six modal spells,
cycling, mandatory sacrifice costs, targeted entry/attack triggers, Equipment and
sacrifice activations. New Knight and black/green Elf token factories preserve
printed token stats/colors. Cards with extra unsupported effects (manifest dread,
controller-created tokens, conditional tails, cost reductions, convoke, twobrid
or uncounterability) remain rejected. This is not a fallback that discards text.

Author validation passes 584 Rust tests (575 unit plus nine integration), 101
compiler/data tests, 69 release Python tests, catalogue consistency, rustfmt and
Clippy with warnings denied. The focused native tests execute each newly admitted
body, every modal mode, both cycling abilities, extra costs, all activation paths,
entry/attack targets, target revalidation, privacy, numeric bounds, and branch
replay. Future-type Vehicle/Spacecraft/planeswalker union branches use synthetic
objects where no full supported definition exists; these fixtures do not admit
those unsupported cards. Public Python fixtures use legal play to remove a white
hybrid, destroy a real combat-damaged creature, and distinguish a creature that
damaged a player from a damaged recipient, checking native clone/restore/staleness.

`standard_removal_targets` runs supported 60-card Standard decks and independently
checks public target predicates and removal outcomes. One hundred games complete
naturally: 119,824 decisions, 2,605 offered-target checks, 1,002 checked removal
outcomes, 23,917 cloned/replayed boundaries, 18,408 maximum initialized state bytes.
This mixed workload exercises a subset of the admissions; focused fixtures cover
the others. The byte count excludes allocator/reference-count metadata, spare
capacity, immutable definitions and external checkpoints. It is not a comparable
benchmark against the previous deck workload, a Java speed claim, or full Standard
acceptance. Full format semantics and broader RL/search integration remain open.
