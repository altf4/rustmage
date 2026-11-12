pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: true,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::Suspect {
                subject: Subject::Targets,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledSuspectedCreaturesAttack,
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(1)],
        },
    ],
    ..CardRules::EMPTY
};
