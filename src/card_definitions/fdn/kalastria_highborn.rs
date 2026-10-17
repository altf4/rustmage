// Directly authored from complete pinned Oracle and XMage executable behavior.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 0,
                    },
                    skip: 2,
                },
                Instruction::LoseLife(PlayerSet::Targeted, 2),
                Instruction::GainLife(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::You,
                other: true,
                nontoken: false,
                subtype: Some("Vampire"),
            },
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 0,
                    },
                    skip: 2,
                },
                Instruction::LoseLife(PlayerSet::Targeted, 2),
                Instruction::GainLife(PlayerSet::Controller, 2),
            ],
        },
    ],
    ..CardRules::EMPTY
};
