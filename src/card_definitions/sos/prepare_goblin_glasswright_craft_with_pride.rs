// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(5302),
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/goblin_glasswright_craft_with_pride"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
