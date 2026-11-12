pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 2,
        }),
        &[
            Instruction::ReturnToHand,
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
