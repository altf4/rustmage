// Adventure spell of Virtue of Strength, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 3,
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
        adventure_parent: Some(Card(4909)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
