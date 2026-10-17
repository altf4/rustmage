# Ordered ability removal
## Delivered core and complete definitions

Four complete cards are directly authored in paired Rust/provenance files:
Fresh Start, Frozen in Ice, Stop Cold, and Enchanted River's Grasp. The current
composition uses observation schema 106 from the shared schema file and retains
all previously implemented behavior. Coverage is 1,987 of 5,164 Standard identities;
the remaining twenty inventoried ability-loss bodies retain their explicit
dependencies. This work does not complete the overall migration.

The shared layer-six evaluator orders loss and gains by dependency and timestamps,
retains effects that began in earlier layers, and preserves captured queued
abilities. It consults a borrowed actual or mana-overlay world rather than storing
mutable derived characteristics in each object. Sparse source/effect chronology
is owned across clone, restore and branch operations. External blocking rules are
separate from removable granted abilities. Counter reset and untap instructions
use exact attached objects; prohibitions apply before Stun counters are consumed.
Both entry replacement paths test the effective provider ability. The existing
controlled-untapped helper receives a typed source-local identity without English
parsing, card-name dispatch or physical-ID cases. The direct registry migration
must preserve this getter supplement as well as the original origin table.

Corrected core validation: 1,939 unit and 12 integration tests pass, followed by
all-target Clippy with warnings denied. Seven actual paid groups run in both seats
with wrong-actor and stale-action denials, raw/index equivalence, owned parent-drop
branches, restore/replay and both-view checks. These cover all four printed bodies,
all supported counter kinds, Stun-preserving prohibited untaps, effect versus
controller-step untaps, lost mana abilities, Equipment earlier-layer continuation,
external blocking rules, old and later grants, multipart Archon and later-layer
Maha suppression/restoration, queued Alchemist programs, and actual Minstrel
entry-tap suppression followed by Disenchant and natural-turn restoration.

After composing the shared-schema parent, four focused Python provenance/wire
checks and one native wire check pass. A fresh optimized library builds and two
release wire checks pass. The only production change from the full-tested core
is selecting the same schema value through the parent's shared include parser;
the source and policies remain frozen for every consuming run.

