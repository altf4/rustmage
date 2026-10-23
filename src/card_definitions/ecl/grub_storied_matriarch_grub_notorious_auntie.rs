// Direct typed behavior; complete Oracle and XMage sources read.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::MENACE,
    extras: &CardExtras {
        transform: Some(TransformSpec {
            front: Card(1893),
            back: Card(5352),
            front_name: "Grub, Storied Matriarch",
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrTransforms,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Subtype("Goblin"),
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::ReturnToHand],
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
                        colored: [0, 0, 0, 1, 0, 0],
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
