// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 3,
                toughness: -3,
            },
            Instruction::ChangeTargetCreatureForm(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&[]),
                base: None,
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            }),
        ],
    ),
    keywords: Keywords::CHANGELING,
    ..CardRules::EMPTY
};
