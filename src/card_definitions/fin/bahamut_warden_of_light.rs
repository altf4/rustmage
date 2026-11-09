const WINGS: &[Instruction] = &[
    Instruction::PutCreatureCountersAndRemember {
        filter: CreatureFilter {
            relation: 0,
            other: true,
            subtype: None,
            keyword: Keywords::NONE,
            attacking: false,
            tokens: None,
            artifact: false,
        },
        kind: CounterKind::PlusOne,
        amount: 1,
    },
    Instruction::GrantRememberedCreatures(Keywords::FLYING),
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: WINGS,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: WINGS,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: one(object(0)),
            program: &[
                Instruction::Destroy,
                Instruction::ExileBattlefieldSourceReturn {
                    transformed: false,
                    under_owner: false,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
