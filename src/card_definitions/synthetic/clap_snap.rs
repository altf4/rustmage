pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::EnsureGoblinArmy, Instruction::AmassGoblins(2)],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1867)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
