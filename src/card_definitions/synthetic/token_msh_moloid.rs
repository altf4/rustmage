// Named green Minion recipe; optional exact mill uses the existing CR701.17b guard.
pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayMill(1),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(1)),
        ],
    }],
    ..CardRules::EMPTY
};
