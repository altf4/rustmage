// Both immutable faces use one physical Oracle identity and retain their object when transformed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    extras: &CardExtras {
        transform: Some(TransformSpec {
            front: Card(552),
            back: Card(5347),
            front_name: "Brigid, Clachan's Heart",
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrTransforms,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5306),
                amount: 1,
                tapped: false,
            }],
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
                        colored: [0, 0, 0, 0, 1, 0],
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
