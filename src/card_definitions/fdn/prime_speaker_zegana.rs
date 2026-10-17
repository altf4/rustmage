// Directly authored from complete Oracle and pinned XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::SourcePower,
        )],
    }],
    extras: &CardExtras {
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::GreatestOtherCreaturePower,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
