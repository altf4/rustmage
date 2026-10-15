# Independent review: typed counters and stun replacement

PR: https://github.com/altf4/rustmage/pull/17
Initial implementation: 9098d72. Reviewed fix: 79ac1c8.

Two independent read-only subagents reviewed this PR:

- Rules: all eight newly admitted definitions against frozen Oracle text,
  counter characteristics/cancellation, zone reset, incarnation binding, stun
  replacement and cleanup state-based actions.
- Library: owned counter/snapshot state, schema 7, privacy and state accounting,
  complete-text compiler admission, native/C/Python integration and replay tests.

Both found the same P2: the added cleanup priority window could draw cards, then
skip discarding during the next cleanup. The fix routes both initial and repeated
cleanup through the full hand-size check. The regression exercises a Summit
Sentinel death drawing the eighth card, mandatory discard, turn advancement, and
snapshot replay at both cleanup boundaries. Both independently verified the fix
and reported no remaining actionable findings.

Validation: 84 Rust tests, 19 compiler/importer tests, catalogue consistency,
rustfmt and Clippy. Release build and ten Python tests pass. Author self-play
completed 100 supported Standard counter games: 52,105 decisions and 647 target
selections. The library reviewer also completed 20 native counter games.

Executable coverage is 114/5,164 Standard identities. Issue #16 remains open for
tokens, entry counters and broader counter operations. This review does not
establish full Standard rules support or full RL performance acceptance.
