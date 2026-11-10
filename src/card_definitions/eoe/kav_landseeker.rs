// Direct typed behavior from the pinned Oracle and XMage sources.
use crate::engine::temporary_tokens::TokenCleanup;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensThenCleanup {
            token: Card(5238),
            amount: 1,
            attacking: false,
            cleanup: TokenCleanup::NextOwnTurnEndStepSacrifice,
        }],
    }],
    ..CardRules::EMPTY
};
