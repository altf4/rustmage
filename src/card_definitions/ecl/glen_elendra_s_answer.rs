// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CounterOpponentsCounted,
            Instruction::CreateTokensScaled {
                token: Card(5291),
                amount: AmountValue::MovedCount,
                tapped: false,
            },
        ],
    ),
    extras: &CardExtras {
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
