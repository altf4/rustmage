# Owned face-up exile play

Issue [#207](https://github.com/altf4/rustmage/issues/207), based on `092dda1101f9c795bd884986fbf4ca5277cf7262`, adds fifteen complete Standard bodies. This standalone catalogue supports **1,470 of 5,164** pinned Standard identities, plus six regression cards. Its fingerprint is `331f4bf4180aa83ef21713f93cccde4c8ed126ef31706179e1505f909f3e80bc`. The remaining 3,694 Standard bodies stay rejected.

| Card | Complete rule supported | Permission lasts through |
| --- | --- | --- |
| Alania's Pathmaker | 4/2; entering exiles top one | End of your next turn |
| Blazing Crescendo | Target creature +3/+1 this turn, then exile top one | End of your next turn |
| Clockwork Percussionist | 1/1 artifact creature, haste; dying exiles top one | End of your next turn |
| Cori Mountain Monastery | Tapped unless you control a Plains or Island; tap for red; {3}{R}, tap to exile top one | End of your next turn |
| Count on Luck | Your upkeep exiles top one | This turn |
| Crimson Operative | 3/2 artifact creature, prowess; entering exiles top one | End of your next turn |
| Diversion Specialist | 4/3 menace; {1}, sacrifice another creature or enchantment to exile top one | This turn |
| Emberheart Challenger | 2/2 haste, prowess; first owned spell/ability targeting it each turn exiles top one | This turn |
| Equilibrium Adept | 2/4; entering exiles top one; second spell each turn grants double strike this turn | End of your next turn |
| Gundabad Opportunist | 4/2; entering exiles top one | End of your next turn |
| Kulrath Zealot | 6/5; entering exiles top one; basic landcycling {1}{R} | End of your next turn |
| Outlaws' Fury | Owned creatures +2/+0 this turn; if you control an outlaw, exile top one | End of your next turn |
| Seize Opportunity | Choose exile top two, or give up to two target creatures +2/+1 this turn | End of your next turn |
| Sizzling Changeling | 3/2 changeling; dying exiles top one | End of your next turn |
| Zuko, Exiled Prince | Legendary 4/3, firebending 3; {3} to exile top one | This turn |

The compiler accepts only complete fixed top-one/top-two clauses granting ordinary paid play of your own face-up cards. It retains every residual keyword, activation, cost, mode and trigger qualification. Outlaws' Fury checks the live outlaw condition at resolution; Cori checks the battlefield before its own entry. Singular and plural grants must match the exiled batch. Unknown trailing rules gate the entire card. Choose-one exile (Heroes' Hangout, Riverwheel Sweep and Strongbox Raider), next-end-step durations, theft, free casting, face-down cards, plot/warp, extra land plays and source-linked durations remain gated.

