// Staged real Power-up program; whole-card admission waits for core review and paid proof.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [1, 1, 1, 1, 1, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::SelectLibraryThenMayTransform(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 7,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::AnyOf(&[
                        LibrarySelectionFilter::Subtype("Hero"),
                        LibrarySelectionFilter::Subtype("Equipment"),
                        LibrarySelectionFilter::Subtype("Vehicle"),
                    ]),
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::Fixed(7),
                destination: LibraryPickDestination::Battlefield,
                public: false,
                maximum_mana_value: None,
            }),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        power_up_abilities: &[0],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
