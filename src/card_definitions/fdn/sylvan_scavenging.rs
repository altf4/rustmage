// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Put a +1/+1 counter on target creature you control.",
                targets: one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                program: &[Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Create a 3/3 green Raccoon creature token if you control a creature with power 4 or greater.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[
                    Instruction::If {
                        guard: StepGuard::Threshold {
                            value: StepValue::GreatestCreaturePower,
                            minimum: Some(4),
                            maximum: None,
                        },
                        skip: 1,
                    },
                    Instruction::CreateTokens {
                        token: Card(5336),
                        amount: 1,
                        tapped: false,
                    },
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
