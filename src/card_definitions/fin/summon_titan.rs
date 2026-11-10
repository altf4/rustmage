// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::REACH.union(Keywords::TRAMPLE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::Mill(
                PlayerSet::Controller,
                MillAmount::Fixed(5),
            )],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnOwnedGraveyardLandsTapped],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::creature_you_control(),
                predicate: &TargetPredicate::OtherThanSource,
            })),
            program: &[
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::TRAMPLE,
                },
                Instruction::GrowScaled {
                    subject: Subject::Targets,
                    amount: AmountValue::Stat(StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 2,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    })),
                    power: 1,
                    toughness: 1,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
