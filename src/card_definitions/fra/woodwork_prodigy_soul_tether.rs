// Draft typed behavior; remains gated pending complete rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::SourceUnprepared,
            event: &TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PrepareSource],
    }],
    extras: &CardExtras {
        prepared_spell: Some(Card(5441)),
        enters_prepared: false,
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
