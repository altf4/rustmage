// Only compiled into tests. This incomplete synthetic fixture is never admitted.
#[cfg(feature = "test-support")]
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::YouManifestDread,
            targets: TargetPlan::NONE,
            program: &[],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfTurnedFaceUp,
            targets: TargetPlan::NONE,
            program: &[],
        },
        TriggeredAbility {
            condition: TriggerCondition::PermanentYouControlTurnedFaceUp,
            targets: TargetPlan::NONE,
            program: &[],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfExiledWhileCrafting,
            targets: TargetPlan::NONE,
            program: &[],
        },
    ],
    extras: &CardExtras {
        borrows_exiled_activations: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
#[cfg(not(feature = "test-support"))]
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    ..CardRules::EMPTY
};
