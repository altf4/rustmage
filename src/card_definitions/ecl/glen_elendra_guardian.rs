// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLASH.union(Keywords::FLYING),
    enters: Counters {
        plus_one: 0,
        minus_one: 1,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: Some((None, 1)),
            loyalty: None,
        },
        targets: one(TargetRule::SpellKinds {
            kinds: 0,
            excluded: 1,
        }),
        program: &[Instruction::CounterAndDrawForController { amount: 1 }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
