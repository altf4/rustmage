# Independent review: counter groups and large combat choices

PR: https://github.com/altf4/rustmage/pull/87
Implementation initially reviewed: 3de998abce3f782f6caadb7385f8af69997893f0.
Final implementation reviewed: 56521839493baabc5ae36fe6c18627587582a57e.

Two independent read-only subagents reviewed all 12 printed admissions, live
recipient filters/source generations, non-targeted group counters, distinct
multi-target legality, doubling, deferred SBAs/death events, exact compiler gates,
state ownership, numeric consumers and native/Python observations. Neither edited
files, mutated Git or posted externally.

The reviews found two actionable issues before merge:

- Repeated legal Zimone doubling caused signed 32-bit base-plus-counter stats to
  wrap in release, incorrectly killing a creature. Counters now use unsigned
  64-bit storage and derived stats/damage/life use checked signed 128-bit values.
  Captured damage totals and exact JSON serialization/decoding were widened too.
  Boundary, huge damage/lifelink and full public 40-doubling regressions cover it.
- Large multi-blocker damage enumerated every integer up to power, making trillion
  amounts unusable in observation and validation. The complete action space now
  contains an exact inclusive damage range. Concrete indexed choices are bounded
  conveniences for large ranges; direct validation accepts every legal integer,
  including interior values absent from that list. Native and public Python
  tests cover huge double-blocked combat, trample bounds and exact replay.

Both reviewers verified final HEAD 5652183 and reported both findings resolved,
with no remaining actionable findings. The rules reviewer independently passed
all 557 Rust tests and the public 40-doubling/interior-damage Python regression.
The library reviewer independently passed all 16 focused Rust tests, 65 release
Python tests and 20 native games with 3,443 public counter checks and 2,842 replay
boundaries. Earlier reviews also passed the full compiler/data suite and
catalogue consistency.

Author final validation passed 557 Rust tests (549 unit plus eight integration),
98 compiler/data tests, 65 release Python tests, catalogue consistency, rustfmt
and Clippy with warnings denied. One hundred supported Standard games completed
naturally: 56,734 decisions, 18,665 independently checked public counter values
(including unaffected objects) and 14,029 cloned/replayed boundaries. Every new
counter effect resolved, including Zimone's activated doubling. Maximum initialized
state was 18,764 bytes, excluding allocator metadata, spare capacity, external
snapshots and immutable definitions. Synthetic IR/link/high-counter fixtures are
identified as such; the 40-doubling/combat fixture uses only public legal play.

Coverage is 922/5,164 Standard identities, six regression definitions and 41 token
factories. Observation schema 35 exposes widened integer ranges and the complete
numeric damage range. Finite checked numeric capacity and recoverable exhaustion
remain tracked in issue #88; wider card semantics and RL integration remain open.
These reviews do not certify full-format conformance or establish an XMage speed
comparison.
