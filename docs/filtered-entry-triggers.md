# Filtered owned-permanent entry triggers

Issue: https://github.com/altf4/rustmage/issues/178

Eight complete bodies bring executable coverage to **1,329/5,164** pinned
Standard cards, with **3,835 unsupported**. Six non-Standard regression cards,
57 token factories and one emblem remain separate. The full migration remains
in progress; Standard is pinned to October 15, 2026.

| Card | Complete body |
| --- | --- |
| Arahbo, the First Fang | Legendary 2/2; other owned Cats get +1/+1; self or another owned nontoken Cat entering creates a 1/1 white Cat |
| Arni, Humble Scribe | Legendary 3/2; another owned nontoken creature entering untaps this source; tap to draw one then discard one |
| Black Panther, Vanguard | Legendary 4/4; another owned nontoken Hero entering chooses a 1/1 white Soldier or +1/+1 for current owned creatures until end of turn |
| Genghis Frog | Legendary 1/3 trample; self or another owned Mutant entering creates a Mutagen with its complete one-mana/tap/sacrifice, sorcery-only counter activation |
| Knightfisher | 4/5 flying; another owned nontoken Bird entering creates a 1/1 blue Fish |
| Lathliss, Dragon Queen | Legendary 6/6 flying; another owned nontoken Dragon entering creates a 5/5 red flying Dragon; one generic and one red gives current owned Dragons +1/+0 until end of turn |
| Pactdoll Terror | 3/4 artifact creature; self or another owned artifact entering makes each opponent lose one life and its controller gain one life |
| Thoughtweft Lieutenant | 2/2; self or another owned Kithkin entering gives a targeted owned creature +1/+1 and trample until end of turn |

`filtered-entry-admissions.json` records all eight full Oracle bodies and **97
SHA-256 entries across 41 unique Java files**, pinned to
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All constructors were read in full.
The manifest includes event/filter classes, source and target effects,
intrinsics, costs and the five existing token definitions. Recorded blobs
match both the pinned revision and read-only checkout.

## Conditions, source identity and complete compilation

The immutable `PermanentEntersYou` condition specifies a kind mask, optional
subtype, nontoken qualifier and explicit self-entry alternative. The controller
must match. For another permanent, all qualifiers must hold. For this exact
source entering, only the explicit self arm applies; it bypasses the
other-permanent filter and fires once, even if the source also matches it.

The word creature requires creature type; artifact requires artifact type.
Unqualified subtypes such as Bird, Hero or Dragon apply to permanents with
that subtype, without an extra creature restriction
([CR109.2](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)).
Changeling matches creature subtypes. Nontoken qualifiers exclude matching
tokens; unqualified Mutant, Kithkin and artifact clauses include them.
Pactdoll observes noncreature artifacts, artifact creatures and artifact tokens
once per entry. It loses/gains life without targeting or dealing damage.

Existing post-batch entry observers see every simultaneous newcomer. Existing
APNAP and owned trigger ordering remain in force. Sources, declarations,
selected modes/targets and private continuations retain exact object
references. Old source effects cannot untap a reentered Arni; targeted and
token/life effects survive source departure. Thoughtweft revalidates its
declared target and cannot substitute a new incarnation. Source departure and
temporary effect expiry update Arahbo's static bonus and temporary grants
normally.

The compiler accepts exact supported subjects and whole effects, including
intervening-if and once-per-turn wrappers. Authenticated full/short source
names are normalized only in exact event headers or a whole source-untap
clause. Arni's original legendary identity authenticates its short name before
the permanent compiler constructs a creature-shaped proxy. Arbitrary names and
entering-object pronouns are not inferred.

Every modal branch, activation and residual clause must compile. Unknown
predicates, type/copy/control changes, unsupported unions, multiplicity,
entering-object effects and residual mechanics remain gated. Existing simple
entry conditions and all previously executable definitions remain unchanged.
No new mutable state field, snapshot layout, cache, token factory, observation
schema (**51**), C ABI (**1**) or control protocol (**2**) change is introduced.

## Validation

Thirteen native tests cover all eight actual full casts from both seats,
intrinsics, matching nontoken/token entries, changeling, opposing and wrong-type
exclusions, noncreature artifacts, self entry once, simultaneous new sources,
APNAP with two owned triggers per player, both Black Panther modes and current
squad membership, exact source/target incarnations, all three activation types,
private discard ownership, temporary expiry and atomic life overflow rollback.
Semantic actions compare entire cloned states and restored replay, including
stale-revision rejection.

Three compiler tests check exact fields, old-condition preservation, guard and
once wrappers, unknown subjects/predicates/pronouns, authenticated source names,
all eight complete admissions and unknown residual gating. Prior negative
fixtures for the newly supported Bird predicate, Knightfisher, Arahbo and Black
Panther now use the new boundary or remain covered by full admission checks.
One fresh release Python method runs nine scenarios: every complete body, both
Black Panther modes and all three activation types, with independent public
effect checks and native/private-choice replay.

Two deterministic Standard100 runs use three legal 60-card/four-copy deck
families and reproduce:

- **66,407 decisions**, casts `[77,155,135,204,99,50,115,139]` in table order.
- Triggers `[98,113,32,267,29,3,226,179]`.
- Arni/Lathliss/Mutagen activations `[131,16,248]`.
- Independently verified trigger programs
  `[98,113,32,266,29,3,223,179]`, plus all `[131,16,248]` activations.
- Black Panther modes `[20,12]`, **189,141 public stat/ability pairs**,
  **131 private choices**, **10,781 replay boundaries** and **21,972 maximum
  initialized logical state bytes**.

The witness manually models token creation, source untap, both modes, life,
target growth/keywords, draw/discard, Dragon squad growth and Mutagen counters
from public before/after facts. Its separate exact-incarnation/turn ledger and
manual Cat/Dragon membership model check every creature's public stats and
abilities for both viewers. It does not read private instruction definitions.
Terminal-producing effects are excluded from resolved-program counts. The
Dragon family makes Lathliss's triggered program an explicit acceptance
requirement, alongside its activation.

Complete checks: **1,109 Rust tests** (1,097 unit and 12 integration), **212
compiler/data tests**, **144 fresh release Python tests**, all-target Clippy
with warnings denied, formatting, catalogue consistency and diff checks.

The separate 1,000-game benchmark completed 571,049 decisions at approximately
414,179 decisions/s and 725 games/s, clone 129 ns and restore 122 ns. Initial
logical state remains **19,784 bytes**. These timings depend on the host and
include policy and observations, exclude inference/tensors/JSON, and are not
an XMage parity comparison. Logical accounting excludes capacity/allocator
overhead, COW sharing, immutable definitions, observations and external
snapshots.
