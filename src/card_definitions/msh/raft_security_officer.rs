// Complete Oracle: target remains any creature; power changes only its locked cost.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 2,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: one(TargetRule::Creature),
        program: &[Instruction::Tap],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        activation_target_reductions: &[(
            0,
            TargetActivationReduction {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::PowerAtMost(3),
                },
                generic: 1,
            },
        )],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
