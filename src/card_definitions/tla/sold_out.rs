// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::RememberCount(AmountValue::TargetDamagedThisTurn(0)),
            Instruction::Exile,
            Instruction::IfAmountAtLeast {
                value: AmountValue::RememberedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
