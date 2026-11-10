const OWN_CREATURE: TargetPlan = one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5544),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: OWN_CREATURE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::AddLegendaryCreatureTypes {
                    subject: Subject::Targets,
                    types: &["Spider", "Hero"],
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: OWN_CREATURE,
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DOUBLE_STRIKE,
            }],
        },
    ],
    ..CardRules::EMPTY
};
