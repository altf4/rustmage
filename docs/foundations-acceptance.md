# First full-set milestone: Foundations

Issue [355](https://github.com/altf4/rustmage/issues/355) tracks the first complete
set. All-Standard remains the long-term goal, reached one verified set at a time.
Foundations has the highest existing admission fraction: 314 of 517 unique cards
at the selection baseline. The remaining 203 cards are gated. Those admission
flags are implementation claims; inherited cards also need rules evidence.

`data/sets/fdn.json` freezes all 517 physical Oracle identities, including the
Beginner Box and Starter Collection. Its 771 XMage printings count cosmetic
variants once. Reprints share one typed implementation; the directory owning a
card does not define its full set membership. Authenticate and report status with:

```sh
python3 -m scripts.set_status FDN --check
```

Finish existing core dependencies, then use `rustmage-implement-set` for FDN-only
batches. Every printed obligation and all faces, token recipes, costs, targets,
choices, triggers and effects must work before a card is admitted. Missing shared
mechanics include copying, persistent chosen colors/types, extra combats,
prevention/replacement, target changes and alternate game-result rules. Each new
mechanic receives interaction tests and owned branch/restore checks.

`tests/foundations_games.rs` starts the end-to-end matrix with twenty naturally
played Standard games: five single-color decks, two seeds, both starting seats.
It reuses the existing policy, pays normal costs and checks terminal outcomes,
combat, tokens, triggers, illegal-action atomicity and raw/index owned replay.
Its 45 nonland identities and basics are an initial regression workload. Passing
this matrix alone cannot certify the set. Extend gameplay coverage with the set's
mechanics and interactions as missing cards land; preserve targeted rules tests
for every card, including inherited implementations.

Accept the set only when all 517 cards have complete tested behavior, remaining
obligations are resolved, and full-game evidence exercises the implemented set
with legal decks and meaningful outcomes. Use two independent nonauthor reviews
for final PRs. Then invoke the skill again to choose and complete the next full
set. A metadata count, partial implementation or one smoke deck cannot waive this
acceptance target.
