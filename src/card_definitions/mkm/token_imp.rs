pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 2)],
    }],
    deck_limit: None,
    ..CardRules::EMPTY
};
