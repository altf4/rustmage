# Self-target events and Valiant

Issue: https://github.com/altf4/rustmage/issues/134
Target: Standard identities frozen October 15, 2026. This stage admits eight
whole bodies, taking coverage to **1,130/5,164**; **4,034 remain unsupported**.
The catalogue has 1,136 executable identities including six regression cards,
49 token factories and one emblem. Observation schema stays 48, ABI 1 and control
protocol 2. No persistent state fields or network service are added.

## Full bodies and provenance

`self-target-admissions.json` records exact pinned Oracle text and SHA-256 hashes
of the eight Java implementations and shared targeting/Valiant helpers at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

| Card | Complete body |
| --- | --- |
| Cactarantula | Own-Desert generic spell discount, reach, optional opponent-target draw |
| Old Fat Spider | Reach, power-at-most-two blocking prohibition, opponent-target draw |
| Ruric Thar, Biomagus | Flying, two separate prowess triggers, opponent-target draw |
| Mouse Trapper | Flash, first-own-target trigger tapping an opposing creature |
| Nettle Guard | First-own-target source +0/+2, paid source-sacrifice artifact/enchantment destruction |
| Seedglaive Mentor | Vigilance, haste, first-own-target source +1/+1 counter |
| Veteran Guardmouse | Hybrid spell payment, first-own-target +1/+0/first strike and private scry 1 |
| Whiskerquill Scribe | First-own-target optional private discard followed by draw only if paid |

Tenured Concocter, Altanak, Pawpatch Recruit, Surrak, Emberheart Challenger,
Flowerfoot Swordmaster and Whiskervale Forerunner retain unsupported dependencies.
Residual text, global/other-creature targeting events, ambiguous later pronouns,
unsupported first-time predicates, copies, retargeting and control changes remain
gated. Formal Valiant is an ability word, not an intrinsic keyword flag.

## Event timing and identity

The existing target-binding hook now dispatches `SelfTargeted` events separately
from Ward. Matching uses the captured stack item's controller and kind; immutable
IR supports own/opponent/any and spell-only or spell-or-ability predicates. This
stage's complete bodies use opponent spell-or-ability and first-own-target forms.
Ward still triggers only for opponents and retains its exact related-stack link;
ordinary self-target triggers have no such link.

Binding occurs during target selection, before costs. A second occurrence of
the same ObjectRef in another group on that stack item is deduplicated. A later
spell or ability is a new event. Trigger ordering and announcement resume through
the existing owned continuations. Draw triggers survive source departure, while
source modifications apply only to the captured incarnation.

The five first-own-target abilities reuse `(ObjectRef, ability)` per-turn trigger
use. The first qualifying event consumes the marker even if its effect is declined,
its trigger is countered, or its mandatory target cannot be announced. Opposing
targets do not consume it. Beginning every turn resets the limits; leaving and
re-entering creates a new object identity. This equivalence depends on the admitted
pool having immutable abilities and no control changes; ability acquisition,
copying, control changes or first-time intervening conditions require a new audit.

Compiler pronoun binding is narrow: an initial source-growth clause or a whole
source-counter sentence has this self event as its antecedent. Later target-bound
pronouns are not guessed. Ruric's short legendary name is authenticated against the
card's own identity; both occurrences of prowess remain separate abilities.

The [official Bloomburrow release notes](https://magic.wizards.com/en/news/feature/bloomburrow-release-notes)
specify first targeting by a spell or ability **you control**, and resolution
before that targeting stack item. This is the rules authority for Valiant. The
pinned Java shared watcher counts targeting events without that controller filter;
Rust tests deliberately cover an opposing target preceding the first own target.
Also see CR 113.7a, 400.7, 601.2c–i, 602.2, 603.2c/e and 603.3.

## Validation and search evidence

- 849 Rust tests (837 unit and 12 integration), including nine new rules tests.
- 159 compiler/data tests, including two new full-body and fail-closed methods.
- 114 fresh release Python tests, including one method exercising all eight
  complete target triggers through real games and the native C ABI.
- `cargo fmt --check`, all-target Clippy with warnings denied, catalogue `--check`
  and `git diff --check` pass.

Native tests cover grouped deduplication; spell, activated and triggered item
kinds; own/opponent predicates; first-event reset/re-entry/countered triggers;
missing mandatory targets; declaration-before-payment timing; atomic rejection;
all eight other printed clauses; private continuation clone/restore and stale
revision rejection. Synthetic events isolate kind/dedup cases without claiming
those fixtures are real multi-target printed spells. The fresh Python test reaches
actual stack triggers, checks the relevant outcome, and clones/restores/replays
all eight cases including optional discard/draw and scry privacy.

`cargo run --release --example standard_self_target -- 100` reproduces:

- 100 completed legal 60-card Standard games, all eight entries and target triggers;
- 57,184 decisions and 1,100 independent target-event checks;
- 347 private discard/scry choices and 27,135 clone/restore/replays;
- 21,728 maximum logical initialized state bytes.

The external event model counts qualifying controller/target events, deduplicates
per-item identity, resets first-event markers per turn, compares announced trigger
counts, and checks the public consumed markers in both directions. Mouse Trapper
triggers with no legal opposing creature are unobservable as stack items but must
still consume their public first-event marker. Real spell and equip ability routes
are exercised, along with Nettle Guard's destruction activation. The finite smoke
policy uses an external once-per-turn activation preference; legal engine actions
remain unrestricted by that preference.

The initial Freeform benchmark remains **19,784 logical bytes**, excluding
allocator/capacity/COW metadata, static definitions, observations and externally
held snapshots. A fresh unpinned 1,000-game sample yields 571,049 decisions,
403,952.88 decisions/s, 707.39 games/s, clone 117.84 ns and restore 100.54 ns.
This includes native smoke policy/observation construction and excludes model
inference, tensor encoding and JSON. It proves neither a performance improvement
nor XMage parity. Full Standard and broader native RL/search acceptance remain
incomplete and active.
