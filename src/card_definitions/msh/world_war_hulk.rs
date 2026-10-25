// Direct typed behavior for all three frozen Oracle chapters.
const OWN_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::NextRedOrGreenCreatureFreeThisTurn],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: one(OWN_CREATURE),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 3,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: one(OWN_CREATURE),
            program: &[
                Instruction::MultiplyTargetStats { factor: 2 },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::TRAMPLE,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
