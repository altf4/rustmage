// Recovery is optional, nontargeted, and includes land cards already in the graveyard.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::CollectGraveyard,
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(2),
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::Hand,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
