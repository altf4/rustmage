// Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 0, 0, 1]))],
        },
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            })],
        },
    ],
    ..CardRules::EMPTY
};
