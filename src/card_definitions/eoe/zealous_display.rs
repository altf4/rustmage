const CONTROLLED: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrowCreatures {
                filter: CONTROLLED,
                power: 2,
                toughness: 0,
            },
            Instruction::If {
                guard: StepGuard::YourTurn,
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::UntapCreatures(CONTROLLED),
        ],
    ),
    ..CardRules::EMPTY
};
