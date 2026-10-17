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
`d676d3a798a26663fc0c0206d7ea5c8d2e359b5310c78a12bf176dd73ee80fca`.
Exactly10 complete definitions change against main d3f8;5258 definitions are
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


The acceptance pilot found Static Snare initially reused the controller-only
attacker count, incorrectly charging5 during an opponent's attack. A distinct
`AllAttackingCreatures` earlier-layer value now implements its unqualified
cost reduction while all existing own-attacker consumers retain their previous
value. Both-seat native actual one/two-attacker payments validate4/3 costs;
real paid public regression and the Standard driver assert4 with one enemy
attacker. Cost locking continues through the existing announcement kernel.
Artifact-only Auras remain closed against static keyword/creature-conditional
grants until their complete noncreature keyword routing is implemented.

The corrected core passes1623 unit tests plus12 integration tests(all targets),
including11 new native groups. Broad compiler baseline had337passing methods
and one stale additional-cost-count fixture(10→11 due Reliquary); corrected
fixture plus final five-method focused module passes6 methods in3.070s.
Final public/Standard and fresh broad acceptance remain in progress.


Final shared grantor check retains the original noncreature-source requirement
for all attachments: a creature Aura cannot legally enchant(303.4d), and a
creature Equipment cannot remain attached. This removes an unnecessary Aura
exception from the initial checkpoint without changing any admitted reachable
route. All56 real paid public cases pass on the corrected5495 core(83.805s);
final fresh full-suite acceptance follows the final one-line guard restoration.
Selected additional helper provenance includes linked exile,attachment,
sacrifice costs,Ward counter-unless-pay and Static Snare's all-player count.
