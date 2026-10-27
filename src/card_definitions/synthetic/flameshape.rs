pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileTopFaceDownPlay {
            amount: 2,
            subtype: "Wizard",
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1676)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
