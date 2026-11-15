// Pinned Oracle and XMage: a colorless Illusion artifact creature, with Beam Me Up.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(6011),
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        beam_me_up_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
