# Exact life-loss history and guarded entries/steps

Issue: https://github.com/altf4/rustmage/issues/122

Three complete Standard bodies are admitted:

| Card | Complete implemented behavior |
| --- | --- |
| Bloodtithe Collector | {4}{B}, 3/4 Vampire Noble, flying; on entry, if an opponent lost life this turn, each opponent discards a card |
| Frilled Sparkshooter | {3}{R}, 3/3 Lizard Archer, reach and menace; enters with a +1/+1 counter if an opponent lost life this turn |
| Stromkirk Bloodthief | {2}{B}, 2/2 Vampire Rogue; at the beginning of its controller's end step, if an opponent lost life this turn, put a +1/+1 counter on target Vampire it controls |

[life-loss-admissions.json](life-loss-admissions.json) contains full Oracle
bodies and three Java source SHA-256 hashes at pinned XMage revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`, plus OpponentsLostLifeCondition,
OpponentsLostLifeCount and PlayerLostLifeWatcher supporting hashes. Catalogue
fingerprint: `15109682262b5ce6413f0af1c6d77d7da8120a81eb4aa7853483e42f3c44d2b4`.
Coverage is 1,102/5,164 Standard identities, plus six regression definitions,
49 token factories and one emblem; 4,062 identities remain unsupported.

One checked helper performs all concrete life loss and adds the positive amount
to the player's owned `u128` current-turn total. It routes combat/noncombat player
damage, fixed program losses, scaled drains, captured emblem-event losses,
activated life costs and pre-entry life payments. Creature/planeswalker damage,
zero losses and gains do not add loss history. Gaining life never cancels earlier
loss; a player can have net life gain and still qualify as having lost life.
Lifelink applies after the damage batch and retains every damage loss. Negative
life totals and wide exact integer/JSON representations remain supported.
Counterfactual funding adjusts its own virtual life only and cannot mutate the
game's history. Begin-turn resets totals; all snapshots and branches own them.
Capacity failure restores life, earlier writes and suspended payment flags.

The typed predicate reads the opposing player's total in the two-player engine.
It is relative to the ability's controller, including during an opponent's turn.
Entry counters are present before priority; life paid while applying the same
entry transaction can enable another incoming permanent's conditional counter.
Entry/step triggers use intervening-if evaluation and normal owned continuations.
Bloodtithe's opponent selects a private discard; the controller cannot see those
hand candidates. Stromkirk requires a controlled Vampire permanent, including
changelings; its target base does not impose an unprinted creature restriction
on future kindred permanents. Ordinary non-Vampires and opposing Vampires fail
legality. Rules references: CR119.3, 119.4, 601.2, 603.4, 614.12 and 702.15 in
the pinned [Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

The original candidates included Cindering Cutthroat, which remains unsupported:
its hybrid activated mana cost needs a separate complete implementation. Generic
activation discard costs still gate Flamecache Gecko. Hired Claw's attack event
and activation condition, Fireglass Mentor's cast permission and Gev's wider
replacement/ward body also remain gated. Exact predicate/entry vocabulary is
emitted only for fully accepted residual bodies. This adds no general
replacement/prevention, life-setting or multiplayer support.

Checks: 800 Rust tests (788 unit, 12 integration), 147 compiler/data and 106
fresh-release Python tests; catalogue consistency, rustfmt, Clippy with warnings
denied and diff checks. Eleven native regressions cover net gain versus actual
loss, zero/negative arguments, both controllers, simultaneous lifelink, nonplayer
damage, real fixed losses, activated/entry payments, funding purity, scaled and
captured losses, immediate counters, opponent private discard, Vampire/changeling
targets, wide JSON and atomic overflow including suspended entry choice flags.
Python tests replay both entry-counter branches, Bloodtithe's private discard
and Stromkirk's targeted counters through the in-process native library.

`cargo run --release --example standard_life_loss -- 100` runs two legal 60-card
Standard decks through 100 complete games. They deliberately contain no life
gain or setters, so positive before/after life decreases independently predict
all losses, including activated costs. Both viewers' totals and clone/restore
are checked. Results: 48,564 decisions, 145,692 public history checks, 20,660
clone/restore/replay boundaries and 20,668 maximum initialized state bytes.
All three entries, zero/positive Collector and Frilled entry branches, and both
guarded trigger kinds are exercised. Dedicated native regressions separately
cover damage/lifelink/net gains. The workload records trigger kinds starting
resolution; focused tests verify complete bodies and private continuations.

Observation schema **44** adds each player's public `life_lost_this_turn`.
ABI 1 and control protocol 2 remain unchanged. Two owned `u128` totals add 32
initialized bytes: measured initial logical state is 19,752 bytes. Accounting
excludes capacities, allocator/COW metadata, immutable catalogue data,
observations and external snapshots. The fresh 1,000-game Freeform baseline
produces 571,049 decisions at 385,609.22 decisions/s and 675.26 games/s;
clone 236.79 ns, restore 104.40 ns. This unpinned local sample includes smoke
policy/observation construction, excludes model/tensor/JSON work, and proves
neither a performance change nor XMage parity.

Full Standard and native RL/search acceptance remain incomplete and active.
