// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::PutCounterOnCreatureThisTurn,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 2)],
    }],
    ..CardRules::EMPTY
};
