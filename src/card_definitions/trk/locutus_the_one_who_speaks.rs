pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupScaling {
            filters: &[CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: true,
            }],
            value: StatValue::Fixed(1),
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        cycling: false,
        cost: ActivationCost {
            colored: [1, 0, 1, 0, 0, 0],
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Creature),
        program: &[
            Instruction::Damage(2),
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
