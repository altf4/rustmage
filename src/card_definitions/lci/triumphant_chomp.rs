pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::DamageScaled(
            AmountValue::GreatestPermanentPower {
                filter: BattlefieldFilter {
                    kinds: 1,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Dinosaur"),
                    max_mana: u32::MAX,
                },
                minimum: 2,
            },
        )],
    ),
    ..CardRules::EMPTY
};
