// Direct typed behavior from the pinned Oracle and XMage sources.
use crate::engine::temporary_tokens::TokenCleanup;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: generic(2),
                skip: 1,
            },
            Instruction::CreateTokensThenCleanup {
                token: Card(5309),
                amount: 1,
                attacking: true,
                cleanup: TokenCleanup::EndCombatSacrifice,
            },
        ],
    }],
    ..CardRules::EMPTY
};
