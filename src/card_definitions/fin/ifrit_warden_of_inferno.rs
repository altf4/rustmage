const BRIMSTONE: &[Instruction] = &[
    Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 4, 0, 0])),
    Instruction::If {
        guard: StepGuard::Expression {
            value: StatValue::SourceCounters(CounterKind::Lore),
            minimum: Some(3),
            maximum: None,
        },
        skip: 1,
    },
    Instruction::ExileBattlefieldSourceReturn {
        transformed: false,
        under_owner: false,
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(9, 9),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: -1,
                    other: true,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::FightSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: BRIMSTONE,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: BRIMSTONE,
        },
    ],
    ..CardRules::EMPTY
};
