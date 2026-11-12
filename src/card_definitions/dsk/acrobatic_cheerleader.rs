pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerIncarnation(&TriggerCondition::SURVIVAL),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Flying,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
