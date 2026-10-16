# Target-owner library destinations

Issue [#209](https://github.com/altf4/rustmage/issues/209) adds exact targeted bottom moves and an owned top/bottom decision made by the target's owner. Thirteen of the sixteen audited candidates have complete executable bodies on main `092dda1101f9c795bd884986fbf4ca5277cf7262`.

| Card | Complete implemented body |
| --- | --- |
| Chrome Companion | 2/1 artifact Dog for {2}; becoming tapped gains one life; {2}, tap bottoms a card from any graveyard |
| Hoverstone Pilgrim | 2/5 artifact Golem for {5}; flying, ward {2}; {2} bottoms a card from any graveyard |
| Jade-Cast Sentinel | 1/5 artifact Ape Snake for {4}; reach; {2}, tap bottoms a card from any graveyard |
| Malevolent Chandelier | 4/4 artifact Construct for {6}; flying; {2} bottoms a card from any graveyard, only as a sorcery |
| The Spot's Portal | {2}{B} instant bottoms a creature; the resolving spell’s controller loses two life unless that player controls a Villain after the move |
| Lost in Space | {3}{U} instant lets an artifact or creature's owner choose top/bottom; the resolving controller then surveils one |
| Misleading Motes | {3}{U} instant lets a creature's owner choose top/bottom |
| Return to the Sewers | {3}{U} instant lets a creature's owner choose top/bottom; the resolving controller then creates a Mutagen |
| Riverwalk Technique | Both {3}{U} instant modes: owner top/bottom for a nonland permanent, or counter a noncreature spell |
| Trip Up | {3}{U} instant lets a nonland permanent's owner choose top/bottom; cycling {2} remains complete |
| Uncharted Voyage | {3}{U} instant lets a creature's owner choose top/bottom; the resolving controller then surveils one |
| Unlucky Drop | {3}{U} instant lets an artifact or creature's owner choose top/bottom |
| Vanish from Sight | {3}{U} instant lets a nonland permanent's owner choose top/bottom; the resolving controller then surveils one |

Dire Downdraft, Run Behind and Uneasy Partings remain fully gated. Their target-dependent cost reductions require a proposed-target funding model; their tuck sentence alone is not sufficient. Mixed spell/permanent targets (Sudden Setback/Swat Away), tiered/spree, Adventure/warp and unrelated exile bindings also remain gated. The compiler recognizes complete exact noun phrases and residual clauses; appending unknown text rejects the whole body. The existing Mutagen factory is reused without changing its recipe or introducing a new token identity.

## Owned operation

Standalone observation schema **57** (combined schema **58**) adds actor-private `target_library`, containing the frozen source reference, target reference and choosing player. The C ABI remains 1 and control protocol remains 2. `ChooseLibraryDestination { object, destination }` accepts only the frozen exact object and `Top` or `Bottom`. The actor is the target owner, independently of the resolving controller. Other observers receive neither the packet nor its legal actions.

The chosen single-card destination is public. Both viewers receive identical `last_target_library_move` with `source`, canonical `card` identity, `target_before`, `target_after`, `owner` and `destination`. This snapshot-owned, boxed record is replaced only by the next successful targeted library movement. It survives unrelated draws, private looks and hidden rearrangements unchanged, including when its exact references have become historical. It is an event, **not current library-position knowledge**. Clients observing every committed action can maintain public history without an external unexposed action transcript; clients skipping actions receive only the latest event. It reveals no unrelated library contents or private reorder choices. Capacity failures restore the earlier public event.

The boxed resolution continuation owns source/controller and exact bound targets; its enclosing resolution owns the next instruction position. Before the destination action, the target remains in its original zone and subsequent instructions have not executed. Clone and snapshot operations own the entire continuation. Revisions invalidate old action indices after restore; forged generations, outside objects, wrong actors and graveyard destinations are rejected without mutation.

Final movement uses the central zone-transition path: attachments, combat declarations, observers, generations, known-hand references and indexes follow existing rules. Library order uses the owner's actual library: top appends and bottom inserts before the untouched existing cards. No draw, mill, sacrifice or shuffle event is invented. Tokens leave through the ordinary transition and cease to exist at the normal state-based-action boundary. The ordered library index contains cards only. A token still makes its real zone change and waits for the normal SBA boundary, but cannot be looked at, selected or drawn as a card while a following instruction resolves ([CR 111.6–8 and 701.25a](https://media.wizards.com/2026/downloads/MagicCompRules%2020260619.pdf)). Sources may leave before an independent activation resolves; targets must retain their exact incarnation and legal target characteristics. Following surveil or token creation runs only after the destination commits, with the resolving controller as actor where required.

Capacity failures restore the complete pre-action state, including a moved target followed by a failing spell departure, pending choices, events, zones and revision. No global mutable decision state exists.

## Audit and validation

The [provenance manifest](library-tuck-admissions.json) records all sixteen complete Oracle texts and **122 SHA-256 entries across 59 Java files**, checked against both pinned XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7` and the local checkout. It distinguishes thirteen admissions from the three cost-related gates.

Standalone core support is **1,468 / 5,164** Standard identities; 3,696 remain gated. There are still 71 token factories and one emblem. The frozen catalogue has 5,255 immutable identities. Canonical comparison changes exactly thirteen previously unsupported definitions and preserves the other **5,242** rows, all **5,255** identities, all **71** factory recipes and metadata, all **442** earlier helper arms across eighteen functions, and every color mask. The catalogue fingerprint is `20d5e06038d843a836577791308e1aa61e37046539907ec2702bffa183994dca`.

Standalone initialized logical state is **19,912 bytes**, sixteen bytes above the parent. The boxed continuation, initialized bound references and latest public movement record are included when present; capacity and allocator overhead remain excluded. A 1,000-game smoke run completed 571,049 decisions at approximately 283,000 decisions/s, with 127 ns clone and 125 ns restore under concurrent host load. It includes native smoke policy and observation construction, excludes model inference/tensor encoding/JSON, and is not a Java comparison.

The native suite passes **1,306 tests** (1,294 unit and 12 integration). Nineteen new tests cover every full body in both seats, both library destinations, frozen exact references, owner privacy, deferred tails, source departure, departure/reentry and hexproof, ward payment/decline, sorcery/summoning restrictions, Mutagen activation, cycling, both Riverwalk modes, live Villain conditions, attachment/combat/token cleanup, divergent branches and early/late capacity rollback. Three additional compiler tests cover full-body admission, exact grammar, complete tail order and residual-text gates. All **248 compiler/data tests**, **159 existing fresh-release Python tests**, all-target Clippy with warnings denied, formatting, catalogue verification and whitespace checks pass on this core.

The separately authored [Standard workload](../examples/standard_library_tuck.rs) and [Python acceptance](../python/test_library_tuck.py) are integrated. All **42 Python scenarios** pass, covering the thirteen bodies in both seats and both printed destinations. Two author acceptance runs and the combined integration run reproduce **100 legal Standard games / 71,875 decisions/replays**. Cast counts in manifest order are `[24,25,23,21,19,26,20,25,19,16,21,16,26]`; move counts are `[60,48,49,52,18,26,15,21,15,15,14,15,20]`. Owner choices split **50/91**, destinations **70 Top/71 Bottom**. The workload verifies **8 cycling resolutions**, **3 counter-mode resolutions**, **60 surveils**, **21 Mutagen creations**, **282 illegal-action rejections**, **141 divergent continuations**, **60 deferred surveils**, **145 Chrome life triggers**, **73 Ward payments** and **18 Mutagen activation counters**. Both viewers receive the same exact historical public move; private surveil/draw does not rewrite it. Python also checks the actual subsequent owner draw after Top.

## Combined integration

Integrated on main `e95897ca0dc11f17b108a9d9059d97c7b70e1834`, this branch supports **1,495 / 5,164** Standard cards, with **3,669** gated identities, six regression cards, **72 token factories**, one emblem and **5,256 immutable identities**. Fingerprint: `c945320f4cf738ccb6765f5081fca7c77511dc69369781619e58dd1bd81f0166`. Observation schema **58** combines these owned destination/history fields with exile play permissions; C ABI remains 1 and control protocol 2.

Canonical comparison against that merged main changes exactly thirteen previously unsupported definitions and preserves **5,243** other rows, every previous identity/color mask, all **72** prior factories/metadata, **446** numeric typed helper arms and **5,183** ordinary IDs. No factory or helper arm is added. All manifest sections—thirteen admitted cards, three gated constructors and shared runtime sources—verify **122 hashes / 59 Java files**.

Combined validation passes **1,333 Rust tests** (1,321 unit and 12 integration), **254 compiler/data tests** (333.780 seconds), **162 fresh-release Python tests** (97.738 seconds), formatting, warnings-denied all-target Clippy, catalogue and whitespace checks. The adjacent exile, filtered-selection and forage workloads retain all vectors and **60,677 / 71,080 / 65,729 decisions/replays** respectively.

Measured initialized logical state is **19,944 bytes**, **32 bytes above merged main**. Combined layout changes are measured rather than added from standalone pointer sizes. The tuck workload reaches **21,608 bytes**; exile/selection/forage reach 21,228/20,752/21,052 bytes. A contended 1,000-game smoke run retains 571,049 decisions, about 222,000 decisions/s and 369/302 ns clone/restore. These are host-load observations under concurrent validation, not comparative performance claims; all standalone benchmark scope and accounting exclusions above still apply.

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_library_tuck -- 100
```
