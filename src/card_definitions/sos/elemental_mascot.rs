// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1,4), keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility { condition: TriggerCondition::ManaSpent { minimum: 0, increment: false, event: &TriggerCondition::SpellCast { relation: CastRelation::You, filter: CastFilter::InstantSorcery, ordinal: None } }, targets: TargetPlan::NONE, program: &[
        Instruction::Grow { subject: Subject::Source, power: 1, toughness: 0 },
        Instruction::IfManaSpent { minimum: Some(5), maximum: None, skip: 1 },
        Instruction::ExileTopPlay { amount: 1, next_turn: true },
    ] }],
    ..CardRules::EMPTY
};
