// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::Discard(PlayerSet::Controller, 2),
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/spellbook_seeker_careful_study"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
