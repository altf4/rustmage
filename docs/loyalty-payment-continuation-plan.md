# Activation cost replacement continuation

Issue #93 requires an activation's costs to finish before the activation is
committed. The previous payment routine could continue past a counter-placement
replacement choice, including for positive loyalty and other source-counter
costs. The owned continuation now resumes unpaid costs and completion only
after that event drains. Real Liliana and Jessica Jones scenarios exercise the
ordering; synthetic fixtures cover additional rollback and stack interactions.
This is one dependency of issue #93, not a claim that its remaining loyalty-cost
modifiers or extra-activation rules are complete.

The continuation owns the activation packet, effective ability definition,
remaining cost phase, and its completion route. When a counter event suspends,
the continuation remains owned until its replacement batch drains. It resumes the
remaining costs, source last-known information, activation observers, and the
appropriate stack or mana completion. Distinct counter costs remain ordered
events, and source departure or a new incarnation cannot redirect an old event.

Both staged and direct paths retain the continuation in owned game state,
including snapshots, restore, transactional failures, viewer-specific pending
targets and logical state accounting. Retained mana receipts preserve actual
counter-cost outcomes and trigger uses across selective rollback. No priority,
state-based action or trigger placement intervenes while costs are being paid.

The focused regressions cover both players, replacement order, activation and
counter observers, Ward, source-copy facts, capacity failures, selective mana
rollback, privacy and raw/indexed branch/restore. Synthetic fixtures are marked
as such. The broader issue remains open for cost modifiers, extra activations,
and incomplete printed card dependencies.
