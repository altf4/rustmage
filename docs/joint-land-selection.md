# Joint owned hand/graveyard land selection

PR #339 / issue #336 completes **Worldsoul's Rage** and **Bonny Pall, Clearcutter**
on merged main `dffafa6`. Coverage is **1,974/5,164 Standard identities**, with
3,190 gated, 96 token recipes, one emblem and observation schema101. Full Standard
remains incomplete. The six-body audit in
[hand-graveyard-land-admissions.json](hand-graveyard-land-admissions.json) pins
112 source records / 63 distinct full Java files, including Beau and its explicit
imports. Wildcard imports are recorded rather than claimed as a transitive closure.
Kellan, the Kid; Michelangelo, Improviser; Spelunking; and The Gitrog, Ravenous Ride
remain gated for their complete residual mechanics, tracked by issue #330.

## Resolution and private choices

Worldsoul's Rage pays and captures selected X through the existing casting engine,
deals that amount to its legal target, then offers an optional selection of up to
X land cards from the resolving player's hand and graveyard. Exact references
accumulate in the existing sparse owned choice. No selected land moves before the
player finishes or reaches the limit; one entry batch handles the whole set,
including suspended entry-replacement payments. Placement does not consume the
normal land-play allowance. A fully illegal damage target fizzles the whole spell
without offering land selection. Known X=0 needs no choice.

Hand candidates and selected hand references are hidden from the peer. Public
graveyard candidates/selections remain public. The same optional phase appears
with no eligible hidden land, allowing a decline without proving hidden-card
availability. Selection and final commit check owner, exact incarnation, original
zone, land type, duplicates and captured limit. Entry uses the resolving controller
independently of the physical card's owner. Source departure does not change the
historical controller of an already announced ability.

Bonny's complete body includes Reach, its Beau entry trigger, and attack-triggered
draw followed by optional hand/graveyard land placement. A generic named legendary
token recipe lowers Beau's whole self-only characteristic-defining ability through
existing typed `DynamicStats`. Beau is a legendary blue Ox whose layer7a power and
toughness track its current controller's lands independently of Bonny's presence.
The legend rule groups tokens by printed token name, rather than catalogue factory
identity. No new per-object field or stat-layer evaluator is introduced.

There is one intentional rules correction relative to the pinned Java source:
`BeauToken` constructs `SetBasePowerToughnessSourceEffect` in layer7b. The Oracle
ability granted by Beau's creation is unconditional and affects only that token,
so it meets CR604.3a and belongs in layer7a under CR613.4a. Rust uses its existing
characteristic-defining path accordingly. The Java hashes establish audited source
provenance, not equivalence to that Java layer assignment. See the
[official OTJ release notes](https://magic.wizards.com/en/news/feature/outlaws-of-thunder-junction-release-notes)
and [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).

## Whole-body admission and preservation

Generic grammar recognizes the complete selected-X damage/placement program, the
singular hand-or-graveyard land clause, and a complete named legendary token CDA.
Admission authenticates original raw text; unknown suffix, infix, nested reminders
or unrelated CDA references are rejected across generic callers and renamed or
reserved-name bodies. No card-name admission whitelist is used.

The strict actual-parent proof changes exactly the two physical definitions and
preserves all5,330 others. All924 inherited helper arms, all95 old token recipes,
colors, aliases, emblem and inline flags remain unchanged. The only additions are
Worldsoul's selected-X helper and Beau's recipe, factory alias and dynamic-stat
helper. The two complete programs and Beau recipe equal the earlier prototype.
A mutation control verifies that preservation rejects an unrelated change.
Inherited raw/modal and live-static preservation tests retain their exact delivered
historical unions and current owned bindings. Schema assertions follow101.

## Validation and evidence scope

Frozen production `e79169e` passes **1,906 unit +12 integration tests (1,918 native)**,
all-target Clippy with warnings denied, formatting, generator check and optimized
library build. All917 tracked execution inputs remain fixed during those gates.
All41 affected compiler/provenance methods pass in7.476s, including whole-body,
renamed/reserved-name, malformed type/cost, raw-text, remaining-family, historical
preservation and mutation controls. This is an affected suite, not a claim of a new
full compiler or full legacy public-API run.

Seven paid native groups exercise both seats, actual Korvold foreign-casting
permissions, source departure, exact references, private eligibility/decline/X=0,
normal quota, simultaneous landfall and tapped entry, shockland payment suspension,
hexproof/fizzle, Bonny's attack/draw, Beau's printed-name legend rule and live CDA.
Checked actions replay raw/index paths through owned forks, parent drop and restore,
with wrong-actor and stale-revision rejection. Interior selected-X values use the
continuous legal domain's raw action path.

Final policy `710a1b0` passes all14 paid natural Standard vectors in11.405s.
Final `bbdb222` adds the durable fixed workload without changing production:
100 naturally terminal, unretried games pass in117.008s across all14 vectors,
with60,866 owned frames and121,732 wrong-actor/stale denials. Four parent public
methods pass in166.203s with no failures, errors or skips:24 private-hand/modal
cases,12 live-stat cases,34 promised Gift cases and18 Suspect cases (88 total).
All918 final tracked inputs and the same immutable schema101 library remain fixed.
Production/generator/carddata/native tests are byte-identical to `e79169e`; only
the public setup policy and workload example changed afterward.

Historical failures remain diagnostic rather than passing claims. The first
actual-parent affected run passed40/41 and failed an obsolete Bonny-gated fixture;
its corrected41-method run passed. Initial public `e79169e` passed6/14 cases and
failed8 setup cases because cleanup discarded required hand lands. Policy3958f86
passed10/14, then exposed three later Bonny-trigger witness checks and one missing
second blue mana behind the setup land cap. Policy-only corrections reserve actual
witnesses, fund required colors and decline later optional attacks' land choices.
No runtime change was made for those failures. Earlier prototype harness, fixture,
and token-count failures are retained in development logs without a green claim.

All jobs are terminal. Completed build non-library outputs were cleaned while
preserving every shared-library path and content. Two independent nonauthor
exact-final-head reviews are required before merge.
