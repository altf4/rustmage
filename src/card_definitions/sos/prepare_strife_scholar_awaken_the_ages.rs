// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(5350),
            amount: 2,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/strife_scholar_awaken_the_ages"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
