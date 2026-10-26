pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::EnsureGoblinArmy,
            Instruction::IfCastFromGraveyard { skip: 2 },
            Instruction::AmassGoblins(3),
            Instruction::Skip(1),
            Instruction::AmassGoblins(1),
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
