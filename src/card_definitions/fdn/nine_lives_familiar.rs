// Directly authored from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        cast_entry_counters: &[(CounterKind::Revival, 8)],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::SourceRevival),
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDeathCard(ReturnKind::Revival)],
    }],
    ..CardRules::EMPTY
};
