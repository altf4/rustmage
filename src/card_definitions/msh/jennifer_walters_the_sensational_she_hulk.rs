// Complete front face, directly authored from the pinned Oracle and Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StaticCastRestriction(0, 1)],
    }],
    extras: &CardExtras {
        static_cast_restriction: 1,
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [2, 0, 0, 0, 1, 0],
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
        targets: TargetPlan::NONE,
        program: &[Instruction::TransformSource],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
