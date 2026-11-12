// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target creature with power 4 or greater.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::PowerAtLeast(4),
            }),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Earthbend 3.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[
                Instruction::AnimateEarthbendTargets,
                Instruction::RegisterEarthbendReturn,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 3,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
