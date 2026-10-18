# Foundations acceptance

Issue [355](https://github.com/altf4/rustmage/issues/355) records merged coverage,
remaining obligations and measured results for the first full-set milestone.
Foundations began with 314 admitted identities; that historical subset is not
the acceptance target. All-Standard remains the long-term goal.

`data/sets/fdn.json` freezes all 517 physical Oracle identities, including the
Beginner Box and Starter Collection. Its 771 XMage printings count cosmetic
variants once. Reprints share one typed implementation; the directory owning a
card does not define its set membership. Authenticate the complete inventory with:

```sh
python3 -m scripts.set_status FDN --check
```

Every printed obligation must work before a card is supported: costs, targets,
choices, triggers, static and replacement effects, generated tokens and relevant
source/controller/zone changes. Direct definitions and their source records live
in `src/card_definitions/<set>/`; native rules tests exercise paid actions,
interactions, hidden information, rejected actions and owned state. The final
eleven-card integration is [PR454](https://github.com/altf4/rustmage/pull/454).

`tests/foundations_games.rs` contains complete games with Standard-legal,
Foundations-only decks, covering all colors and mechanic-specific workloads.
The broad inventory matrix requires all 517 identities to be supported and
included in its decks. Inclusion alone does not prove that a card was cast or
that every obligation executed; targeted rules tests and the natural-game
mechanic observers supply that evidence. The existing policy pays actual costs,
checks terminal outcomes, rejects illegal actions and verifies raw/index replay.

Run the frozen integrated source through the full validator:

```sh
python3 scripts/validate.py --release
```

It checks source provenance, formatting, native tests, all game matrices, strict
Clippy and the release build. Retain its report, resource measurements and exact
shared-library hashes; verify positive library admission/owned-state checks and
two independent nonauthor reviews of the final published batch before merging.
The tracking issue and final PR record actual results, including any failures.

The bounded `owned_rollouts::foundations_private_choice_owned_sibling_rollouts`
workload starts from naturally reached private library choices in legal
Foundations games in both seats. It compares distinct multi-action raw/index
sibling continuations after parent disposal, then restores and replays the saved
choice. Run its optimized measurement through the resource guard:

```sh
python3 scripts/resource_guard.py -- cargo test --release --test foundations_games owned_rollouts::foundations_private_choice_owned_sibling_rollouts -- --exact --nocapture
```

Reported branch/decision rates include cloning, snapshots and both players' JSON
observation equality checks. State bytes exclude unused capacity and allocator
overhead. This measures trusted simulator verification, not hidden-information
MCTS or neural inference. The separate Freeform smoke benchmark is a limited
regression workload and must not be presented as full-set or XMage parity
performance. Record optimized results with their exact source revision in issue
355 and the final PR.

Accept Foundations only after all 517 implementations and the combined rules,
game and owned-search checks pass with no known outstanding obligations. Then
**pause for the user to regroup before selecting or implementing another set**.
Do not mark the all-Standard objective complete at this milestone.
