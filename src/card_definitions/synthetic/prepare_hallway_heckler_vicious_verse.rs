// Vicious Verse prepared by Hallway Heckler.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[Instruction::Damage(1)],
    ),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 0,
        }],
        prepared_identity: Some("@prepare/hallway_heckler_vicious_verse"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
