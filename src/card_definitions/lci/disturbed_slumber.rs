pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::AnimateTargets(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Dinosaur"]),
                base: Some((4, 4)),
                keywords: Keywords::REACH.union(Keywords::HASTE),
                duration: AnimationDuration::EndOfTurn,
            }),
            Instruction::RequireTargetsBlocked { group: 0 },
        ],
    ),
    ..CardRules::EMPTY
};
