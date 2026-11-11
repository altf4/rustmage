pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::HASTE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: generic(2),
                    skip: 1,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Subtype("Dinosaur"),
                }),
                TargetSpec::one(TargetRule::Grouped {
                    base: &TargetRule::Creature,
                    constraint: TargetGroupConstraint::OtherThanGroup(0),
                }),
            ]),
            program: &[Instruction::PowerDamage {
                source: 0,
                recipient: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
