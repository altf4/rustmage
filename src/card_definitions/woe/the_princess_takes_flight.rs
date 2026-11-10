// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::ExileTargetsLinkedToSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: one(TargetRule::creature_you_control()),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 2,
                    toughness: 2,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::FLYING,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnLinkedExileToBattlefield],
        },
    ],
    ..CardRules::EMPTY
};
