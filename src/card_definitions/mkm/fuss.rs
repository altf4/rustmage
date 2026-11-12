pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PutCreatureCounters {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: true,
                tokens: None,
                artifact: false,
            },
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(1654)),
        cost_alternatives: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
