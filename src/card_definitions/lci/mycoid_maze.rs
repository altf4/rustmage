const GREEN_MANA: ActivatedAbility = ActivatedAbility {
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
        0, 0, 0, 0, 1, 0,
    ]))],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        GREEN_MANA,
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 3,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(1),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            })],
            ..GREEN_MANA
        },
    ],
    ..CardRules::EMPTY
};
