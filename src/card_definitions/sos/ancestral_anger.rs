// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::TRAMPLE,
            },
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::Stat(StatValue::Sum(&[
                    StatValue::Fixed(1),
                    StatValue::NamedCount {
                        filter: CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: true,
                        },
                        name: "Ancestral Anger",
                    },
                ])),
                power: 1,
                toughness: 0,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
