const BLINK: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Creature,
    predicate: &TargetPredicate::Nontoken,
};
const RETURN_AT_END: Instruction = Instruction::BlinkExiledTargetsUntilEndStep;
const DESTROY: Instruction = Instruction::DestroyTargetsUnlessGreaterCreaturePower;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Blink nontoken creature",
            targets: one(BLINK),
            program: &[RETURN_AT_END],
        },
        SpellMode {
            text: "Destroy creature if no other has greater power",
            targets: one(TargetRule::Creature),
            program: &[DESTROY],
        },
        SpellMode {
            text: "Blink nontoken creature + Destroy creature if no other has greater power",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(BLINK),
                TargetSpec::one(TargetRule::Creature),
            ]),
            program: &[RETURN_AT_END, Instruction::SelectGroup(1), DESTROY],
        },
    ]),
    mode_costs: Some(&[generic(1), generic(2), generic(3)]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
