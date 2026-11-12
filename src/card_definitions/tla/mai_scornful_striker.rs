// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FIRST_STRIKE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Any,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::CastingPlayer, 2)],
    }],
    ..CardRules::EMPTY
};
