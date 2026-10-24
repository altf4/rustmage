// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2,3),
    triggers: &[TriggeredAbility { condition: TriggerCondition::CardsLeaveGraveyard, targets: TargetPlan::NONE, program: &[Instruction::PrepareSource] }],
    extras: &CardExtras { prepared_spell: Some(Card(5370)), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
