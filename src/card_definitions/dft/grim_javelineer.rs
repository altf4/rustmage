// Complete Oracle/XMage: one declaration trigger and a source-independent exact-target death clause.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Attacking,
            })),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 1,
                    toughness: 0,
                },
                Instruction::DelayTargetEvent {
                    event: DelayedTargetEvent::DiesOnce,
                    ability: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(1)],
        },
    ],
    ..CardRules::EMPTY
};
