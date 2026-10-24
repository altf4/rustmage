// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::FLYING,
        }],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/encouraging_aviator_jump"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
