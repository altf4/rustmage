// Protection is a removable layer-six grant lasting until the original controller's next turn.
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
    deck_limit: None,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfTransforms,
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
            program: &[Instruction::GrantProtectionUntilNextTurn],
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
                        colored: [0, 1, 0, 0, 0, 0],
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
