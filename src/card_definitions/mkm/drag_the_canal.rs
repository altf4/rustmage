pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card(5329),
                amount: 1,
                tapped: false,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::CreaturesDiedThisTurn),
                minimum: 1,
                skip: 3,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
            Instruction::Surveil(2),
            Instruction::CreateTokens {
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
