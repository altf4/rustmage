// Directly authored full frozen Oracle and pinned XMage body; shared rules are typed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(
            &TriggerCondition::ControlledAbilityTargetsPlayerOrPermanent,
        ),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
