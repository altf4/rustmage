// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: false,
        }),
        &[Instruction::Draw(PlayerSet::Targeted, 3)],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/emeritus_of_ideation_ancestral_recall"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
