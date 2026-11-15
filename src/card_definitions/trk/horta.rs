// Pinned Oracle and XMage: basic landcycling is a hand-zone discard activation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 6),
    keywords: Keywords::TRAMPLE,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        cycling: true,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::BasicLand,
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
