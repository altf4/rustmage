// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(5)],
        },
        TriggeredAbility {
            condition: TriggerCondition::LoyaltyAbilityActivated,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GainLife(PlayerSet::Controller, 1),
                Instruction::AdditionalLandPlays(1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
