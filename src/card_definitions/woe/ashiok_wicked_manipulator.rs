// Pinned Oracle/XMage behavior.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ReplaceLifePaymentWithLibraryExile],
    }],
    extras: &CardExtras {
        starting_loyalty: 5,
        ..CardExtras::EMPTY
    },
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(1),
            targets: TargetPlan::NONE,
            program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 2,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Any,
                    optional: false,
                    reveal: false,
                    remainder: LibraryRemainder::Hand,
                },
                look: LibraryLook::Fixed(2),
                destination: LibraryPickDestination::Exile,
                public: false,
                maximum_mana_value: None,
            })],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-2),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5478),
                amount: 2,
                tapped: false,
            }],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-7),
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::ExileTopLibrary {
                players: PlayerSet::Targeted,
                amount: AmountValue::Stat(StatValue::TotalManaValue(CountFilter {
                    zone: CountZone::Exile,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                })),
            }],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
