// Both faces authored from the pinned Oracle and complete XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::ControlledCreatureDeaths,
        }],
        adventure_identity: Some("Callous Sell-Sword // Burn Together"),
        adventure_spell: Some(Card(5430)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
