// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::ChosenX),
            Instruction::PutHandPermanent {
                maximum: AmountValue::ChosenX,
                tapped: true,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
