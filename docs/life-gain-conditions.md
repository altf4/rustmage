# Life-gain conditions and opposing-creature debuffs

Issue: https://github.com/altf4/rustmage/issues/136
Standard pool: October 15, 2026. This stage implements thirteen whole bodies,
bringing executable Standard coverage to **1,143/5,164**, with **4,021 unsupported**.
There are also six regression identities, 49 token factories and one emblem.
Observation schema stays 48, ABI 1 and control protocol 2. No persistent GameState
field or server is added; existing positive life-gain history is reused.

## Bodies and provenance

`life-gain-condition-admissions.json` preserves exact pinned Oracle text and
Java SHA-256 hashes at revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.
All thirteen implementations and the shared gain condition/watcher were read.

| Card | Complete body |
| --- | --- |
| Efflorescence | Always two counters; resolution-time conditional trample/indestructible |
| Old-Growth Educator | Vigilance/reach; entry trigger conditionally adds two source counters |
| Poisoner's Apprentice | Mandatory opposing creature target; resolution-time conditional -4/-4 |
| Tenured Concocter | Vigilance, optional opponent-target draw, continuous source +2/+0 |
| Thornfist Striker | Ward 1, continuous own-creature +1/+0 and trample |
| Ulna Alley Shopkeep | Menace, continuous source +2/+0 |
| Withering Curse | Exactly one branch: global -2/-2 or destroy all creatures |
| Ambush Gigapede | Flash, opposing entry -2/-2 |
| Burrog Befuddler | Flash, opposing entry -1/-0 |
| Cogwork Wrestler | Artifact creature, flash, opposing entry -2/-0 |
| Front Porch Sentries | Opposing death-trigger -1/-1 |
| Grim Bauble | Opposing entry -2/-2; paid tap/source-sacrifice private surveil 2 |
| Humbling Elder | Flash, opposing entry -2/-0 |

Foolish Fate, Lumaret's Favor, Enlightened Confidant, Follow the Lumarets, Lunar
Convocation, Moseo, Ragost and Tragedy Feaster retain unsupported dependencies.
Infusion is an ability word; it does not create an intrinsic keyword.

## Three timing contexts

`StepGuard::GainedLife` reads whether the captured controller's existing
`life_gained_this_turn: u128` is positive. Zero/negative proposed gain does not
count, opposing gains do not count, and subsequent life loss does not erase a
gain. Both players' histories reset at every turn start, regardless of the active
player. The check never narrows wide history into a signed amount.

A conditional effect uses an `If` program instruction evaluated at resolution.
Old-Growth Educator's and Poisoner's Apprentice's triggers therefore occur even
without prior life gain. Apprentice must choose a legal opposing creature before
resolution; gaining life in response can enable its -4/-4. The compiler also
supports the separate intervening-if form through the existing trigger guard
checks. Future whole-body dependencies still gate any such cards not yet complete.

Continuous source stats use a pure zero-or-one `LifeGainOccurredThisTurn` stat
expression. Global additive modifiers and positive layer-6 grants carry an
immutable `requires_life_gain` flag. Effective stats/keywords query the source
controller's live history, preserving existing layers/counters/attachments and
source removal behavior without a derived cache. Multiple Thornfists stack power
bonuses; their keyword grants commute. Broader dependency, setting, removal,
copying and control changes remain gated.

Virtual mana funding copies each player's gain predicate for matching conditional
grants. Current virtual moves cannot gain life: mana programs remain single
production instructions, and payment life loss does not alter prior gain history.
The funding memo key need not duplicate a value constant across that search.
A dedicated native test compares virtual/live keyword queries, including wide
history and an opposing source.

Efflorescence's explicit temporary grant uses its spell controller's history,
including when targeting an opposing creature. Its permanent counters always
apply. Withering Curse checks its condition on resolution and executes only the
selected branch. Indestructibility does not prevent death from zero toughness;
it does prevent the destroy branch. The compiler authenticates whole conditional
sentences and keeps unsupported guards, residual clauses and ambiguous pronouns
gated. An explicit opponent-creature growth target remains distinct from an
unrestricted or friendly target.

The [official Secrets of Strixhaven release notes](https://magic.wizards.com/en/news/feature/secrets-of-strixhaven-release-notes)
confirm the entry-trigger/target timing for Old-Growth Educator and Poisoner's
Apprentice, and the self-target behavior for Tenured Concocter. Also see CR 119,
603.4, 608.2, 611.3, 613, 704.5f and 702.12.

## Validation and limits

- 858 Rust tests: 846 unit and 12 integration, including eight new rules tests
  and one virtual-funding regression.
- 161 compiler/data tests, including two new whole-body/guard methods.
- 117 fresh release Python binding tests. Three new methods cover three
  continuous bodies, both branches of both spells, and eight entry/death bodies;
  the existing self-target method additionally exercises Tenured's optional draw.
- Formatting, all-target Clippy with warnings denied, catalogue consistency and
  diff checks pass.

Native rules fixtures cover net life loss, zero/negative/opposing gains,
`u128::MAX` history, reset, source removal/re-entry, late response gains, mandatory
targets while false, atomic illegal-action rejection, all thirteen other clauses,
indestructible/zero-toughness branch differences and owned private surveil replay.
Python cases independently clone/restore/replay through the native C ABI.

`cargo run --release --example standard_life_gain_conditions -- 100` runs legal
60-card Standard decks containing all thirteen bodies, Moment of Triumph,
Healer's Hawk and Giant Growth. Even/odd seed policies explicitly prefer enabled
or baseline branches; that preference is external to engine legality. Coverage
assertions require both outcomes of all four conditional effect bodies and all
seven opposing debuffs, plus gain transitions and Grim Bauble surveil.

The workload's public-state oracle checks live history/grants, conditional power
changes, counters, keyword grants, chosen wipe branches and surviving debuff
recipients. It distinguishes actual resolution from Ward countering, accounts
for lifelink and source changes when combat resumes, and tracks opposing
Efflorescence recipients separately. Native fixtures isolate lethal debuffs and
private-body details when later automatic combat would obscure the intermediate
state. Every stack/announcement/resolution boundary and periodic other actions
are cloned and restored, including stale-index rejection.

The deterministic 100-game result was reproduced twice: **61,031 decisions**,
**167,852 history/body checks**, **317 gain-transition stat checks**, **204 private
Grim Bauble surveil choices**, **26,183 clone/restore replays**, and **20,720 maximum
logical initialized bytes**.

A fresh unpinned 1,000-game Freeform benchmark yields 571,049 decisions,
401,710.40 decisions/s, 703.46 games/s, clone 131.52 ns and restore 110.49 ns.
Initial state remains **19,784 logical bytes**, excluding capacity/allocator/COW
metadata, static definitions, observations and external snapshots. The sample
includes native smoke policy and observations, excludes model inference, tensor
encoding and JSON, and establishes neither a speedup nor XMage parity.

Full Standard and broader native RL/search acceptance remain incomplete and active.
