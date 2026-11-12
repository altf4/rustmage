use crate::engine::temporary_tokens::TokenCleanup;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTemporaryTokensThenCleanup {
            token: Card(5266),
            amount: AmountValue::Fixed(1),
            keywords: Keywords::HASTE,
            cleanup: TokenCleanup::OwnEndStepSacrifice,
        }],
    }],
    ..CardRules::EMPTY
};
