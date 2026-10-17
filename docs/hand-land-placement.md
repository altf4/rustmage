# Private hand-land placement — issue325

Prototype from actual main8dd3952. The complete twelve-body audit records187
source/direct-import records and86 unique complete pinned Java files. Five full
bodies now lower: Embrace the Paradox, H.E.R.B.I.E. Scout Unit, Insidious Fungus,
Lessons from Life and Nicanzil, Current Conductor. Seven bodies retain explicit
whole-body dependencies tracked in issue330, including PuPu's dynamic base-power activation. No
name whitelist, partial body or erased unknown reminder authorizes execution.

The prototype generalizes existing owned private HandCreature choices to an
optional exact HandLand choice, always retaining decline even with no eligible
card so peer scheduling cannot reveal hidden hand types. Both branches use the
real entry transaction; land placement does not consume a normal land play and
preserves printed entry replacements and immutable ownership. Observation schema97
is reserved for the new choice/action contract. Exact revealed land/nonland explore
matching occurs at event queue time; empty-library exploration matches neither.

Insidious Fungus uses one real activated ability with announce-time modes before
targets and payment, reusing the existing modal kernel. Its three paid modes and
source sacrifice must all be tested. Modal external-object costs need joint per-mode reservation/target funding proof;
they remain gated. Modal mana-production abilities remain gated
until their classification/production solver is implemented.

Prototype execution source a36efe3 passes all 1,847 native tests (1,835 unit and
12 integration), all-target Clippy and formatting. Eleven paid/adverse groups
cover both seats, all five whole bodies and Fungus modes, real Ward-countered
and stale-target-fizzled activations after source sacrifice, printed entry-life
choices and landfall, source departure, supplied real foreign exile permissions,
wrong owner/controller/kind/zone, stale generations, late transactional overflow,
and full peer-view equality across different hidden eligibility for both land and
inherited creature hand placement. Every action checks owned parent-drop, clone,
index/raw, restore and wrong-actor/stale rejection.

Seven compiler/raw/cost/type/current-parent/source-audit methods pass. The strict
comparison is exactly five changed definitions, 5,326 preserved and all 888 prior
helpers/colors/aliases/recipes unchanged; no helpers or factories are added.

The optimized prototype library is
`/home/altf4/.cache/rustmage-immutable/handland325-a36efe3-b683-schema97-e8172e49872d.so`,
SHA256 `e8172e49872d596b0f0d5d55f57e6ec77fa1db8fd69231699799a1e98ed2e53c`,
FP `b68305118668e7c0c2078d057f9fe1490fc54ff983b48fa6cbeb59702817025c`.
The initial naturally drawn paid Standard policy passes all 24 body/mode/decline
cases in 40.877s; adding explicit one-mana Map payment assertions then passed
the same 24 cases in 35.040s. Both Nicanzil printed conditions execute in each of its games,
using actual paid Map exploration with exact current-counter assertions. A prior
policy run's ten failures were one fixture error: checking entry tapped state
again after later natural untaps. The corrected fixture checks it once at entry;
no engine change accompanies that policy correction.

Final actual-main composition, fresh combined gates and the 100-game workload
remain required. No prototype result is a delivery or full Standard
completion claim. Two independent NONAUTHOR exact-final-head reviews are required.
