// Omit Variables prepared by Void Extrapolator.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Mill(
            PlayerSet::Controller,
            MillAmount::Fixed(3),
        )],
    ),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 0,
        }],
        prepared_identity: Some("@prepare/void_extrapolator_omit_variables"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
