// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::BeginCombat,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::Expression {
                    value: StatValue::CardsDrawnThisTurn,
                    minimum: Some(2),
                    maximum: None,
                },
            },
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: AmountValue::Stat(StatValue::CardsDrawnAfterFirstThisTurn),
            }],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NoMaximumHandSize],
    }],
    ..CardRules::EMPTY
};
