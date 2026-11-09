// Directly authored from the paired Oracle and pinned XMage sources.
const GESTALT_MODE: &[Instruction] = &[Instruction::DelayNextCreatureSpell { ability: 3 }];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlayOnSourceLoreTurns],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: GESTALT_MODE,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: GESTALT_MODE,
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedSpellCastCreature,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantCastSpellHaste],
        },
    ],
    ..CardRules::EMPTY
};
