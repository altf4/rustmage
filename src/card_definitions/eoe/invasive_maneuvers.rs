// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Spacecraft"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                minimum: 1,
                skip: 2,
            },
            Instruction::Damage(5),
            Instruction::Skip(1),
            Instruction::Damage(3),
        ],
    ),
    ..CardRules::EMPTY
};
