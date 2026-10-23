// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GrowCreatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: true,
                        subtype: Some("Dinosaur"),
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    power: 1,
                    toughness: 1,
                },
                Instruction::GrantCreatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: true,
                        subtype: Some("Dinosaur"),
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    keywords: Keywords::FLYING,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensScaled {
                token: Card(5360),
                amount: AmountValue::ChosenX,
                tapped: false,
            }],
        },
    ],
    extras: &CardExtras {
        x_symbols: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
