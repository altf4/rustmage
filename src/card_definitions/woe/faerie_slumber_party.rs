// Count the controllers of actual returns, including returned creature tokens.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ReturnBattlefieldCountOpposingControllers(&[BattlefieldFilter {
                kinds: 1,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }]),
            Instruction::CreateTokensScaled {
                token: Card(5546),
                amount: AmountValue::Multiple {
                    value: &AmountValue::MovedCount,
                    factor: 2,
                },
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
