const ATTACKED_WITH_SPACECRAFT: StatValue = StatValue::AtLeast {
    value: &StatValue::DeclaredSubtypeAttackersThisTurn("Spacecraft"),
    minimum: 1,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: ATTACKED_WITH_SPACECRAFT,
                multiplier: 3,
            },
        )],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 6,
            mana_component: true,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::Destroy],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
