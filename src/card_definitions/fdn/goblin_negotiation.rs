// Direct typed behavior from complete pinned Oracle and XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::DamageForExcess(AmountValue::ChosenX),
            Instruction::CreateExcessTokens(Card(5324)),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
