// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(5349),
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/campus_composer_aqueous_aria"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
