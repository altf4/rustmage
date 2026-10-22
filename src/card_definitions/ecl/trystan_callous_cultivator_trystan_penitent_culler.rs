// Full front-face mill and Elf-card condition, evaluated after the mill completes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::DEATHTOUCH,
    extras: &CardExtras {
        transform: Some(TransformSpec {
            front: Card(4697),
            back: Card(5349),
            front_name: "Trystan, Callous Cultivator",
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrTransforms,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
                Instruction::If {
                    guard: StepGuard::Expression {
                        value: StatValue::Count(CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Elf"),
                            basic: false,
                            other: false,
                            cards_only: true,
                        }),
                        minimum: Some(1),
                        maximum: None,
                    },
                    skip: 1,
                },
                Instruction::GainLife(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 0,
                    },
                    skip: 1,
                },
                Instruction::TransformSource,
            ],
        },
    ],
    ..CardRules::EMPTY
};
