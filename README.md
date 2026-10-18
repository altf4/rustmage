# Rustmage

Rustmage is a Magic: The Gathering rules engine written in Rust for reinforcement
learning and game-tree search. It runs in the same process as your training or
search code, providing a simulation environment for developing and evaluating
bots that play Magic.

A program creates a game, reads a player's observation, and chooses an action.
Rustmage enforces the rules, resolves the consequences, and advances to the next
decision. Each player's observation contains the information available to that
player. A fixed random seed and the same sequence of decisions reproduce the
same game.

Search algorithms can save a position, create independent branches, explore
possible continuations, and restore an earlier state. Branches share memory
where possible while keeping their changes isolated. This makes branching and
rollback central parts of the engine, supporting workloads such as Monte Carlo
tree search.

The engine is available through a native Rust API and in-process Python bindings.
Your application supplies the bot, learning algorithm, and reward function;
Rustmage supplies the game rules and state transitions.

Current support includes all **517 unique cards in the Foundations inventory**,
including its Beginner Box and Starter Collection, with rules tests and complete
games using Foundations-only decks. The long-term goal is complete Standard
coverage; implementation of the wider card pool is still in progress. Standard
games reject cards whose rules have not been implemented.

Card behavior is authored directly in Rust, organized by set, and built on shared
abilities, effects, targets, and costs. XMage's implementations and authoritative
Oracle text guide that work. Gameplay executes deterministic Rust code.

To explore the project:

- [Rust examples](examples/) show how to drive games and branch game state.
- [Python bindings](python/rustmage.py) expose the engine to Python applications.
- [Card implementation guide](docs/direct-card-authoring.md) explains how to add card behavior.
- [Foundations validation](docs/foundations-acceptance.md) describes the coverage and testing process.
