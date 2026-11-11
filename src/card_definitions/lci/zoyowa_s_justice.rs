pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1 | 16,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 1,
            max_mana: u32::MAX,
        }),
        &[Instruction::ShuffleTargetThenDiscover],
    ),
    ..CardRules::EMPTY
};
