const TREASURE: Card = Card::from_catalogue_id(5302);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        &[
            Instruction::CreateTokens {
                token: TREASURE,
                amount: 1,
                tapped: false,
            },
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Treasure"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                power: 2,
                toughness: 2,
            },
        ],
    ),
    ..CardRules::EMPTY
};
