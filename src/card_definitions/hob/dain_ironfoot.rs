// Complete Oracle body and pinned XMage class; replacement-created tokens are all captured.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensReflexiveWithSubjects {
                token: Card(5406),
                amount: 1,
                ability: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantEquippedAttackers(Keywords::DOUBLE_STRIKE)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::AttachReflexiveSubjects],
        },
    ],
    ..CardRules::EMPTY
};
