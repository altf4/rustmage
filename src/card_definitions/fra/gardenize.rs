// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::You,
                other: false,
                nontoken: false,
                subtype: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Charge,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddManaScaled {
                color: ChoiceColor::Green,
                amount: AmountValue::SourceCounterQuantity(CounterKind::Charge),
            }],
        },
    ],
    ..CardRules::EMPTY
};
