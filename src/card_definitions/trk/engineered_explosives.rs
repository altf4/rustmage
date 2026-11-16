// Complete pinned Oracle and XMage: Sunburst counts colors spent, and source
// charge counters remain available as last-known information after sacrifice.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        x_symbols: 1,
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::ColorsSpentKind(CounterKind::Charge),
        }],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DestroyManaValueExactly {
            kinds: 1 | 16 | 32 | 64 | 256,
            amount: AmountValue::SourceCounterQuantity(CounterKind::Charge),
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
