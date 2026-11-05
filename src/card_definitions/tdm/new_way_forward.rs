// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PreventNextChosenSourceDamage { ability: 0 }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DamagePlayersScaled(
                PlayerSet::EventObjectController,
                AmountValue::DamageEventAmount,
            ),
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::DamageEventAmount),
        ],
    }],
    ..CardRules::EMPTY
};
