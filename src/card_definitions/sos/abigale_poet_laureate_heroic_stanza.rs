// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2,3), keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility { condition: TriggerCondition::SpellCast { relation: CastRelation::You, filter: CastFilter::Creature, ordinal: None }, targets: TargetPlan::NONE, program: &[Instruction::PrepareSource] }],
    extras: &CardExtras { prepared_spell: Some(Card(5351)), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
