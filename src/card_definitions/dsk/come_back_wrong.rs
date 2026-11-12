pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::DestroyTargetsRememberCreatureCards,
            Instruction::ReturnRememberedCardsForOwnEndStep,
        ],
    ),
    ..CardRules::EMPTY
};
