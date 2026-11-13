// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 7),
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::WaterbendCost(3),
                Instruction::GrantKeywords {
                    subject: Subject::Source,
                    keywords: Keywords::CANT_BE_BLOCKED,
                },
            ],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
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
                filter: SearchFilter::Subtype("Island"),
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            })],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
