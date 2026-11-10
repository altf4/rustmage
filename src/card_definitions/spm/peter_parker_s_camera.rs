pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            remove: Some((Some(CounterKind::Film), 1)),
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::YourAbility),
        program: &[Instruction::CopyTargetAbility { group: 0 }],
    }],
    extras: &CardExtras {
        fixed_entry_counters: &[(CounterKind::Film, 3)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
