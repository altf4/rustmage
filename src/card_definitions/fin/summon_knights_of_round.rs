// Direct typed behavior from the paired Oracle and pinned XMage sources.
const OTHER_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: true,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

const fn knights(number: u16) -> TriggeredAbility {
    TriggeredAbility {
        condition: TriggerCondition::Chapter { number },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5236),
            amount: 3,
            tapped: false,
        }],
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::INDESTRUCTIBLE,
    triggers: &[
        knights(1),
        knights(2),
        knights(3),
        knights(4),
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 5 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GrowCreatures {
                    filter: OTHER_CREATURES,
                    power: 2,
                    toughness: 2,
                },
                Instruction::PutCreatureCounters {
                    filter: OTHER_CREATURES,
                    kind: CounterKind::Indestructible,
                    amount: 1,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
