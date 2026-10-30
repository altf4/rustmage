# Loyalty-cost counter placement gap

Inspired Tethermage must trigger when its controller puts one or more loyalty
counters on any planeswalker, including counters placed to pay a positive
loyalty cost. The current `pay_activation_definition` uses `add_counter`
directly for positive loyalty costs, bypassing the counter-placement event and
its replacement effects. The Inspired Tethermage scenario for a positive
loyalty cost therefore describes a known failing obligation; the card must
remain gated.

Simply routing this cost through `activation_cost_counters` is not sufficient.
`stage_counter_placements_inner` can suspend for replacement-order choices,
while `finish_announcement` takes the announcement out of state, calls the
payment routine and can continue inserting the ability. An owned activation
payment continuation must retain the announcement, complete the placement and
any replacement choices, and resume the remaining costs exactly once before
the activation is committed. It must also preserve rollback and avoid exposing
priority during payment. Both the announcement and legacy action paths must
share this ordering. A prohibited or altered placement must be evaluated using
the rules for paying that specific cost, rather than assuming success.

Required tests include an unmodified positive loyalty cost, counter-placement
observers, one and multiple replacement effects with order choices, non-effect
versus effect-only multipliers, failed payment, state-based actions and source
departure, copied/granted loyalty abilities, and raw/index branch/restore at
each suspended choice. The original guarded build is still in flight; neither
the scenario nor a continuation design has been compiled or tested here.

The same payment routine already calls `activation_cost_counters` for typed
`activation_put_counters` costs, including Mazemind Tome and Jessica Jones,
before it performs later source costs or returns to `finish_announcement`.
That existing path also needs the owned continuation when replacements defer;
the fix must not be limited to positive loyalty costs. The new
`loyalty_cost_uses_non_effect_counter_replacements` scenario distinguishes
Doc Samson's cost-inclusive replacement from Doubling Season's effect-only
replacement, and is expected to fail until the gap is closed.
