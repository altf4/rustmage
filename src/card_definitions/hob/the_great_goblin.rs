// Complete printed obligations from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CounterPlacedByYouOnControlledSubtypes {
                subtypes: &["Goblin", "Orc", "Army"],
            },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::Damage(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::OtherControlledPermanentWithSubtypeDies {
                subtypes: &["Goblin", "Orc", "Army"],
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlay {
                amount: 1,
                next_turn: true,
            }],
        },
    ],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 1, 1, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 1,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
