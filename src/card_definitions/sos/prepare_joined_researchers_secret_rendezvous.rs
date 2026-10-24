// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[Instruction::Draw(PlayerSet::Each, 3)],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/joined_researchers_secret_rendezvous"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
