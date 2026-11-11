pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpellCastLimit(1)],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 4,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Any),
        program: &[Instruction::Damage(5)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
