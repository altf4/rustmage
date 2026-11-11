// Apply the shrink to the resolution cohort and the replacement to each actual death.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Each,
                power: -2,
                toughness: -2,
            },
            Instruction::ExileOpposingCreaturesThatWouldDieThisTurn,
        ],
    ),
    ..CardRules::EMPTY
};
