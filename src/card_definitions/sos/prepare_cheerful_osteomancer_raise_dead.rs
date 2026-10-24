// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
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
        &[Instruction::ReturnToHand],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/cheerful_osteomancer_raise_dead"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
