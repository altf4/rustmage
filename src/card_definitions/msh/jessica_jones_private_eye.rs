// Direct typed behavior from complete frozen Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlayScaled {
            amount: AmountValue::SourcePower,
            next_turn: false,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        activation_put_counters: &[(0, CounterKind::Stun, 1)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
