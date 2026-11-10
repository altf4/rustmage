pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: -1,
                    other: true,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::PutCountersScaled {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::SourcePower,
                },
                Instruction::AddCreatureTypes {
                    subject: Subject::Targets,
                    types: &["Mutant"],
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::YourTurn,
                event: &TriggerCondition::ControlledPermanentWithSubtypeDies("Mutant"),
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::DeathEventPower,
            )],
        },
    ],
    ..CardRules::EMPTY
};
