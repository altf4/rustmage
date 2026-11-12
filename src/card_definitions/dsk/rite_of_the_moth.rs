// Directly authored from the paired Oracle and pinned XMage sources.
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
            CounterKind::Finality,
        )],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [2, 0, 1, 0, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
