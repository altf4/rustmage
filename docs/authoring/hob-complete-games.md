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

Prototype run 6, with only the full-pool gate temporarily disabled, passed the
corrected witness check: 24 natural terminals, 14,839 fully replayed decisions,
717 sampled owned branches, seven rejected partial-attack attempts, all five
spell colors and 92 printed landfall packets. The guarded run terminated with
exit 0; receipt `<cache>/rustmage-build-reports/943460d630c745fda939efe834eded27.json`.
The full-pool gate was restored after termination. This remains provisional until
rerun on the final 193-card head with the gate enabled and independent review.

Final local gated verification at `18be28ac` and release `91f55d59` checked all
193 pool members, then completed 24 legal games with 14,839 replayed decisions,
717 sampled owned branches, seven rejected partial declarations, all five colors
and the corrected witness counts. Two independent nonauthor reviewers audited
the results. The 25.48-second verification duration includes full replay and
owned checks and is not a simulation benchmark.

The separately guarded optimized benchmark on `91f55d59` repeated those 24
cases ten times: 240 games and 148,390 decisions in 13.30336 seconds, or 18.04
games/s and 11,154.33 decisions/s. It includes game construction, rules, legal
actions, the shared policy and observation telemetry, and excludes replay,
owned checks, model inference, tensors and network. It is a six-deck corpus, not
a claim about all possible HOB games or XMage parity. Both independent reviewers
audited its log, arithmetic and guard receipt. Reports remain under
`<cache>/rustmage-authoring/hob-complete-set/final-benchmark/`.

The final published PR head still requires two independent nonauthor reviews.
