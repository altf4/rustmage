// Soul Tether prepared by Konstrari Improviser.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(5438),
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 2,
        }],
        prepared_identity: Some("@prepare/konstrari_improviser_soul_tether"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
