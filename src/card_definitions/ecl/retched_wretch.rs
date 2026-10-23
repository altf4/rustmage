// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::SourceCounter {
            kind: CounterKind::MinusOne,
            minimum: 1,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDeathCard(ReturnKind::WithoutAbilities)],
    }],
    ..CardRules::EMPTY
};
