// Reveal through the second land; randomize only the nonland remainder actually revealed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING.union(Keywords::DEATHTOUCH),
    activated: &[ActivatedAbility {
        exhaust: true,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 4,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::All,
                    filter: LibrarySelectionFilter::Kinds(2),
                    optional: false,
                    reveal: true,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::UntilKindsCount { kinds: 2, count: 2 },
                destination: LibraryPickDestination::BattlefieldTapped,
                public: true,
                maximum_mana_value: None,
            }),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
