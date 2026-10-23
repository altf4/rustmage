// Complete typed behavior from full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 1,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::OtherThanSource,
                },
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::CaptureDelayedReturnTarget,
                Instruction::ScheduleStep {
                    step: TriggerStep::End,
                    relation: CastRelation::Any,
                    ability: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 3,
                    },
                    skip: 1,
                },
                Instruction::Skip(1),
                Instruction::ResolveDelayedReturn,
            ],
        },
    ],
    ..CardRules::EMPTY
};
