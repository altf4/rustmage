// Complete pinned front-face rules; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::Any,
                guard: StepGuard::Always,
            },
            guard: StepGuard::Expression {
                value: StatValue::CreaturesDiedThisTurn,
                minimum: Some(3),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PrepareSource],
    }],
    extras: &CardExtras {
        prepared_spell: Some(Card(5459)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
