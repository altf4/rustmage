pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Recruit { token: Card(5241) }],
    }],
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
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 2 }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
