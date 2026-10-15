# First training target: current Standard

The user selected current Standard on October 15, 2026. Prioritize the rules and
cards required by that pool before expanding to older formats. The library does
**not** yet implement the full format. `Standard` enforces the frozen legal pool
but rejects unimplemented cards. Generic `Constructed` does not enforce Standard
set legality or bans.

The initial target snapshot is
[standard-2026-10-15.json](../data/targets/standard-2026-10-15.json), researched
against Wizards' [Standard page](https://magic.wizards.com/en/formats/standard),
[banned list](https://magic.wizards.com/en/banned-restricted-list), and
[September 25 rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Use tabletop/Magic Online Standard as the initial variant, consistent with the
existing MTGO pipeline; Arena-specific digital substitutions and bans need their
own explicitly selected variant if requested later.

Next acceptance gates:

1. Build a pinned inventory of unique legal card identities/printings for the
   listed sets and legal reprints; reconcile bans and incomplete XMage entries.
   The target JSON is planning data. The initial inventory and reconciliation are
   now in [standard-inventory.md](standard-inventory.md), with 26 source gaps
   explicitly recorded; executable rules coverage remains incomplete.
2. Group Standard cards by required mechanics, port reusable rule primitives,
   and establish a machine-readable implemented/unsupported report. Missing
   dependencies must block deck admission.
3. Run complete legal Standard decks through the Rust and Python library APIs,
   using independent rules fixtures and selected XMage differential scenarios.
4. Implement Standard deck and sideboard validation separately from engine card
   support, then single-game self-play and later best-of-three sideboarding.
5. Add the policy encoder, batched inference/worker integration, and fair
   hidden-information search. Re-run performance measurements on these decks.

The local XMage Standard class computes sets using the host calendar and permits
future sets for beta testing. Copying that behavior would make training legality
change with wall-clock time. Use a versioned explicit manifest instead, and
record its identity in trajectories/checkpoints. Keep future set or ban changes
as reviewed updates rather than silently changing old experiments.
