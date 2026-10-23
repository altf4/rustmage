// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::DEATHTOUCH,
    extras: &CardExtras {
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::PermanentLeftBattlefieldThisTurn,
            amount: EntryCounterAmount::Fixed(2),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
