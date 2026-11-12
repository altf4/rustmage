pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
            Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::DiesOnce,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5329),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
