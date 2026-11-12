// Direct typed behavior from the pinned Oracle and XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::InstantSorcery,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::SourceUnsolved(&StepGuard::Expression {
                    value: StatValue::InstantSorcerySpellsCastThisTurn,
                    minimum: Some(4),
                    maximum: None,
                }),
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SolveSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::InstantSorcery,
                    ordinal: None,
                },
                guard: StepGuard::SourceSolved,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    static_groups: GROUPS,
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
