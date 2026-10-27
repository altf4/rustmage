pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 1),
            Instruction::EnsureGoblinArmy,
            Instruction::AmassGoblins(2),
        ],
    ),
    ..CardRules::EMPTY
};
