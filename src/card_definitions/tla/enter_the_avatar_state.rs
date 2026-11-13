// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::AddCharacteristics {
                subject: Subject::Targets,
                addition: CharacteristicAddition {
                    card_types: 0,
                    creature_types: &["Avatar"],
                    colors: 0,
                    duration: AnimationDuration::EndOfTurn,
                },
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FLYING
                    .union(Keywords::FIRST_STRIKE)
                    .union(Keywords::LIFELINK)
                    .union(Keywords::HEXPROOF),
            },
        ],
    ),
    ..CardRules::EMPTY
};
