# Exact live subject conditions

Issue [#213](https://github.com/altf4/rustmage/issues/213) implements nine complete Standard bodies. The integrated release builds on main `cbe9d3d7af0d2b5c5532e8faae159fed3b583c0b`, including target-dependent costs. Historical standalone measurements below use library-destination checkpoint `990475788f7beb37a34ebfede0313caad5e4c290`; combined evidence follows them.

| Card | Complete body and condition |
| --- | --- |
| Aunt May | Legendary {W} 0/2 Human Citizen; each other controlled creature entry gains one life; the exact entering Spider gets one +1/+1 counter |
| Dreamdew Entrancer | {2}{G}{U} 3/4 Frog Wizard with reach; ETB taps zero or one creature, adds three stun counters, then draws two only if its controller controls that creature |
| Fear of Immobility | {4}{W} 4/4 enchantment Nightmare; ETB taps zero or one creature, then adds stun only to an opponent's chosen creature |
| Hazardroot Herbalist | {2}{G} 1/4 Rabbit Druid; one aggregate attack trigger boosts a controlled creature +1/+0, then grants deathtouch only if that creature is a token |
| Ingenious Leonin | {4}{W} 4/4 Cat Soldier; {3}{W} puts a counter on another attacking controlled creature, then grants first strike only to a Cat |
| Little Bear | {2}{G} 3/2 Bear with flash; ETB untaps another controlled creature, then adds a counter only to a Bear |
| Thwip! | {W} instant; gives a creature +2/+2 and flying, then gains two life only if it is a Spider |
| Wakandan Royal Guard | {4}{G} 4/4 Human Soldier Hero with vigilance; ETB places either two counters on another Hero or one counter on another creature or itself |
| Yip Yip! | {W} Lesson instant; gives a controlled creature +2/+2, then flying only if it is an Ally |

## Immutable execution

`Instruction::IfSubject` checks an existing immutable `CreatureFilter` against one exact live `Subject`. The compiler uses `Targets` for a single creature target and `Entering` for Aunt May's captured entry context. Relation, subtype/Changeling, token and source-versus-other checks share the existing intrinsic creature predicate. No mutable registry, object field or continuation field is added. Observation schema stays **58**, C ABI 1 and control protocol 2.

Target guards require the bound `ObjectRef` generation to remain current. The shared resolution boundary first revalidates targets, including controller relations, combat membership and hexproof. A packet whose chosen targets are all illegal fizzles completely. Zero-target optional ETBs resolve their empty operation without a conditional benefit. An unrelated current creature, a source's own subtype or a departed-and-returned incarnation cannot satisfy the subject binding. The current engine's admitted subset has no dynamic control changes; existing owner/controller equivalence remains its boundary.

Aunt May's entry reference is non-target context captured by the existing ETB dispatcher. Life gain executes first and remains independent if the source or entrant leaves. A later counter requires that exact entering object to remain a Spider on the battlefield. Simultaneous entrants retain separate packet references; suppression prevents the packets entirely. An independent queued Leonin or entry ability survives its source leaving. Temporary growth and keywords expire normally; life and counters persist.

Wakandan's program branches **before** placing counters: a true other-Hero guard executes one `PutCounters(amount: 2)` and skips the one-counter branch; false executes one `PutCounters(amount: 1)`. It never places one counter and then another. Other bodies preserve their printed sequence, including tap/stun before drawing, untap replacement before the Bear counter, and normal boosts before conditional benefits. Global capacity rollback still restores every preceding instruction, pending packet, RNG state and revision when a later operation fails.

The compiler accepts closed whole conditional clauses and single creature antecedents whose earlier instructions retain that exact battlefield subject. Moving-target antecedents, ambiguous pronouns, multi-target singular antecedents, unknown conditional tails and unimplemented residual text reject the complete body. [Cemetery Recruitment and Budding Insurgent](subject-condition-admissions.json) remain gated: their moving-object/LKI conditions are outside this primitive. Blink, control changes, dynamic types, ability removal, keyword counters and unrelated residual mechanics are not added.

## Standalone provenance and preservation

The [manifest](subject-condition-admissions.json) records complete Oracle text for nine admissions and two explicit gates, with **40 SHA-256 entries across 40 Java files**: eleven constructors and twenty-nine inspected shared helpers. Every byte hash is verified against both the local XMage checkout and pinned revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

Support becomes **1,504 / 5,164** Standard cards; **3,660** remain gated. There are six additional regression cards, **72 token factories**, one emblem and **5,256 immutable identities**. Fingerprint: `65b178a8f25f9f738202d90d0b2bac7de6ee645dd24d038341a02a9bb5eccd39`.

Canonical comparison against the parent changes exactly the nine previously unsupported bodies and preserves **5,247** other definitions, all **5,256 identity/ID mappings and color masks**, **5,183 ordinary IDs**, all **72 existing factories/metadata**, the emblem and **446 prior numeric helper arms across eighteen functions**. No factory or helper arm is added. Apart from those nine definitions, the fingerprint and their required additions to the supported-card `ALL` list, generated Rust remains byte-identical.

## Standalone validation

Sixteen native tests cover full casts in both seats, both live branches, optional zero/one target cardinality, subtype/Changeling/token/relation/other predicates, source/target departure and reincarnation, independent life gain, simultaneous captured entry references, suppression, attack membership, Ward payment/decline, wrong actors, full fizzles/hexproof, stun replacement, temporary expiration, permanent counter/life persistence, divergent owned choices and late capacity rollback. Every replay helper checks clones, snapshots, full state equality and stale action revisions.

Public Python acceptance exercises **40 scenarios**: nine bodies in both seats and both live branches, plus both optional ETBs with zero targets in both seats. It checks effect deltas through the public ABI, illegal actors/object IDs, divergent target clones and deterministic restore/replay. The full suite passes **1,349 Rust tests** (1,337 unit plus twelve integration), **257 compiler/data tests** and **164 fresh-release Python tests**, with formatting, warnings-denied all-target Clippy, catalogue verification and whitespace checks.

Two legal Standard100 runs reproduce **62,864 decisions/replays**. In table order, casts are `[16,20,17,21,12,25,24,31,17]`, effects `[78,20,17,55,16,25,24,31,17]`, positive conditions `[29,13,9,7,9,12,13,16,13]`, negative conditions `[49,7,8,48,7,13,11,15,4]`, and controller counts `[152,131]`. There are **78 Aunt life triggers**, **26 conditional draws**, **37 tap effects**, **12 real untaps**, **6 self-one/16 other-Hero-two counter operations**, **7 token-deathtouch checks** and **404 illegal-action rejections**. Every committed decision is cloned and restored/replayed; both subtype outcomes and both controller seats are witnessed for every body.

Initialized logical state remains **19,944 bytes**. The new workload reaches **22,180 bytes**. A contended 1,000-game native smoke run retains 571,049 decisions, approximately 147,000 decisions/s and 307/589 ns clone/restore. These are host-load observations, include native smoke policy/observation construction, exclude model inference/tensor encoding/JSON, and are not a Java performance comparison. Logical bytes exclude capacity, allocator overhead and external snapshots.

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_subject_conditions -- 100
```

## Integrated release evidence

Combined support advances from 1,514 to **1,523 / 5,164 Standard cards**, with 3,641 gated and six additional regression bodies. The catalogue retains 5,256 identities, 72 token factories and one emblem. Exactly nine definitions change; all 5,247 others, prior IDs/color masks/factory metadata and **466 existing numeric helper arms** remain unchanged. All-section provenance verifies forty SHA-256 entries across forty pinned Java files. Fingerprint: `6887dacf503b5b12795192b85c8f7009221784be16d217f34c8f04300f46d3db`.

The integrated release passes **1,363 Rust tests** (1,351 unit and twelve integration), **260 compiler/data tests** (430.128 seconds) and **166 fresh-release Python tests** (201.192 seconds), plus warnings-denied all-target Clippy, formatting, generated-catalogue and whitespace checks. The subject-condition Standard100 reproduces all **62,864 decisions/replays** and the standalone vectors above. Adjacent target-cost Standard100 retains all 60,950 decisions/replays and every payment/effect/private/locked-cost witness.

Observation schema remains **58**; initialized logical state remains **19,944 bytes**, with 22,180 bytes maximum in the subject-condition workload. The combined 1,000-game native smoke benchmark retains 571,049 decisions, approximately 205,000 decisions/s and 229/239 ns clone/restore under concurrent host load. These workload measurements carry the exclusions above and are not a Java performance comparison.
