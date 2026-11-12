const OTHER_CREATURES: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1,
    exclude: 0,
    subtype: None,
    basic: false,
    other: true,
    cards_only: false,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 3,
            },
            alternatives: &[
                ManaCost {
                    colored: [1, 0, 0, 0, 1, 0],
                    generic: 3,
                },
                ManaCost {
                    colored: [2, 0, 0, 0, 0, 0],
                    generic: 3,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::Stat(OTHER_CREATURES),
        }],
        as_turned_face_up_counters: &[(
            CounterKind::PlusOne,
            TurnUpCounterAmount::Stat(OTHER_CREATURES),
        )],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
