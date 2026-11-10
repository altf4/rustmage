pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::DamageWithControllerScaled {
            damage: 5,
            controller_damage: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 16,
                exclude: 0,
                subtype: Some("Equipment"),
                basic: false,
                other: false,
                cards_only: false,
            })),
        }],
    ),
    ..CardRules::EMPTY
};
