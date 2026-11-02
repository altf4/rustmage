// Only successful exact exiles add counters/links; copied templates keep their immutable origin.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::LinkedExileCreature {
            ability_origin: None,
        }),
        program: &[Instruction::CopySourceFromLinkedExile { base: (0, 0) }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        x_symbols: 1,
        entry_exile_counters: Some(EntryExileCountersSpec {
            filter: SearchFilter::Creature,
            kind: CounterKind::PlusOne,
            per_card: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
