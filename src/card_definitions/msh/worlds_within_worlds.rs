// Direct whole Oracle behavior; all hand choices precede one entry batch.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ExileAllCreaturesRemember,
            Instruction::DeployHandCreaturesEach,
            Instruction::ReturnExiledResultToHands,
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};
