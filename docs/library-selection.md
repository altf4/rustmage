# Private library selection

Issue [#200](https://github.com/altf4/rustmage/issues/200) adds fifteen complete Standard bodies. Integrated on main `cea0eabf3dd759299a306239f92d413576b49c0f` after the independently reviewed offspring implementation, this branch implements **1,449 of 5,164** pinned legal cards, with six regression cards, 70 token factories and one emblem. It leaves 3,715 Standard bodies gated. Its catalogue fingerprint is `684a9b9b9d964a4c5184505ce936cbd8144aaf7066a04927fc209c1d7d3b0644`. The standalone flashback-based milestone supported 1,422 Standard cards with 60 factories and fingerprint `5e850975698853961ea14828c130a020bfa920750020a298df790a2ad1dea37b`; measurements below were first collected there.

| Card | Complete implemented body |
| --- | --- |
| Accumulate Wisdom | Look three; put one into hand, or all three with three Lessons in your graveyard; order the remainder on the bottom |
| Beastrider Vanguard | 2/2 for {1}{G}; {4}{G} looks three, optionally reveals one permanent into hand, orders the remainder on the bottom |
| Boughside Wanderers | 4/4 for {4}{G}{G}; entry looks four, optionally reveals one permanent into hand, random bottom remainder; landfall grants +2/+2 this turn |
| Commune with Beavers | Look three; optionally reveal one artifact, creature or land into hand; order the remainder on the bottom |
| Commune with Evil | Look four; put one into hand and the rest into the graveyard; gain three life |
| Commune with Nature | Look five; optionally reveal one creature into hand; order the remainder on the bottom |
| Confounding Riddle | Both modes: look four, put one into hand and the rest into the graveyard; or counter a spell unless its controller pays four |
| Flow State | Look three; put one into hand, or two with both an instant and a sorcery in your graveyard; order the remainder on the bottom |
| Follow the Lumarets | Look four; optionally reveal up to one creature/land, or up to two after gaining life this turn; random bottom remainder |
| Living Phone | 2/1 artifact creature for {2}{W}; death looks five, optionally reveals one creature with power at most two; random bottom remainder |
| Morbius the Living Vampire | 3/1 for {2}{U}{B} with flying, vigilance and lifelink; {U}{B} and exile from graveyard looks three, puts one into hand and orders the remainder on the bottom |
| Rakshasa's Bargain | All eight {2/B}{2/G}{2/U} payment equivalents; look four, put two into hand and the rest into the graveyard; printed mana value remains six |
| Resentful Revelation | Normal {1}{B} and flashback {6}{B}; look three, put one into hand and the rest into the graveyard; flashback departs to exile |
| Squad Rallier | 3/4 for {3}{W}; {2}{W} looks four, optionally reveals one creature with power at most two; random bottom remainder |
| Stock Up | Look five; put two into hand and order the remainder on the bottom |

The original thirteen candidates all compile in full. The same closed grammar additionally admits Boughside Wanderers and Living Phone, whose remaining landfall/death bodies already have native enforcement. No unsupported sentence, extra qualification or arbitrary remainder destination is ignored. Unknown residual text gates the entire card. Existing modal, activation, hybrid-cost, flashback, characteristic-definition and trigger machinery supplies the other parts of each body.

The [Java provenance manifest](library-selection-admissions.json) records **99 SHA-256 entries across 44 unique files**, verified against revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7` and the local checkout. It includes all fifteen constructors, look/pick/remainder helpers, conditional selectors, permanent/power filters, graveyard costs and life watcher. Comparing generated definitions against the shared flashback checkpoint changes exactly fifteen old rows and leaves all 5,244 identities and existing token recipes unchanged. Integration with offspring likewise changes only those fifteen rows, preserves all 5,254 identities and all 70 factories, and leaves 5,239 definitions unchanged. Every original Oracle byte and compiler source remains fingerprinted.

## Owned choices and search

Integrated observation schema **55** adds actor-private `library_selection`; the C ABI remains 1 and the control protocol remains 2. The standalone author branch used schema 54; schema 55 includes both offspring and library-selection fields. The view contains the looked `remaining`, `selected` and ordered `bottom` objects, exact eligible `candidates`, `minimum`/`maximum`, `ordering`, `reveal` and `remainder` (`graveyard`, `bottom_any` or `bottom_random`). Candidates identify eligible hand picks; bottom ordering can choose any remaining card. The opponent sees no window or legal actions. Ordinary observation objects never expose library contents.

`SelectLibrary { card }` records an eligible exact incarnation. `FinishLibrarySelection` is legal only after the minimum has been met and before ordering starts. Mandatory counts clamp to eligible cards; optional picks permit zero through the maximum. Empty libraries skip the window, and forced all-card picks need no permutation choices. For `bottom_any`, finishing enters `OrderLibraryBottom { card }`; the first ordered card is the highest in the bottom group. Every remaining card must be ordered exactly once. Random remainders offer no ordering action. A nonempty random window with no eligible picks still exposes the looked information privately and requires its sole Finish action before committing.

The private transaction keeps all looked cards in the library until selection and any bottom ordering finish. Partial choices change only the owned continuation. Final commit moves selected cards to hand as new incarnations, reveals only Oracle-required selections, and handles the remainder. Plain puts stay hidden. Revealed hand references persist until that incarnation leaves hand. Putting a card into hand is not drawing; putting the remainder into the graveyard is not milling. Bottom rearrangement preserves incarnations and the untouched middle of the library. Random ordering consumes the game's owned RNG only at commit.

Conditions and eligible characteristics are evaluated when the instruction begins and frozen in the private window. Lesson counts use the controller's graveyard; Flow State requires both card types there; infusion uses life actually gained this turn. Power filters use current characteristic-defining power in the library, excluding battlefield-only boosts. A source may leave before or during its independent ability's resolution. Morbius's cost already exiles the source before selection begins.

The suspended frame owns exact references and is cloned with the game. Each indexed action is revision checked; restore invalidates stale indices. Capacity failures restore the entire pre-action state, including a selected hand put followed by failing graveyard moves, the private frame, RNG, knowledge and revision. There is no process-global choice or random state.

## Reproduction

```sh
cargo test
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_library_selection -- 100
cargo run --release --bin benchmark -- 1000
```

Seventeen new native tests cover both seats and every complete body, all eight two-brid routes, both Riddle modes/payments, normal and flashback origins, zero/short libraries and empty filters, optional cardinalities, source departure, conditional counts, characteristic power, hidden/revealed destinations, exact bottom ordering, owned random replay, illegal actions and atomic capacity failure. The separate Python and public Standard acceptance ledgers verify effects through the shared library interface, with independent destination, incarnation, privacy and replay expectations.

The standalone core passes **1,249 Rust tests** (1,237 unit and 12 integration) and **238 compiler tests** and **156 fresh-release Python tests**, formatting, all-target Clippy with warnings denied, generated-catalogue verification and whitespace checks. All sixteen previous generated helper functions retain every old row; exact full-body mutation tests keep unknown residual instructions gated. The integrated branch passes 1,274 Rust tests (1,262 unit and 12 integration), 243 compiler/data tests and 158 fresh-release Python tests. It preserves all 435 earlier helper arms across 18 functions; the only added helper row is Resentful Revelation's flashback cost. All 70 factories and color masks are unchanged. Integrated library-selection Standard100 reproduces every metric below; offspring Standard100 retains 53,447 decisions/replays, with maximum logical state 21,316 bytes after the 16-byte addition.

The initialized logical state is **19,896 bytes**, sixteen bytes above the shared flashback milestone. The private frame is boxed and its initialized vectors are included in `state_bytes()`. A native 1,000-game smoke run executes 571,049 decisions at approximately 237,000 decisions/second, with 136.95 ns clone and 136.89 ns restore on this concurrently loaded host. Measurements include the existing native smoke policy and observation construction, exclude neural inference and JSON, and are neither an XMage parity claim nor a Java speed comparison.


The final public Standard workload completes 100 legal sixty-card/four-copy games naturally. Two runs reproduce **91,468 decisions**, 615 private frames and **9,142 replay boundaries**, with all fifteen complete bodies observed for both players. Independent ledgers verify 479 selected cards, 287 graveyard remainder cards, 189 revealed and 290 hidden hand puts, and 1,206 partial choices whose destinations remain uncommitted. There are 304 ordered and 368 random remainders; 314 later top-window comparisons independently confirm earlier bottom order. Zero-pick windows occur 397 times, including the explicit information step for no-eligible random remainders, and two windows are shorter than requested.

The workload exercises sixteen all-Lesson branches, twenty-one two-card Flow State branches and one life-gained two-card infusion branch. It witnesses seventeen already-exiled Morbius sources, fifteen already-dead Living Phone sources, thirty-one flashback casts and forty-three landfall boosts. Confounding Riddle reaches both modes and three paid versus four declined counter payments. All five creatures' printed power/toughness and keywords and Commune with Evil's additional life gain are checked. Maximum initialized logical state is **21,432 bytes**. Public Python checks additionally cover short/empty libraries, zero eligible cards, subsequent real draws after bottom ordering and six seeded random-order cases that observe both permutations through later private windows, with identical clone/restore results.
