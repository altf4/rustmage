# Committed discard history and scaled draws

Issue: https://github.com/altf4/rustmage/issues/128

Two complete Standard bodies are admitted:

| Card | Complete implemented behavior |
| --- | --- |
| Misty Knight, Hero for Hire | {1}{R}, legendary 3/1 Human Detective Hero; {2}, tap and discard a card: draw a card for each card its controller discarded this turn |
| Green Goblin, Revenant | {3}{B}{R}, legendary 3/3 Goblin Human Villain; flying and deathtouch; when it attacks, discard a card, then draw a card for each card its controller discarded this turn |

Full Oracle text and pinned Java card/watcher SHA-256 hashes are in
[discard-history-admissions.json](discard-history-admissions.json), at XMage
revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Catalogue fingerprint:
`74e5d02190a48e82a4446c848e655984a6e79a1b1fe71bfa8f64fc3d223a8d87`.
Coverage is **1,114/5,164** Standard identities, six regression definitions,
49 token factories and one emblem; **4,050** Standard identities remain unsupported.

Each player's checked `u64` discard history is updated at `discard_batch`, using
only committed moved objects. The complete batch updates both players before
zone/discard observers run. The history is independent of any watcher permanent,
includes cycling, mandatory spell/activation costs, resolution effects and
cleanup, and resets for both players at each turn start. Reentering and then
discarding a card again counts the new discard. Ordinary hand-to-graveyard moves,
exile, milling, drawing, selecting a pending cost or querying legal actions do
not increment it. Wider discard replacements remain gated with their bodies.

`AmountValue::CardsDiscardedThisTurn` reads the captured source controller's
live total when its draw instruction executes. Misty's paid discard counts;
later discards while its ability waits on the stack also count. Source departure
does not erase history or the captured ability. Green Goblin's private discard
choice suspends its resolution; after commitment the following draw reads the
updated total. With an empty hand, it discards nothing and draws the earlier
total, including zero. The existing authenticated full/short self-name parsing
already handles Green Goblin's printed attack trigger; no new alias is needed.
Unsupported event-subject/alone-attacker rules, including Jiang Yanggu, Alone,
and unknown residual clauses remain gated.

Rules references: CR601.2h, 602.2, 701.9 and 608.2 in the pinned
[Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Observation schema **47** adds public per-player `cards_discarded_this_turn`
totals. Pending choices still hide opposing hands and cost selections. ABI 1
and control protocol 2 remain unchanged. Snapshots/clones own both totals and
all resolving/announcing choices. Checked overflow returns `CapacityExceeded`
and restores the entire action and revision. A native test crosses 2^60 for
exact JSON projection and exhausts `u64` during an activation cost to verify
full rollback of the card, source tap, mana and pending announcement.

Initial logical state is **19,784 bytes**, 32 bytes above the preceding stage
including player-struct alignment. Logical initialized accounting excludes
capacity, allocator/COW metadata, statics, observations and external snapshots.

Validation passes **826 Rust tests** (814 unit and 12 integration), **153
compiler/data tests** and **110 freshly built release Python tests**, formatting,
Clippy with warnings denied, catalogue and diff checks. Eight new native tests
cover mixed-player batches, cause-specific updates, private/resumable discards,
late source entry, response-time discards, source departure, empty hands,
cycling/spell/activation costs, cleanup, reset, exact wide JSON and overflow.
Two new public binding tests branch/restore both cards' native continuations.

`cargo run --release --example standard_discard_history -- 100` plays 100
complete games with legal identical 60-card Standard decks. An independent
model counts hand-to-graveyard transitions only on paid-cost/discard actions
and resets on turn changes; the workload's other spells do not move hand cards
to the graveyard. It checks both viewers' totals after every decision. Results:
**101,093 decisions, 404,372 history checks, 512 scaled draw checks, 57,636
clone/restore/replays and 21,228 maximum initialized state bytes**. Both bodies
enter and resolve; scaled histories cover 0 through 6 and observed causes cover
cost, effect and cleanup. Draw expectations cap successful draws by the library
size, including terminal empty-library attempts. An external per-turn set limits
repeated policy activations; the engine still offers legal repeats.

A fresh 1,000-game Freeform baseline yields 571,049 decisions at 396,420.99
decisions/s and 694.20 games/s, clone 323.43 ns and restore 104.83 ns. This
unpinned timing sample includes smoke policy and observation construction and
excludes model inference, tensor encoding and JSON. It establishes neither a
performance change nor XMage parity.

Full Standard and native RL/search acceptance remain incomplete and active.
