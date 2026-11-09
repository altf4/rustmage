// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::AnyPlayerCombatDamageAtLeast(6),
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::EndCombat,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::TransformSource,
                Instruction::AttachSourceToControlledCreature,
            ],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
