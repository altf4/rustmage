const MESMERIZE: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::Chapter { number: 1 },
    targets: one(TargetRule::Creature),
    program: &[Instruction::GrantKeywords {
        subject: Subject::Targets,
        keywords: Keywords::CANT_BE_BLOCKED,
    }],
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    triggers: &[
        MESMERIZE,
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            ..MESMERIZE
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::TapPermanentGroup(&[BattlefieldFilter {
                    kinds: 2,
                    relation: 1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }]),
                Instruction::ExileBattlefieldSourceReturn {
                    transformed: false,
                    under_owner: false,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
