// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
            Instruction::IfSubject {
                subject: Subject::Targets,
                filter: CreatureFilter {
                    relation: -1,
                    other: false,
                    subtype: Some("Goat"),
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                skip: 1,
            },
            Instruction::Grow {
                subject: Subject::Targets,
                power: 3,
                toughness: 0,
            },
        ],
    ),
    ..CardRules::EMPTY
};
