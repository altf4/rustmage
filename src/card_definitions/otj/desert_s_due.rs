const DESERTS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Desert"),
    basic: false,
    other: false,
    cards_only: false,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::GrowScaled {
            subject: Subject::Targets,
            amount: AmountValue::Stat(StatValue::Sum(&[StatValue::Fixed(2), DESERTS])),
            power: -1,
            toughness: -1,
        }],
    ),
    ..CardRules::EMPTY
};
