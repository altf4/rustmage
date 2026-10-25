// Legendary Zabu recipe from Ka-Zar's complete printed token instruction.
pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
