const CONTROLLED_CREATURES: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::AttackingOrBlocking,
        }),
        &[Instruction::DamageScaled(AmountValue::Stat(
            StatValue::Sum(&[
                StatValue::Fixed(2),
                StatValue::CountUnion(&[
                    CONTROLLED_CREATURES,
                    CountFilter {
                        kind: 0,
                        subtype: Some("Spacecraft"),
                        ..CONTROLLED_CREATURES
                    },
                ]),
            ]),
        ))],
    ),
    ..CardRules::EMPTY
};
