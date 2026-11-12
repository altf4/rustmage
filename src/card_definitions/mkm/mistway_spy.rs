// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfTurnedFaceUp,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ScheduleTurnTrigger {
                    event: TurnCallbackEvent::ControlledCreatureCombatDamagePlayer(
                        crate::engine::Player::P0,
                    ),
                    ability: 1,
                    once: false,
                },
                Instruction::ScheduleTurnTrigger {
                    event: TurnCallbackEvent::ControlledCreatureCombatDamagePlayer(
                        crate::engine::Player::P1,
                    ),
                    ability: 1,
                    once: false,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5296),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
