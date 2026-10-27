// Full Oracle; the player is the sole target, not any attacking creature.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: false,
        }),
        &[
            Instruction::ExileAttackingCreatures(PlayerSet::Targeted),
            Instruction::MayPlayer {
                player: PlayerSet::Targeted,
                skip: 1,
            },
            Instruction::SearchTargetPlayerMovedCount(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 0,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
