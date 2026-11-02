// Complete Oracle and pinned XMage: live controller life scales both stats.
const MODIFIER: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::ControllerLife,
    stats: Stats {
        power: -1,
        toughness: -1,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, MODIFIER)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 2,
                colored: [0, 0, 1, 0, 0, 0],
                mana_component: true,
                life: 2,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 2 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 13,
            toughness: 13,
        }),
        static_ability_groups: GROUPS,
        scaling_modifiers: &[MODIFIER],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
