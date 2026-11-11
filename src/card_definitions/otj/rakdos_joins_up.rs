// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
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
            program: &[Instruction::ReturnToBattlefieldWithCounters(&[(
                CounterKind::PlusOne,
                2,
            )])],
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledLegendaryCreatureDies,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::DamageScaled(AmountValue::DeathEventPower)],
        },
    ],
    ..CardRules::EMPTY
};
