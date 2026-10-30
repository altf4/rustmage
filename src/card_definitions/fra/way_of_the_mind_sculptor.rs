// Complete pinned Oracle and XMage behavior; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(5)],
        },
        TriggeredAbility {
            condition: TriggerCondition::LoyaltyAbilityActivatedRemoving(2),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
