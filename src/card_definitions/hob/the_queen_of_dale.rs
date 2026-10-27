// Full Oracle and XMage: first actual noncreature cast by each opponent per turn.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::NonCreatureOrdinal(1),
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Recruit { token: Card(5241) }],
    }],
    ..CardRules::EMPTY
};
