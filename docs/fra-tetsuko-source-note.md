# Tetsuko Umezawa, Pursuer: blocker controller

At pinned XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`,
`TetsukoUmezawaPursuer.java` records the blocker's controller when it blocks.
The Rust implementation uses the exact blocker's current controller as the
ability resolves, or its last known controller if it has left the battlefield,
following Comprehensive Rules 608.2h and 608.2k. The blocker's qualifying
characteristics and opponent relationship are checked when it blocks.

Rules reference: https://media.wizards.com/2026/downloads/MagicCompRules%2020260619.pdf

The control-change and departure cases are covered by
`src/engine_fra_tetsuko_pursuer_independent_tests.rs`.
