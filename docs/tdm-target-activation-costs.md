This shared engine checkpoint covers target-subtype spell surcharges, target
color-count activation discounts, activation X-life costs, and exactly-X
activation targets. It changes no card definition or admission. Behold and
subtype-filtered free casting remain a separate subsequent checkpoint.

`CardExtras.target_cost_increase` supplies a typed target rule and one generic
increase when any chosen exact target qualifies. Pretarget completion checks
whether a qualifying target can be avoided. The shared spell total applies the
increase before reductions; Harmonize proposals include both the target
qualification and the chosen creature's discount.

`activation_target_color_discounts` identifies one mandatory permanent-target
activation and its generic reduction per distinct effective WUBRG color. Raw
activation alternatives are retained using a five-color optimistic bound before
targeting. Every accepted target and the final cost lock use the exact raw cost,
taxes and reductions through the same wide arithmetic. Actual payment never
uses that optimistic bound. A mana ability removing the tax provider after the
cost is locked leaves the locked cost intact.

`ActivationLifeX` adds the chosen X to the existing ordered Life cost clauses.
An ordinary X mana component is still expressed by the leading `ActivationX`
markers. Funding includes the activation source's costs and the life spent by
mana children in one borrowed resource world. Actual life payment is committed
once through the existing cost transaction.

`TargetsExactlyX` uses one homogeneous target group with static minimum zero.
The announced cardinality stays in u32/usize instead of narrowing into the u16
minimum field. The declaration domain requires enough distinct legal targets;
completion requires exactly X. X0 has no targets and can resolve normally.

The paired evidence retains the original68 positive cases, an independently
failing MAX-tax/five-color witness, and70 matching-input final cases. The final
14 focused groups include real payment, target incarnation/control, locked cost,
costed mana children, wrong-actor rejection, transactional failure and owned
raw/index/restore behavior. The56 inherited cases cover casting/Harmonize,
variable costs, acquired activations, target reductions, taxes and equipment.
Global trace/schema acceptance, whole-card/set validation and simulation
throughput remain integration work.
