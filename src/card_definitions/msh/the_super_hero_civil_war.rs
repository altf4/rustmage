// Exact-source Saga control followed by frozen EOT boosts and an optional fight.
const OWN: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Creature,
                    constraint: TargetGroupConstraint::TotalManaValueAtMost(6),
                },
                min: 0,
                max: 2,
            }),
            program: &[Instruction::GainControlTargetsUntilSourceLeaves],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GrowCreatures {
                    filter: OWN,
                    power: 1,
                    toughness: 1,
                },
                Instruction::GrantCreatures {
                    filter: OWN,
                    keywords: Keywords::VIGILANCE,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec {
                    rule: TargetRule::Grouped {
                        base: &TargetRule::Creature,
                        constraint: TargetGroupConstraint::OtherThanGroup(0),
                    },
                    min: 0,
                    max: 1,
                },
            ]),
            program: &[Instruction::Fight {
                source: 0,
                recipient: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
