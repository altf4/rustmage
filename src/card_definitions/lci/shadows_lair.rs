const TAP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 1, 0, 0, 0,
    ]))],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        TAP,
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 1, 0, 0, 0],
                mana_component: true,
                remove: Some((Some(CounterKind::Dread), 1)),
                ..TAP.cost
            },
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 1),
            ],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
