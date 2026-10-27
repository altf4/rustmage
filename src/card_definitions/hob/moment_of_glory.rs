pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::IfCastFromGraveyard { skip: 1 },
            Instruction::PutOtherControlledCreatureCounters {
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
