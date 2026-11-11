const OTHER_CAVES: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Cave"),
    basic: false,
    other: true,
    cards_only: false,
};
const GRAVEYARD_CAVES: CountFilter = CountFilter {
    zone: CountZone::Graveyard,
    other: false,
    cards_only: true,
    ..OTHER_CAVES
};
const MANA: ActivatedAbility = ActivatedAbility {
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
        0, 0, 0, 0, 0, 1,
    ]))],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        MANA,
        ActivatedAbility {
            guard: ActivationGuard::StatAtLeast {
                value: StatValue::Sum(&[
                    StatValue::Count(OTHER_CAVES),
                    StatValue::Count(GRAVEYARD_CAVES),
                ]),
                minimum: 3,
            },
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AnimateSource(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Elemental"]),
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            })],
            ..MANA
        },
    ],
    ..CardRules::EMPTY
};
