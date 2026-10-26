pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::IfCastFromGraveyard { skip: 2 },
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::Skip(1),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
