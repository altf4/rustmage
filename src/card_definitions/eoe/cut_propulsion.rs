pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::IfSubject {
                subject: Subject::Targets,
                filter: CreatureFilter {
                    relation: -1,
                    other: false,
                    subtype: None,
                    keyword: Keywords::FLYING,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                skip: 2,
            },
            Instruction::ScaledPowerDamage {
                source: 0,
                recipient: 0,
                factor: 2,
            },
            Instruction::Skip(1),
            Instruction::PowerDamage {
                source: 0,
                recipient: 0,
            },
        ],
    ),
    ..CardRules::EMPTY
};
