// Complete Oracle/XMage: two main-relative combats and a repeated source-independent delay.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::AdditionalCombatsAfterCurrentMain(2),
            Instruction::ScheduleBeginCombatThisTurn { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapCreaturesThatAttackedThisTurn],
    }],
    ..CardRules::EMPTY
};
