// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Eerie,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfThisAbilityResolutionNumber { number: 1, skip: 1 },
            Instruction::Surveil(2),
            Instruction::IfThisAbilityResolutionNumber { number: 2, skip: 1 },
            Instruction::Discard(PlayerSet::Opponent, 1),
            Instruction::IfThisAbilityResolutionNumber { number: 3, skip: 1 },
            Instruction::ChooseGraveyardReturnFromEither,
        ],
    }],
    ..CardRules::EMPTY
};
