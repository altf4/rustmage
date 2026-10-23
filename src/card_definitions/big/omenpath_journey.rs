// Directly authored from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::SearchDifferentNames(SearchSpec {
                filter: SearchFilter::Land,
                maximum: 5,
                destination: SearchDestination::LinkedExile,
                tapped: false,
                reveal: false,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::RandomLinkedExileToBattlefield],
        },
    ],
    ..CardRules::EMPTY
};
