// Directly authored from pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING
        .union(Keywords::VIGILANCE)
        .union(Keywords::DEFENDER),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttackWithoutDefenderIfScried],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 3,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        program: &[Instruction::Surveil(1)],
    }],
    ..CardRules::EMPTY
};
