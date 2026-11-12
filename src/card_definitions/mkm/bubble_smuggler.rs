// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 5,
        })),
        as_turned_face_up_counters: &[(CounterKind::PlusOne, TurnUpCounterAmount::Fixed(4))],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
