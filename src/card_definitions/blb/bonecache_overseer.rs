const DEPARTURES: StatValue = StatValue::AtLeast {
    value: &StatValue::CardsLeftGraveyardThisTurn,
    minimum: 3,
};
const FOOD: StatValue = StatValue::FoodSacrificedThisTurn;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::StatAtLeast {
            value: StatValue::Sum(&[DEPARTURES, FOOD]),
            minimum: 1,
        },
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            life: 1,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
