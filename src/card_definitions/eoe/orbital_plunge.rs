// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::DamageForExcess(AmountValue::Fixed(6)),
            Instruction::IfNoExcessDamage { skip: 1 },
            Instruction::CreateTokens {
                token: Card(5238),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
