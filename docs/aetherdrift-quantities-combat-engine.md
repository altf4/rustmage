# Aetherdrift quantity, attack-group, and combat-schedule engine checkpoint

This records shared engine work before ordinary Aetherdrift card authorship.
The fixture scenarios retain existing physical metadata and admit no DFT cards.
Full source pins and complete Oracle bodies are in
[`aetherdrift-quantities-combat-sources.json`](aetherdrift-quantities-combat-sources.json).
XMage revision is `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.
The pinned membership/Oracle snapshot is the repository's 2026-10-15 pool.

## Source interpretation and typed APIs

| Consumer | Audited obligations and reusable evidence | Shared representation |
| --- | --- | --- |
| Sab-Sunen, Luxa Embodied | Counter parity includes every kind, including zero. `CountersSourceCount.ANY` reads current exact permanent or departure LKI. Place the +1/+1 counter first, including replacements, then test odd parity for the draw. | `AmountValue::SourceTotalCounters` opts into existing sparse `source_counter_lki` for all `CounterKind::ALL`; `StatValue::SourceTotalCounters` and `Parity` drive an ordinary `CombatRestriction`; `IfAmountParity` gates later instructions. |
| Sita Varma, Masked Racer | Exhaust's X is a real activation cost. After its counter event/replacements, the controller may set each other controlled creature's base P/T to the source's signed power. `SourcePermanentPowerValue.ALLOW_NEGATIVE` and `SetBasePowerToughnessAllEffect.init` freeze source value and exact recipients. | Existing `ActivationX`, `ChosenX`, exhaust, and optional-choice machinery; `ResolvedStatValue::SourcePower`; `SetPermanentGroupBaseStats` with explicit `exclude_source`. Equal P/T expressions are sampled once before any base writes and share one existing layer-7b timestamp. |
| Sabotage Strategist | One event for a completed declaration with one or more creatures attacking the observer's controller as a player. `DefenderAttackedEvent`/`FixedTargets` retain exact attackers. Neither planeswalker/Battle attacks nor later creatures put onto the battlefield attacking join that group. | `TriggerCondition::CreaturesAttackYou`, existing `reflexive_subject` exact group, and `GrowDeclaredAttackers`. No targeting or new per-game group cache. |
| Oviya, Automech Artisan | Every creature attacking an opponent **player** of the provider's current controller has trample. `GainAbilityAllEffect` is live and removal-aware; the card predicate compares the actual defender player, not a planeswalker's controller or a Battle protector. | `OpponentPlayerAttackerKeywords`, shared layer-six evaluation, and `CharacteristicWorld::attacking_defender` in live and speculative payment worlds. Sparse funding falls back to unchanged actual attacker facts for objects outside its mutable resource set. |
| Full Throttle | `AdditionalCombatPhaseEffect(2)` inserts after the current main. The repeated delayed ability lasts through this turn and is independent of the spell's later zone. `AttackedThisTurnWatcher` uses exact objects that declared an attack; `UntapAllEffect` selects current creatures at resolution without a controller restriction. | `AdditionalCombatsAfterCurrentMain`, cold `MainPhaseCombatView` with frozen resume step, `ScheduleBeginCombatThisTurn`, and `UntapCreaturesThatAttackedThisTurn`. Existing untap, stun and prohibition rules perform the actual event. |

The Sab-Sunen fixture must supply both its immutable static group and the existing
`CardExtras::combat_restriction` lookup. A static group alone does not populate
that separate typed accessor. Source departure freezes the latest counters, not
the trigger-time count, and a later incarnation cannot receive the old source's
counter placement or replace its quantity LKI.

For Sita-style effects, a new incarnation still controlled by the ability's
controller is an **other** creature and can be in the affected set. The original
exact source is excluded. The source departure test therefore gives the returned
incarnation to the opponent to isolate LKI sampling; a separate test proves that
a controlled returned incarnation receives the old source's sampled signed base.
A current exact noncreature has no power and supplies zero. Departed creatures
supply their actual signed departure power.

For Battle fixtures, a non-Siege Battle is protected by its controller. The
attack-group scenarios construct that rules-consistent defender; an opponent
protector imposed on a non-Siege Battle is correctly replaced by existing SBAs.

## Independent outcome contracts

| Target | Distinguishing assertions |
| --- | --- |
| `dft_quantities_engine` (10 scenarios) | Zero/hot/named parity controls attacking and blocking; losing the restriction restores permission. Doubled placement produces even counters and no draw. Latest exact departure parity survives blink and owned raw/index/restored resolution. Nested modal conditional headers opt into all-kind capture. Actual exhaust-X payment with X=2 is replaced before signed group sampling. Optional decline retains paid counters. Later entrants and new recipient incarnations do not inherit the fixed base effect. Negative power, a current noncreature, and a new source incarnation have separate proofs. |
| `dft_attack_groups` (8 scenarios) | One non-targeted group for two player attackers; walker/Battle-only declarations produce none. Hexproof is irrelevant, source departure preserves the packet, a blinked attacker and later entrant are excluded, countering prevents the bonus, and cleanup removes it. Owned branches resolve the same exact group. Live/funding keyword queries distinguish player/walker/Battle/missing defenders, provider control and suppression, provider sacrifice, and control returning after a control Aura is sacrificed during funding. Recipient membership is deliberately not restricted to the provider's controller. |
| `dft_combat_schedule` (7 scenarios) | Main1 inserts two combats before its normal combat, then Main2; Main2 inserts two and resumes at End. No main phase appears between inserted combats. Multiple effects and legacy queued `AdditionalCombat` retain the frozen successor. A non-main resolution creates the delay but no phases. Repeated packets keep the original resolving controller. Exact current attacked creatures untap across controller changes; blinked/new/noncreature objects do not. Stun and true untap prohibition differ. Countering one trigger leaves later triggers active. Owned scheduling and cleanup expiration are explicit. |

The latest coordinated integrated run reported all 10 quantity, all 8 attack-group
and all 7 combat-schedule scenarios passing. Validation is owned by the root
integrator; this worker did not run Cargo. These shared proofs do not establish
complete card wiring or complete-set acceptance.

## Deferred mana constructor and continuation contracts

The max-speed nested proof exposed a separate action-generation gap: cost windows
could execute currently payable indexed mana actions but could not announce one
whose own mana component needed another mana ability. Performing that other mana
ability first can change cost/production sampling; the test keeps the deferred
ordering as a meaningful contract.

`activation_packet` is an immutable source/definition/index/grant/borrowed-instance
constructor. Its preview ID is the next unused ID without mutation and exists
only in an uncommitted funding header. `activation_item` allocates the real ID.
`payment_announcement` is shared by actual and preview paths, retaining costs,
exhaust starting authorization and counters. `announced_mana_quality` is shared
so the original source definition's tagged production is frozen before departure.

The cost-window preflight clones only cold enclosing announcement frames once,
including selected exact groups, completion flags and borrowed slots. It creates
no Game clone. It replaces the inherited parent's cost context with the proposed
child context and installs the existing exact `PaymentContinuation` through every
enclosing frame. Successful payment of the child alone is insufficient: all
parents must still be completable after the child's actual production and costs.
The existing funding reachability search is reused; no contributor powerset is
introduced.

`ManaActivationKey` compares the exact source incarnation, immutable definition
and index, borrowed acquisition slot, self-grant timestamp, source copy origin
and grantor/definition/origin. New candidate announcements and speculative mana
moves both reject an instance already active in any mana ancestor. Completing an
already-announced instance uses the existing frozen continuation path instead.

`dft_deferred_mana` adds three scenarios: a real depth-two nested chain in both
seats with all-ancestor recursion rejection and read-only action generation; a
fundable child that would strand its parent; and an unpayable child despite a
payable parent. Rejected actions retain the exact snapshot/revision. The retained
`dft_speed_grants` nested scenario adds source-grant provenance and owned
raw/index/restore after disposing of the parent Game.

The initial deferred action adapter handles indexed pure mana with a mana
component and no unselected external cost. Existing external-cost choices retain
their existing route. Additional adapters, named activation taxes and taxed
Crew/Saddle costs remain engine blockers; they are not claimed implemented by
this checkpoint. The deferred commit had not yet consumed coordinated validation
when this document was authored.

## Grant provenance is not an extra tap cost

The first combined run also exposed an old conflation:
`announcement_reserve` treated every activated grant context as an attachment
that had to tap, requiring the source to be its own distinct additional cost.
`attached_activation_tap_cost` now recognizes only the exact immutable
`AttachmentActivation` template with a distinct grantor. Both speculative reserve
selection and actual tap payment use that predicate. Self/max-speed and external
planeswalker grants retain their provenance without acquiring invented costs.
Contrasting scenarios prove that genuine attachment templates still reserve and
pay both distinct taps, including rejection if the grantor becomes tapped before
commitment.
