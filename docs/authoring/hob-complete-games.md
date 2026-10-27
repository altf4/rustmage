# HOB set-only game workload

The workload defines six 60-card Standard decks using physical HOB members, with
four-copy limits for nonbasic cards, both starting seats, deterministic seeds,
natural endings and all five spell colors. The common smoke policy scores legal
actions; a generic per-declaration memory removes only attackers whose most
recent attempted declaration was canceled because it had no valid completion.
The ordinary offered-action index is retained for raw/index comparisons. Verify
mode checks wrong-actor transactional failure, owned raw/index branches, restore
and complete replay. Benchmark mode omits those verification costs. Witnesses
are observations of the named events, not an exhaustive card-coverage claim.

The initial prototype at base fd1fc23e skipped the all-members-supported check
because five cards were still gated. Run 1 hit an 8192-decision limit; run 2
identified repeated Attack/CancelAttackDeclaration; run 3 recorded legal choices
showing the canceled partial declaration offered no completion. Run 4 used the
generic cancellation memory and completed all 24 games and replay checks, but
failed its hone witness requirement. Run 5 revised the fixed Dwarf deck mix and
passed 24 games, 14839 replayed decisions, 717 sampled owned branches, five spell
colors, and all five tracked mechanic categories (3 enduring-story players, 5
hone-bearing objects, 40 Goblin Army tokens, and 36 Human Soldier tokens).
The original count of 112 landfall packets was overstated: an independent review
found it included Silvan Reveler's unrelated enter trigger. The witness was
corrected to require the printed ability index and origin; its corrected count
requires a fresh run. Its verification rate is not a simulation benchmark.
Logs and resource receipts are retained in `<cache>/rustmage-authoring/hob-games/`.

This is prototype evidence only: the full-pool admission check was disabled in
those runs. It has since been restored, and a counter of rejected attack attempts
added to the report. Both changes require verification on the final full-set head,
including a separate benchmark and independent review, before set acceptance.
