pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::SacrificeOtherCreaturesRememberGraveyard,
            Instruction::ReturnGraveyardCreaturesToOwnersExceptRemembered,
        ],
    }],
    ..CardRules::EMPTY
};
