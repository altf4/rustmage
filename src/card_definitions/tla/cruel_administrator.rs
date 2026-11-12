pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5706),
            amount: 1,
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Attacked,
            amount: EntryCounterAmount::Fixed(1),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
