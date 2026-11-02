// Full pinned Oracle and XMage: count the controlled Mount/Vehicle union once.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 65,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::DamageScaled(AmountValue::Sum(&[
            AmountValue::Fixed(2),
            AmountValue::Stat(StatValue::CountUnion(&[
                CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Mount"),
                    basic: false,
                    other: false,
                    cards_only: false,
                },
                CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Vehicle"),
                    basic: false,
                    other: false,
                    cards_only: false,
                },
            ])),
        ]))],
    ),
    ..CardRules::EMPTY
};
