// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FLYING,
            },
            Instruction::AddCharacteristics {
                subject: Subject::Targets,
                addition: CharacteristicAddition {
                    card_types: 1 | 32,
                    creature_types: &["Horror"],
                    colors: 0,
                    duration: AnimationDuration::EndOfTurn,
                },
            },
        ],
    ),
    ..CardRules::EMPTY
};
