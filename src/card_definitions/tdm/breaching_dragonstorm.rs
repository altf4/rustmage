pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileUntilNonlandCastOrHand {
                maximum_mana_value: 8,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PermanentEntersYou {
                kinds: 0,
                nontoken: false,
                subtype: Some("Dragon"),
                include_source: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnSourceToHand(TargetZone::Battlefield)],
        },
    ],
    ..CardRules::EMPTY
};
