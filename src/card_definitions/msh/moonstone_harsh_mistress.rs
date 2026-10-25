// Exact per-discard identity; the independent permission survives source departure.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Discarded {
            relation: CastRelation::You,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::MayExileDiscardedPlay { next_turn: true }],
    }],
    ..CardRules::EMPTY
};
