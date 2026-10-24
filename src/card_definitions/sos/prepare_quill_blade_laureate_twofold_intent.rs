// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DOUBLE_STRIKE,
            },
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/quill_blade_laureate_twofold_intent"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
