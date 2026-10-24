// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[Instruction::GainLifeScaled(
            PlayerSet::Targeted,
            AmountValue::ChosenX,
        )],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/infirmary_healer_stream_of_life"),
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
