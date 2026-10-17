# Attachment-granted Ward and typed Aura legality

Issue257 extends the existing exact-stack Ward payment kernel to ordinary Aura
and Equipment grants. Each printed or granted Ward instance remains a separate
trigger owned by the recipient's controller. Already queued packets survive
removing the attachment. Grants use the existing immutable programs and indexed
attachment observers; Ward is not a keyword bit.

`AuraSpec.kinds` is an immutable enchant kind predicate, evaluated using current
object characteristics alongside the existing controller relation. All prior
Aura helper rows gain only `kinds: 1`, preserving their exact creature predicate;
Hardlight Containment uses artifact mask16. Casting targets and existing entry,
attachment and state-based checks retain their appropriate targeting distinctions.
The ability recipient lookup now accepts a legal noncreature enchanted artifact;
Equipment grants still require a creature recipient and noncreature Equipment.

The complete initial admissions are Dusk Rose Reliquary,Trapped in the Screen,
Dwarven Mattock,Falcon's Wing Harness,Lavaspur Boots,Hardlight Containment,
Sheltered by Ghosts and Super Strength. The shared linked-exile target union also
admits two complete non-Ward neighbors: Detention Chariot (Crew3,cyclingW,6/6)
and Static Snare (flash,discount per attacking creature). Their constructors were
read in full; acceptance must cover these residuals as well as linked exile.
All ten names are grammar-derived, with no name-specific admission override.

Local schema77,1770 Standard/1776total,84 token recipes. Catalogue fingerprint
`4a58441aad0d01f9c058ee473c7907023fcfe0430e70ab7b04e83ecfa30684b2`.
Exactly10 complete definitions change against main d3f8;5245 definitions are
byte-identical. Helper changes also include the narrow default Aura representation
and real new additional costs/equip/crew/cycling/reduction data; no claim of wholly
byte-identical helper tables is made.

`attachment-ward-admissions.json` pins all10 full constructors plus initial shared
helpers to Java492369c7844b8d76479f0393ff9eb6b0b0cf30d7. `ward-inventory.json`
preserves the149-card whole Standard Ward-text inventory including all faces;
29 were supported on baseline,120 gated. Eight Ward additions leave112 baseline
Ward-related gated cards; two extra admissions do not themselves mention Ward.
Future mixed/dynamic/OR costs,Role tokens,global grants,disguise/cloak,Prepared
and Ward suppression remain explicit whole-body backlog. Existing mana,life,
discard,creature sacrifice,poison and evidence Ward payment remains reused.

Checkpoint evidence: all1612 preexisting unit tests passed on initial core;
8 new native groups passed on the full ten-admission core;4 focused compiler
methods passed with complete-body/cost/layout/unknown-residual negatives.
Full paid public-library,Standard and broad final acceptance are still pending.
