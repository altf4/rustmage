// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::LandEnteredAndPrimeLandCount,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensThenCounters {
            token: Card::from_catalogue_id(5679),
            amount: 1,
            kind: CounterKind::PlusOne,
            counters: AmountValue::Stat(StatValue::CONTROLLER_LANDS),
        }],
    }],
    ..CardRules::EMPTY
};
