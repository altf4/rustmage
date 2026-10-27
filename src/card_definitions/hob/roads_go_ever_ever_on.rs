// Complete Oracle text; pinned XMage inventory records absence of a card class.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Search(SearchSpec {
                    filter: SearchFilter::AllOf(&[
                        SearchFilter::BasicLand,
                        SearchFilter::Subtype("Plains"),
                    ]),
                    maximum: 2,
                    destination: SearchDestination::LinkedExile,
                    tapped: false,
                    reveal: false,
                }),
                Instruction::GainLife(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnOneLinkedExileToHand],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnOneLinkedExileToHand],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ScheduleAttackTriggerThisTurn { ability: 4 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 2,
                    exclude: 0,
                    subtype: Some("Plains"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                power: 1,
                toughness: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
