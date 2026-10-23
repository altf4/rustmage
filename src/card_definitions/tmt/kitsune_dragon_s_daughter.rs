// Complete typed behavior from full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::VIGILANCE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Filtered {
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
                    constraint: TargetGroupConstraint::DifferentControllers,
                },
                min: 2,
                max: 2,
            }),
            program: &[
                Instruction::May { skip: 1 },
                Instruction::ExchangeCreatureControl,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Filtered {
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
                    constraint: TargetGroupConstraint::DifferentControllers,
                },
                min: 2,
                max: 2,
            }),
            program: &[
                Instruction::May { skip: 1 },
                Instruction::ExchangeCreatureControl,
            ],
        },
    ],
    ..CardRules::EMPTY
};
