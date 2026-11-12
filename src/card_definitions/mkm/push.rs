pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: true,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[Instruction::Destroy],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(3287)),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
