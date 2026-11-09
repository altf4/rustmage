pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: true,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ReturnToHand],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 3,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileBattlefieldSourceReturn {
            transformed: true,
            under_owner: true,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
