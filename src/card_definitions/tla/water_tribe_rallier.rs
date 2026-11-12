// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 5,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::WaterbendCost(5),
            Instruction::SelectLibrary(LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::CreaturePowerAtMost(3),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
