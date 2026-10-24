// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card::SosPestToken,
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 0,
        }],
        prepared_identity: Some("@prepare/lluwen_exchange_student_pest_friend"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
