// Complete pinned XMage Saga class and Oracle chapters.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec {
                    rule: TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 1,
                        relation: 1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
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
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
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
            program: &[Instruction::ReturnToHand],
        },
    ],
    ..CardRules::EMPTY
};
