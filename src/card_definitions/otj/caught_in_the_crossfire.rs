const OUTLAWS: Instruction = Instruction::DamageCreaturesByOutlaw {
    outlaw: true,
    amount: 2,
};
const OTHERS: Instruction = Instruction::DamageCreaturesByOutlaw {
    outlaw: false,
    amount: 2,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Deal 2 damage to each outlaw creature",
            targets: TargetPlan::NONE,
            program: &[OUTLAWS],
        },
        SpellMode {
            text: "Deal 2 damage to each non-outlaw creature",
            targets: TargetPlan::NONE,
            program: &[OTHERS],
        },
        SpellMode {
            text: "Deal 2 damage to each outlaw, then each non-outlaw creature",
            targets: TargetPlan::NONE,
            program: &[OUTLAWS, OTHERS],
        },
    ]),
    mode_costs: Some(&[generic(1), generic(1), generic(2)]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
