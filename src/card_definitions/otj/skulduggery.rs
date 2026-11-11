const OPPOSING_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(OPPOSING_CREATURE),
        ]),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
            Instruction::SelectGroup(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: -1,
                toughness: -1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
