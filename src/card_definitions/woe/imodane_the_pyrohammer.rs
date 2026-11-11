// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSpellDamagesSoleCreatureTarget,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayersScaled(
            PlayerSet::Opponent,
            AmountValue::DamageEventAmount,
        )],
    }],
    ..CardRules::EMPTY
};
