pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target artifact or enchantment.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16 | 32,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Put a +1/+1 counter on target creature you control. It gains indestructible until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::INDESTRUCTIBLE,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
