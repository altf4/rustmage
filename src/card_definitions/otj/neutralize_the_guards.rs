// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Targeted,
                power: -1,
                toughness: -1,
            },
            Instruction::Surveil(2),
        ],
    ),
    ..CardRules::EMPTY
};
