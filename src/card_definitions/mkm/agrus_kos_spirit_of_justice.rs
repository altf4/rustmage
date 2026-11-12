pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::DOUBLE_STRIKE.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::IfSubjectSuspected {
                subject: Subject::Targets,
                skip: 2,
            },
            Instruction::Exile,
            Instruction::Skip(1),
            Instruction::Suspect {
                subject: Subject::Targets,
            },
        ],
    }],
    ..CardRules::EMPTY
};
