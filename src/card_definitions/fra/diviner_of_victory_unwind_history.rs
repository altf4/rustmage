// Complete pinned front-face rules; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ScryOrSurveil,
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow { subject: Subject::Source, power: 1, toughness: 1 }],
    }],
    extras: &CardExtras { enters_prepared: true, prepared_spell: Some(Card(5458)), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
