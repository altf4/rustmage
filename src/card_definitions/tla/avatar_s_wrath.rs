// Direct typed behavior from Oracle and the pinned XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec { rule: TargetRule::Creature, min: 0, max: 1 }),
        &[
            Instruction::AirbendOtherCreatures,
            Instruction::ProhibitOpponentNonhandCastsUntilNextTurn,
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};
