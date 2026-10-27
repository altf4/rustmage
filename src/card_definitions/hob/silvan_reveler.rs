pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::DiscardThenReturnLand { tapped: true },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::InZone {
                zone: TargetZone::Graveyard,
                event: &TriggerCondition::LandEntersYou,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 1, 0, 0, 1, 0],
                        generic: 1,
                    },
                    skip: 1,
                },
                Instruction::ReturnSourceToHand(TargetZone::Graveyard),
            ],
        },
    ],
    ..CardRules::EMPTY
};
