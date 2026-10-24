// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: false,
        }),
        &[
            Instruction::Draw(PlayerSet::Targeted, 2),
            Instruction::LoseLife(PlayerSet::Targeted, 2),
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/scheming_silvertongue_sign_in_blood"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
