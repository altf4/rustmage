# Activation cost replacement continuation

Issue #93 requires an activation's costs to finish before the activation is
committed. Counter-placement costs can enqueue replacement work. In the current
payment routine, later costs and activation observers can run before that work
is processed. This affects loyalty-counter additions and other activation costs
that put counters on their source. A real-card regression is being checked
separately; this document is a design note, not evidence of a completed fix.

The continuation must own the activation packet, effective ability definition,
remaining cost phase, and its completion route. Record the next phase before
staging a counter event; if that event suspends, retain the continuation until
the whole replacement batch and any nested work finish. Only then resume the
remaining costs, source last-known information, activation observers, and the
appropriate stack or mana completion. Two separate counter costs must remain
separate ordered events, and source departure or a new incarnation must never
redirect an old event.

Both staged announcements and direct activation paths need the same semantics.
Nested mana announcements must preserve their enclosing announcement and
replacement context. The continuation belongs to owned game state and must
participate in snapshots, restore, transactional errors, observation privacy,
and logical state accounting. No priority, state-based action, or trigger
placement may intervene while costs are being paid. The synchronous no-choice
path should use the same cost ordering without mandatory heap allocation.

Exercise real positive-loyalty and counter-cost cards with replacement choices,
both players, effects that alter later cost legality, activation and counter
observers, source departure, nested mana, capacity failures, and raw/indexed
branch/restore. Synthetic fixtures should be explicitly identified and should
supplement, not replace, genuine supported-card interactions.
