// Direct typed behavior from complete pinned Oracle and XMage sources.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        colored: [0; 6],
        generic: 0,
        alternatives: &[],
        additional: None,
        mana_component: false,
        tap: false,
        sacrifice: false,
        exile_self: false,
        discard: false,
        life: 0,
        remove: None,
        loyalty: Some(amount),
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        starting_loyalty: 3,
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
            program: &[Instruction::CreateTokens {
                token: Card(5306),
                amount: 1,
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
            cost: loyalty(-2),
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: -1,
                other: false,
                nonland: false,
                tapped: true,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::Damage(4)],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-8),
            targets: TargetPlan::NONE,
            program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::All,
                    filter: LibrarySelectionFilter::AllOf(&[
                        LibrarySelectionFilter::Kinds(1 | 16 | 32 | 64 | 256),
                        LibrarySelectionFilter::ExcludingKinds(2),
                    ]),
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::Shuffle,
                },
                look: LibraryLook::Amount(AmountValue::Stat(StatValue::ControllerLife)),
                destination: LibraryPickDestination::Battlefield,
                public: false,
                maximum_mana_value: Some(AmountValue::Fixed(3)),
            })],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
