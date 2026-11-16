// Pinned Oracle and XMage: haste Klingon Warrior and graveyard Beam Me Up route.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(6010),
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        beam_me_up_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
