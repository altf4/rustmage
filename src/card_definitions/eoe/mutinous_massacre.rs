pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseManaValueParityDestroyCreatures,
            Instruction::TakeCreaturesUntilEndTurn,
        ],
    ),
    ..CardRules::EMPTY
};
