// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(5303),
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        harmonize_cost: Some(ManaCost {
            generic: 5,
            colored: [0, 1, 0, 1, 1, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
