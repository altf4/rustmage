// Omit Variables prepared by Paradox Shaper.
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
        prepared_identity: Some("@prepare/paradox_shaper_omit_variables"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
