# Chandra, Torch of Defiance: empty library

The pinned XMage implementation at revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7` skips the +1 damage when its
controller's library is empty. The Oracle text says to deal 2 damage to each
opponent if the player does not cast the exiled card. With no card to cast, the
Rust implementation follows that printed condition and deals the damage.

The empty-library, paid-cast, decline, copy and damage cases are exercised in
`src/engine_fra_chandra_torch_tests.rs`.
