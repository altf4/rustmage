// Direct typed behavior from the pinned Oracle and XMage sources.
const ARTIFACT_CREATURE: AnimationSpec = AnimationSpec {
    artifact: true,
    creature_types: AnimationCreatureTypes::Unchanged,
    base: Some((4, 3)),
    keywords: Keywords::NONE,
    duration: AnimationDuration::EndOfTurn,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(1 | 16)),
        &[
            Instruction::IfEvidence { skip: 2 },
            Instruction::AnimateTargets(AnimationSpec {
                base: Some((1, 1)),
                ..ARTIFACT_CREATURE
            }),
            Instruction::Skip(1),
            Instruction::AnimateTargets(ARTIFACT_CREATURE),
        ],
    ),
    extras: &CardExtras {
        collect_evidence: Some((6, 0)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
