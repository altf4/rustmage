pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec {
                rule: TargetRule::Spell,
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::PutCountersScaled {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: AmountValue::ChosenX,
            },
            Instruction::SelectGroup(1),
            Instruction::CounterUnlessPayScaled {
                amount: AmountValue::GreatestCreaturePower(CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                }),
                exile: false,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
