const CONTROLLED_CREATURE: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCreatureCounters {
                filter: CONTROLLED_CREATURE,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::GrantCreatures {
                filter: CONTROLLED_CREATURE,
                keywords: Keywords::TRAMPLE,
            },
        ],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 5,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
