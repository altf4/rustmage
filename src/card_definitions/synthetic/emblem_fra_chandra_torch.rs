pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    deck_limit: None,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast { relation: CastRelation::You, filter: CastFilter::Any, ordinal: None },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::Damage(5)],
    }],
    ..CardRules::EMPTY
};
