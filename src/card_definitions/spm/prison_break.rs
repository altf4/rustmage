pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[Instruction::ReturnToBattlefieldWithCounter(
            CounterKind::PlusOne,
        )],
    ),
    extras: &CardExtras {
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
