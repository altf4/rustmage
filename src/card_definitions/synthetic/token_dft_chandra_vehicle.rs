// A 3/2 colorless Vehicle token with crew 1.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    deck_limit: None,
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 2,
        }),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 1 }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
