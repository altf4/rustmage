// Whole Oracle: actual optional payment precedes independent targeted reanimation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 3,
                    },
                    skip: 1,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::ReturnTargetAttackingWithCounter(
                CounterKind::Finality,
            )],
        },
    ],
    ..CardRules::EMPTY
};