Historical failures are retained rather than relabeled: the first authored build
had Rust string/include encoding errors; the next three new tests exposed omitted
Aura spell target plans in the direct definitions. A later test wrongly expected
an Alchemist counter instead of its printed temporary +2/+0 effect. All were
corrected before the passing full core run. An eight-case natural-game diagnostic
was explicitly terminated when the missing controlled-entry provider gate was
found, then the gate was fixed and the real paid regression added. No passing
natural-game workload is claimed; the unaccepted diagnostic policy is preserved
as evidence and omitted from default test discovery. Repeated evaluation during observations needs
profiling and reuse of a coherent ephemeral world (issue #354).

## Historical architecture checkpoints

Issue [337](https://github.com/altf4/rustmage/issues/337). Foundation `61a7d0cef4b10591958cea4391c553db4c2c07ff`; schema102 reserved. This is a source/architecture checkpoint, **no new card admitted and no runtime changes**. Final implementation must compose live-static100 and hand/grave101 before freezing.

`ability-removal-admissions.json` inventories all24 complete physical cards containing lose/loses all abilities, with410 pinned Java root/explicit single-class-import records across114 unique files at `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Imports are evidence, not a transitive closure claim or compiler whitelist. Every complete printed body and dependency remains tracked. Four initial candidates are Fresh Start, Frozen in Ice, Stop Cold, and Enchanted River's Grasp. Others require further characteristic, duration, counter, targeting or cost semantics; they remain gated until their entire body is implemented.

## Existing engine boundaries

- `engine_equipment.rs::keywords` currently unions printed, temporary, attachment, suspect, counter-provider, and global grants. `Object.gained_keywords` loses effect order. A boolean suppression test after that union would incorrectly remove later grants; a boolean before it would incorrectly retain earlier grants.
- `engine_stats.rs::register_battlefield_sources` maintains immutable candidate indexes, not effective ability indexes. This is useful: removal should not physically delete these registrations because an Aura can leave and restore printed abilities without reentry. Every consumer must test the effective source ability.
- Trigger routes are distributed across cast/attack/draw/discard/life/damage/explore/evidence/forage/death/step observers. Temporary-granted death packets use exact source provenance but currently read grants unconditionally. Suppress only abilities unavailable at event time; already queued `SpellView` programs survive source removal or ability loss.
- `ObjectView` captures predeparture keywords/stats, but death lookback also needs effective non-keyword ability identities. `finish_zone_events` must never re-query only the post-event battlefield. Simultaneous source/recipient departures share one frozen pre-event characteristic evaluation.
- `engine_activations.rs` and `engine_funding.rs` both read static activation definitions. Funding simulates mana-resource sacrifice and cannot merely read a suppression bit from the original real Game: a consumed removal Aura or provider can change availability. Real and virtual legality need the same characteristic evaluator against their respective resource states.
- Replacement/counter/token/entry/timing/cost/untap/static-stat queries read immutable card helpers through many independent paths. Printed characteristics like physical mana value are not abilities; distinguish them from the static ability implementing a CDA or continuous provider.
- `Object::untap` is invoked by turn advancement and multiple instruction routes. Frozen in Ice forbids all attempts; Stop Cold/River forbid only controller untap-step actions. A centralized Game-level attempt API must distinguish cause, reject forbidden attempts before consuming Stun, and preserve existing Stun replacement behavior.
- Current counter kinds are Lore, PlusOne, MinusOne, Stun, Loyalty. River removes every supported kind atomically; this does not require adding keyword counters. Abigale remains gated until actual keyword-counter kinds and per-kind timestamp refresh exist.

## Proposed characteristic stage

1. Add sparse owned layer6 effect records under existing optional continuous-effects state, using exact recipient/source incarnation, immutable granting definition/ability identity, timestamp and duration. No unconditional vector or flags on each Object. Reuse the monotonic source/effect clock introduced by base stats/live statics, extended to effect creation and later grants.
2. Expose a single evaluated ability-set interface: printed/continuous/granted ability identities plus keyword mask. Printed ability identity must distinguish a whole static ability from its separate layer fragments. Keep candidate registries immutable and let queries request presence of the relevant identity.
3. Evaluate layer6 additions/removals in dependency order, timestamp for independent effects and dependency cycles. Earlier-layer participation must be recorded: CR613.6 effects already begun continue later fragments even if their generating ability disappears in layer6. In particular, removing Changeling in layer6 does not retroactively erase creature types established in layer4. Same-layer ability providers removed before contributing must not continue granting through an unconditional static index.
4. Evaluate layer7 from effective layer6 abilities, preserving earlier-started multi-layer fragments. Maha/Aettir start at7b. Archon is one effect spanning6 (flying) and7b (base): its dependency on removal of the provider must resolve before its layer6 fragment starts; both fragments disappear when that dependency removes the ability. Give every continuous effect explicit shared identity rather than grouping all printed abilities on a card or blanket-gating every7b fragment with the final source flag. Ability loss does not erase earlier resolved continuous effects, keyword counters, permanent counters, creature types, colors, mana value, attachment or owner/controller facts.
5. Capture effective ability identities in pre-event LKI for zone-change observer dispatch and queued source context; do not add a second full dynamic ability vector to every SpellView. Own all sparse context so parent-drop/restore/replay and both observers remain deterministic.
6. Keep a fast path when no ability-removal effects are active. Existing timestamp data must already retain chronology of grants that predate the first removal, so activating the slow path later cannot invent their order. Prototype and measure this representation before committing the public contract.

## Rules evidence and difficult cases

Pinned primary CR2026-09-25 local `<temp>/rustmage-rules-20260925.txt`:613.1f/613.3 (layer6),613.6 (multi-layer continuation),613.7a-e/m/n (timestamps),613.8a-c (dependencies),122.1b (keyword counters),603.10a (pre-event lookback),113.7a (queued abilities independent of source). Read the complete relevant text rather than treating the Java implementation as the rules authority.

A single 'last loss timestamp' is insufficient if source ability removal changes the existence of another layer6 provider. Likewise, clearing only Keywords does not disable mana abilities, Ward, death grants, replacements, cost reductions, continuous statistics or combat restrictions. An ad-hoc source boolean without earlier-layer participation is insufficient for Changeling/type-changing providers. These are architecture constraints to resolve, not reasons to narrow individual admitted card text.

## Required acceptance

Both seats, genuinely paid normal bodies, foreign owner/current controller, actual Aura source departure/reentry, earlier and later keyword/trigger grants, same-kind counter timestamps when implemented, Ward already on stack, suppress future Ward and death triggers, simultaneous lookback, temporary owner-return grants, live Maha/Archon/Aettir, mana/funding sacrifice changes, vehicle/land/artifact recipients, every untap cause/Stun interaction, all-counter removal, layer7 growth/counters retained, exact owned clone/rollback/private choices/illegal actions and logical byte accounting. Strict generic raw renamed/infix/nested/copy-reminder controls and full parent preservation. Native/Clippy/fmt/generator, meaningful focused compiler/public vectors, fixed100 unretried Standard games, immutable complete source freeze, then two NONAUTHOR reviews.

## Identity migration prerequisite

Current catalogue helpers have immutable executable fragments but do not retain which original static ability supplied each fragment. Grouping all helpers from one physical Card into one ability would be incorrect: a source with an earlier-layer effect and a separate later-layer lord must not keep the lord merely because the former already started. Conversely, giving Archon Flying and its4/4 setter unrelated identities would break their single-effect continuation. The compiler migration must retain shared static-effect identity and first applicable layer through extraction; existing helper lookup indexes can remain optimized but carry the identity needed by evaluation. No new caller may approximate this with a card-wide erased flag.

Other queries to classify explicitly: basic-land intrinsic mana abilities versus retained subtype facts; Auras' Enchant restrictions (attachment legality/SBAs); Equip as an activated ability versus the attachment relation; continuous combat restrictions; CDA base statistics versus printed base numbers; ETB replacement abilities versus captured external replacement instructions; cast timing and reductions; static trigger suppression; granted Role/attachment triggers; Saga chapter/final sacrifice rules; granted subtype changes and Changeling before layer6. Quoted grant instructions and already resolved duration effects retain their historical program identities when the original granting source loses abilities.

## Concrete representation to prototype

`AbilityKey` should distinguish printed trigger/activation indexes, intrinsic mana/keywords, and continuous static groups. A generated static-fragment-origin table maps existing optimized helper rows back to groups, so evaluation does not reparse Oracle text. Keep full same-ability grouping through extractors that split a combined stats/keyword line into residual synthetic text. Each group's first layer and fragment list must be explicit.

Sparse resolution-grant records retain timestamp, exact recipient, immutable program provenance, granted keyword/ability payload and expiry; original `gained_keywords` can remain a checked fast-path aggregate while records retain order, or be removed once equivalent query performance is established. Stamp static grant providers at source entry/attachment and temporary grants on resolution even before any removal effect exists. Vanilla permanents need no new timestamp allocation. Existing counter-conditioned static grants use the provider's timestamp, not the timestamp of the +1/+1 counter that merely changes their condition.

Predeparture LKI should freeze evaluated ability presence in the event batch or optional event-owned context, not a permanent allocation on every Object or SpellView. Every source observer in a simultaneous batch reads that same frozen world. Source removal after queueing never changes the immutable stack program; source ability removal before the event prevents that packet from being generated.

Funding evaluation needs a borrowed characteristic-world interface that can view real Game objects or virtual resource-consumption state. Reusing a cached real-world 'has abilities' bit would make mana readiness stale when a sacrificed source changes the removal/grant graph. Candidate helper definitions remain static and shared; evaluate only sparse affected sources with an allocation-free common path when no removal rows exist.

### Immediate paid earlier-layer witness

Stop Cold can enchant a noncreature artifact Equipment already admitted in the parent. Ninja's Blades, Samurai's Katana and White Mage's Staff each have one continuous ability adding a Job subtype in layer4, granting triggered/keyword abilities in6 and/or a bonus in7c. Removing the Equipment's abilities6 disables its separate Equip ability, but that one effect has already begun in4 and continues its later fragments on the same recipient under613.6. Test this using a genuinely paid Equipment, actual attach/Job token, and paid Stop Cold; require the retained Job/subsequent-grant/stat fragments and denied Equip. A source-wide effective flag on all Equipment helpers would fail this first-batch case. By contrast Maha's pure7b clause is lost, and Archon's6+7b group obeys the explicit dependency before first application.

## Sparse chronology and dependency checkpoint

The first owned runtime foundation records static layer6 provider timestamps,
resolution keyword effects, and temporary triggered-ability grants. One resolving
instruction shares its timestamp across every affected recipient. Suspect keeps
its persistent designation and records both granted abilities; becoming suspected
again does not refresh that timestamp. Actual new Equipment attachment retimestamps
the provider, while re-equipping the same recipient does not. Cleanup removes only
expiring effects and departure removes only the departing recipient incarnation.
No cache or extra field was added to every Object or Spell.

Three paid native groups passed in both seats: Boros Charm plus Royal Treatment
shared timing and real cleanup, Person of Interest plus repeated Convenient Target
and eventual Unsummon, and Samurai's Katana Job entry plus actual Equip movement.
The frozen 921 tracked inputs remained unchanged; terminal cleanup reclaimed
75,980,800 allocated bytes and retained every shared-library path and digest.
These are foundation checks, not full ability-removal or card-admission acceptance.

All six predefined Role factories now carry their complete static clause origins.
Royal, Sorcerer and Young Hero retain exact granted-trigger index0; Monster's stats
and trample stay one effect. Cursed is a pure7b setter. Wicked's separate death
trigger remains distinct from its static bonus. This extends provenance only;
factory definitions, programs, identities and admitted card counts do not change.

An iterative strongly-connected-component scheduler evaluates the frozen layer6
recipient sets. Dependencies override timestamp order, while only an actual cycle
ignores its internal dependencies. An older effect outside a cycle still waits.
Earlier-started effects and source-independent resolution grants continue; removing
abilities clears prior keywords and nonkeyword grants but preserves the underlying
object's other characteristics. Started effect identities are retained for later
layers. Three scheduler regressions pass, covering these distinctions and both
Suspect-granted abilities together. The first compilation exposed two invalid
`usize::from(u32)` conversions; corrected indexing passed in a fresh target. All922
inputs stayed fixed, 76,046,336 allocated bytes were cleaned, and every SO preserved.
The complete source/provenance/raw control group passes21 methods.

The scheduler is not yet connected to a removal Aura, source queries or prospective
funding worlds. The four priority bodies remain gated until those full semantics
and paid public acceptance are implemented. Schema102 is a prototype reservation;
final composition will use a fresh schema above any intervening merged103 contract.

## Actual and virtual earlier-layer queries

The real Game and speculative FundingState now implement one borrowed fact
interface for current object presence, control, incarnation, life, attack status,
layer4 grants and live-static recipients. Funding overlays supply their own object
and life values; no real-game ability-result cache crosses virtual payments. The
previous duplicate funding count/subtype logic and actual live-recipient predicate
now delegate to that interface. A new both-seat query regression changes only a
virtual Maha source's controller, zone and incarnation, verifies each recipient
change, and verifies the actual game remains unchanged. The existing71 funding
checks pass on this refactor.

The complete native run initially had1903 passing tests and three failures. The
new chronology adds an entry timestamp choice for two simultaneously searched
keyword Auras. Completing that choice exposed a real privacy gap in the old
public-only timestamp observer: it did not apply entry visibility. The observer
now filters both pending and selected rows by the same exact-incarnation visibility
as Aura attachment/life choices. A both-seat private library-search test verifies
that references, card identities and actions remain private through attachment
and timestamp ordering, then commits the full search/shuffle/draw transaction.
The other assertions now account exactly for the shared sparse clock allocation
and distinguish absent Role attachments from the presence of that clock.

After these corrections, complete `cargo test --lib --tests` passes1906 unit plus12
integration tests in23.608s, with all923 tracked inputs fixed. Terminal cleanup
reclaimed220,344,320 allocated bytes; both generated shared libraries retain their
exact paths and hashes. The earlier failing1903/3 run remains a failed scope.
The22 source/provenance/raw controls also pass, including separate Changeling
and flying origins when their independent abilities share one printed line.
These gates cover the shared-query/timeline foundation, not removal-card acceptance.

## Ordered borrowed-world projection

The evaluator now builds each static effect from its immutable group and evaluates
its recipients against the borrowed world. One group can grant distinct abilities
or keyword sets to different recipients without inventing separate source clocks.
Resolution-created keyword and triggered-ability grants preserve historical source,
printed program definition, controller and timestamp independently of source presence.
Started group identities remain available for later-layer fragments. Exact attachment
links are separate from creature-only Equipment grant recipients; a future Stop Cold
artifact recipient must not be discarded by that unrelated filter.

After every paid chronology action the independently ordered projection is checked
against the current executable keyword query, including owned restored branches.
Three both-seat groups pass on the projection checkpoint. A second focused35-method
native run passes after adding the allocation-free default dispatch and optional
owned ObjectView ability snapshot. All924 inputs remain fixed;76,496,896 allocated
bytes were cleaned and the SO preserved. The snapshot carries effective printed and
nonkeyword grant identities only when a removal provider exists; default objects
and spells gain no persistent cache or field.

The removal fragment is not yet emitted by any accepted physical body. Remaining
source/activation/trigger/replacement queries, departure LKI filtering and all four
complete Aura consumers still need wiring before any admission or final acceptance.

The central trigger dispatcher now checks the event's captured printed/granted
ability identities before creating a new packet. Temporary departure grants also
check their individual effect timestamps, so an old erased grant cannot borrow
availability from a later identical grant. Already queued packets keep their owned
programs. Activation announcements and intrinsic mana actions consult effective
printed availability; speculative funding consults its own overlay and keeps a
conservative upper bound when a virtual departure could restore an ability.
Complete native validation passes1906+12 again (24.126s,924 fixed inputs), with
224,165,888 allocated terminal bytes cleaned and both SO identities preserved.
No accepted card emits the removal fragment yet; continuous helper suppression,
untap restrictions and the full four physical bodies remain required.

## Actual schema103 parent composition

The branch now composes merged main2812c74 (1,977 executable Standard cards,
schema103), including complete joint hand/land selection and all three base-power
consumers. Prototype observation schema104 adds optional ability identities. Every
inherited Python policy change is a schema-assertion literal only, with normalized
AST equality verified. No inherited casting guard or raw-authentication route was
removed. Historical modal and base-power union proofs retain their exact delivered
commits and bind the current owned programs/helpers separately; the first31-method
run correctly failed its still-current helper-set assertion before that historical
context correction. The corrected33-method affected source suite passes5.202s.

Strict actual-parent preservation retains all5,333 definitions,928 existing helper
arms,96 token recipes, colors, aliases, flags and the emblem. Exactly772 new static
origin helper rows are added, including the inherited casting restriction and the
legendary token's layer7a CDA. No executable program or admission changes. A durable
unrelated-mutation control rejects; every token CDA is checked for a source identity.

Static base setters, layer7a definitions, additive providers, live clauses and
attachment aggregates now consult their precise groups. Earlier-started Job effects
continue; unrelated abilities and pure later-layer providers can cease. A dedicated
sparse removal-provider index keeps the common no-removal check constant-time and
allows virtual presence changes without caching real-game outcomes.

Complete native validation on the frozen actual-parent source passes1,920 unit and12
integration tests in26.823s. All937 tracked inputs remain fixed. Terminal cleanup
reclaims225,583,104 allocated bytes while preserving both SO paths/hashes. This is
foundation/integration validation: the four priority Auras remain gated, and the
remaining rule providers and full printed bodies still require implementation.

## Rule restrictions and direct-authoring transition

Effect-created “can't block/be blocked this turn” changes game rules rather than
adding an ability. The existing resolving instructions now record those restriction
bits separately and apply them after layer6. Suspect remains an explicit ability
grant: both its menace and quoted can't-block ability can be removed. Conditional
static self-rules and generic external Aura restrictions similarly retain rule-layer
identity; removing their source's printed ability still suppresses the provider.
Existing executable programs are unchanged. All20 existing resolution-rule bodies,
including the Beat a Path inset, were checked against their complete printed text.

The source controls pass34 methods. A fresh36-method native selection passes,
including both-seat genuine paid Enter the Enigma and Sandstorm Verge actions,
owned replay and an explicitly hypothetical recipient-removal boundary. The
hypothetical boundary is not represented as casting a still-gated Aura. All937
inputs stayed fixed;76,816,384 allocated bytes were cleaned and the SO preserved.

Further parser/origin expansion stops here following the direct typed-Rust card
strategy. The shared layer6 machinery is retained. The four complete Aura bodies
will be authored directly, using typed static-effect identities and programs after
remaining runtime providers, untap restrictions and all-counter removal are wired.
No new card admission or final acceptance is claimed at this transition checkpoint.

Runtime untapping now distinguishes effect-driven attempts from the current
controller's untap step. A prohibition is checked before removing Stun counters;
restricted multi-object untaps freeze eligibility before changes, while the common
unrestricted turn path keeps its allocation-free loop. TapAttached can use any
exact attached permanent, and a Subject-based full-counter reset supports River.
Thirty-six existing step-related native tests pass on frozen938 inputs (18.014s);
terminal cleanup reclaims76,873,728 allocated bytes with its SO unchanged. Actual
prohibited-untap cases await the directly authored complete Aura definitions.