The [provenance manifest](exile-play-admissions.json) contains the complete fifteen Oracle bodies and **149 per-card SHA-256 records across 52 Java files**, checked against both pinned revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7` and the local checkout. Critical logic includes ExileTopXMayPlayUntilEffect, PlayFromNotOwnHandZoneTargetEffect, exact FixedTargets, ContinuousEffectImpl's next-turn tracking, the Valiant/Flurry bases, costs, cycling, outlaw predicates and firebending. Rules authority is the [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf), especially 117.1a–b, 305.1–3, 400.7, 406.3, 601.2, 611.2a and 514.2–3a.

## Native and RL contract

Observation schema **57** adds public `exile_permissions: Vec<ExilePlayPermission>`. Each record owns `{ object: ObjectRef, player: Player, expires_turn: u32 }`. All supported exile is face up; both players see the same rights and exact public card incarnation. Library contents remain hidden before exile. A record grants permission, not immediate castability: ordinary turn/step/priority, targets, land allowance and funding still determine legal actions.

`AnnounceExile { card }` enters the same staged casting path as hand casting: optional offspring declaration, selected hybrid cost, modes/targets, mandatory additional costs (including forage), mana reductions/funding and actual payment. The normal printed mana value, colors, characteristics and cast triggers remain unchanged. `SpellView.cast` uses `origin: Exile, method: Normal`; this operation never sets flashback's stack-departure marker. Resolved, countered, bounced or otherwise removed spells use their ordinary destinations. Hand-only direct cast shortcuts cannot bypass the permission path. Existing `PlayLand { card }` is offered for a permitted exiled land only during its owner's main phase with an empty stack and an unused land allowance. Entry replacements and life choices run normally. Exile life-choice lands add no stale hand-knowledge record.

The sorted registry is a small owned State vector. Legal checks use binary search by CardId and compare owner, exact generation and expiry; cleanup/central zone transitions retain sorted order. Every exile exit removes the right before the new incarnation is created. Exile exit/reentry cannot inherit a grant. Permissions have no source dependency, so removing or reincarnating the granting source has no effect. Multiple grants for different cards coexist. The complete registry participates in snapshots, clone/restore, transaction rollback and logical byte accounting. No per-object field, global mutable registry, trait graph, action-space cap or RNG consumption is added.

The current engine has two players with alternating turns and no supported extra/skip-turn effects. `expires_turn` names the inclusive absolute turn whose cleanup ends the permission. A this-turn grant uses the current turn; a next-owner-turn grant uses current turn +2 during the owner's turn and +1 during the opponent's turn. These checked deadlines exclude the owner's current turn, even for grants during that turn's end step or a priority-bearing cleanup. At the relevant cleanup's CR514.2 action, the record expires before any resulting priority. A new this-turn grant during subsequent cleanup priority survives only until the next cleanup pass, as CR514.3a requires. Any future turn-scheduling extension must replace this explicit alternating-turn calculation with owner-relative scheduling and extend these boundary tests before admission.

Initial 60-card-per-player logical state is **19,912 bytes**, an actual **16-byte total layout increase** from the measured 19,896-byte parent. The raw empty Vec header is 24 bytes; existing layout padding means the total-State increase differs. Each active permission is 24 initialized bytes; exile also removes one 4-byte library ID. Cloning copies only the small active registry in addition to existing state/COW handles. The public 100-game fixture reaches 21,196 logical bytes. Accounting excludes spare capacity, allocator overhead, immutable definitions, observations, retained snapshots and inference.

## Integration with filtered library selection

On merged main `bc219a65ffbe516cccc5286adf86e75677442e23`, the combined catalogue supports **1,482 / 5,164** Standard cards, with **3,682** gated identities, six regression cards, **72 token factories** and one emblem. Its fingerprint is `4ac5e5f414465d3569689ec39beb9b48ce15ad8216ed6206b763be97c2ceba9a`. Schema remains this feature's **57**; C ABI remains 1 and control protocol 2.

Canonical integration comparison changes exactly the same fifteen previously unsupported definitions and preserves **5,241** other definitions, all **72** prior factories and metadata, **446** prior numeric typed helper arms, **5,256** old color masks and **5,183** ordinary IDs. No immutable helper arm or factory is added. The standalone comparison below includes 459 total match arms, including wildcard/default cases; its corresponding numeric typed-arm count is 442.

Integrated validation passes **1,314 Rust tests** (1,302 unit plus 12 integration), **251 compiler/data tests** (294.327 seconds) and **161 fresh-release Python tests** (85.670 seconds), plus all-target warnings-denied Clippy, formatting, catalogue and whitespace checks. The exile Standard100 run reproduces every standalone vector and **60,677 decisions/replays**; the adjacent filtered-selection and forage workloads retain **71,080** and **65,729** decisions/replays, respectively. Their maximum logical sizes are 20,720 and 21,020 bytes after the registry's layout increase. The combined initialized state is **19,912 bytes**; its 1,000-game smoke run measures about 310,000 decisions/s, 126 ns clone and 128 ns restore under concurrent host load. These timings carry the same limited-catalogue and non-comparison caveats as the standalone measurements.

## Validation

```sh
cargo test
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release --lib
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_impulse -- 100
```

Fifteen new native groups cover all fifteen real grant effects and permitted spell casts in both seats; source departure; wrong actor/generation; exile reentry; ordinary instant/sorcery timing; normal land limits; every end-step/cleanup/active-player deadline; all two-brid equivalents; paid/unpaid offspring; real discard and all three forage routes; actual Stormcatch Mentor reductions; printed characteristics; actual Counterspell and all ordinary stack exits; capacity failure rollback; RNG-preserving empty grants; private cycling search/shuffle/reveal; Valiant's first event, Flurry's second cast, Firebending and complete modal/outlaw/entry residuals. Actions are cloned, restored and replayed, and old revisions are rejected.

Complete checks pass **1,302 Rust tests** (1,290 unit plus 12 integration), **248 compiler/data tests** (226.383 seconds), and **160 fresh-release Python tests** (56.791 seconds), plus formatting, all-target Clippy with warnings denied, generated-catalogue verification and whitespace checks.

Compiler regressions exercise complete fifteen-card admission, arbitrary residual rejection, excluded mechanics, singular/plural and duration boundaries, and duration-sensitive catalogue fingerprints. Four older negative lists now omit newly completed Sizzling Changeling, Emberheart Challenger, Zuko and Diversion Specialist; all their other exclusions remain enforced. Independent immutable comparison proves exactly fifteen changed Definition rows, **5,240 unchanged other rows**, unchanged **459 helper arms**, all **71 existing token recipes**, the name index and all **5,183 ordinary IDs**. No factory is added or reindexed.

The direct Python acceptance loads a fresh release library and plays all fifteen effects through its public interface, checks face-up exact permissions, paid spell and land plays, expiry and late denial, normal stack destinations, source independence and full replay/stale-index behavior. Its Freeform setup fixtures deliberately repeat cards to reach each boundary; they do not claim Standard deck validation.

The separate public `standard_impulse` example uses legal 60-card/four-copy Standard decks and actual offered actions. It independently checks all fifteen grant clauses and each deadline, library counts, printed entry stats/keywords, Prowess, Flurry, Firebending, both boost modes, outlaw team boosts, actual spell/land permission consumption, source departure, normal stack exits and expired-action denial. Every decision is cloned/restored/replayed; it consumes no hidden engine state. Its fixture guard is not an engine action limit.

Two final reproducible 100-game runs match exactly: each completes **60,677 decisions/replays**, with grants in the table's order `[18,15,218,97,72,19,10,5,18,17,13,4,30,193,39]`; verified residual units `[13,9,25,27,12]` for Prowess, Flurry, Firebending, targeted boosts and outlaw boosts. It performs **225 paid exile spell casts**, **326 exile land plays**, with owner counts `[250,301]`, **187 expirations** and **319 plays after the granting source left**. Both runs finish naturally with the same catalogue and all witnesses.

A separate native 1,000-game smoke benchmark executes 571,049 decisions at approximately 375,160 decisions/second, with 124.70 ns clone and 122.00 ns restore on this host. The measured original parent initializes 19,896 logical bytes; this implementation initializes 19,912. These host-dependent timings include smoke policy and observation construction, exclude neural inference/tensor encoding/JSON, and do not claim XMage parity or a Java speed comparison.
