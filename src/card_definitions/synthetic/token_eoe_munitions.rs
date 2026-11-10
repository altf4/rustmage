pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: one(TargetRule::Any),
        program: &[Instruction::Damage(2)],
    }],
    ..CardRules::EMPTY
};
