// Chandra's command-zone emblem.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    deck_limit: None,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ArtifactEntersYou,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::Damage(3)],
    }],
    ..CardRules::EMPTY
};
