// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exile: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
                Instruction::Reflexive { ability: 3 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::DelayCreatureAttacksAlone { ability: 4 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ReturnToHand],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedCreatureAttacksAlone,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Entering,
                    kind: CounterKind::PlusOne,
                    amount: 3,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Entering,
                    keywords: Keywords::TRAMPLE
                        .union(Keywords::LIFELINK)
                        .union(Keywords::INDESTRUCTIBLE),
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
