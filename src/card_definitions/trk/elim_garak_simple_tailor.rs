// Oracle and official TRK release notes: one trigger for each dilemma faced.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FaceDilemma(CastRelation::You),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::Opponent, 2),
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
