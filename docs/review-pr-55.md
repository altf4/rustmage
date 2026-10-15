# Independent review: creature Auras and transactional entry

PR: https://github.com/altf4/rustmage/pull/55
Implementation reviewed: a55b098 against main.

Two independent read-only subagents reviewed this milestone:

- Rules: Aura casting and revalidation, non-targeted entry transactions and APNAP,
  impossible and simultaneous entries, attached entry effects, linked-return and
  payment continuations, state-based actions, attachment LKI, all 14 admissions
  and complete-text compiler gating.
- Library: entry transaction ownership and resumption, snapshots and initialized
  memory accounting, schema 23 private/public projections, attachment LKI and
  funding, and all 14 admissions.

Both reported no actionable findings. Both independently passed 308 Rust tests
(300 unit and eight integration), 56 compiler tests and catalogue consistency.
The library reviewer also passed 34 release Python tests and 20 native games
with 137 Aura attachments, five noncasting choices and 845 replayed boundaries.
Neither reviewer edited files, mutated Git or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and the complete Rust/compiler/release Python suites. One hundred
supported Standard games completed naturally: 57,512 decisions, 729 Aura
attachments, 40 noncasting choices, 135 Aura battlefield-to-graveyard changes
and 4,711 cloned/replayed boundaries. Maximum initialized state was 11,768 bytes
for that workload, excluding allocator metadata, spare capacity, snapshots and
static definitions. Fixtures distinguish synthetic IR/link combinations from
actual complete-card admissions; see auras.md for their scope.

Coverage is 670/5,164 Standard identities, six regression cards and 36 token
factories. Broader enchant clauses, layers, control changes, granted and
attached-creature triggers, residual card semantics and full native RL/search
integration remain outstanding. These reviews do not certify full Standard
conformance or establish an XMage performance comparison.
