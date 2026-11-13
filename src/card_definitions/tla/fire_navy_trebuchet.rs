// Direct typed behavior from the pinned Oracle and XMage sources.
use crate::engine::temporary_tokens::TokenCleanup;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    keywords: Keywords::DEFENDER.union(Keywords::REACH),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensThenCleanup {
            token: Card(5710),
            amount: 1,
            attacking: true,
            cleanup: TokenCleanup::NextEndStepSacrifice,
        }],
    }],
    ..CardRules::EMPTY
};
