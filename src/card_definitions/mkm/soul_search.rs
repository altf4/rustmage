pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[Instruction::RevealTargetHandExileNonlandReward {
            maximum: 1,
            token: Card::from_catalogue_id(5477),
        }],
    ),
    ..CardRules::EMPTY
};
