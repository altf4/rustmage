// The granted trigger belongs to the recipient and survives Sygg's departure.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::CANT_BE_BLOCKED,
    extras: &CardExtras {
        transform: Some(TransformSpec {
            front: Card(4299),
            back: Card(5351),
            front_name: "Sygg, Wanderwine Wisdom",
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrTransforms,
            targets: one(TargetRule::Creature),
            program: &[Instruction::GrantUntilEndTurn { ability: 1 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(
                &TriggerCondition::SourceCombatDamagePlayerOrPlaneswalker,
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
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
                        colored: [1, 0, 0, 0, 0, 0],
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
