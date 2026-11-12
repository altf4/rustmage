pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::CANT_BLOCK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureBecomesBlocked,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayIfEventPermanentOnBattlefield { skip: 1 },
            Instruction::ExileEventPermanentPlayThisTurn,
        ],
    }],
    ..CardRules::EMPTY
};
