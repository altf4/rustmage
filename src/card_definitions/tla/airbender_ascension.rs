// Direct typed behavior from Oracle and pinned XMage sources; admission awaits tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::AirbendTargets],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Quest,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::Expression {
                    value: StatValue::SourceCounters(CounterKind::Quest),
                    minimum: Some(4),
                    maximum: None,
                },
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: 1,
            }),
            program: &[Instruction::BlinkImmediately {
                counter: None,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
