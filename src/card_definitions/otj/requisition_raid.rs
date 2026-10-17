// Complete printed behavior; mode combinations resolve in printed order.
// Base mana, identity and format metadata come from the pinned card facts.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy artifact",
            targets: one(object(16)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Destroy enchantment",
            targets: one(object(32)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Destroy artifact + Destroy enchantment",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(object(16)),
                TargetSpec::one(object(32)),
            ]),
            program: &[
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
            ],
        },
        SpellMode {
            text: "Counters on selected player creatures",
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Targeted,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        SpellMode {
            text: "Destroy artifact + Counters on selected player creatures",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(object(16)),
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::PutPlayerCreatureCounters {
                    players: PlayerSet::Targeted,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
        },
        SpellMode {
            text: "Destroy enchantment + Counters on selected player creatures",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(object(32)),
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::PutPlayerCreatureCounters {
                    players: PlayerSet::Targeted,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
        },
        SpellMode {
            text: "Destroy artifact + Destroy enchantment + Counters on selected player creatures",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(object(16)),
                TargetSpec::one(object(32)),
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
                Instruction::SelectGroup(2),
                Instruction::PutPlayerCreatureCounters {
                    players: PlayerSet::Targeted,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
        },
    ]),
    mode_costs: Some(&[
        generic(1),
        generic(1),
        generic(2),
        generic(1),
        generic(2),
        generic(2),
        generic(3),
    ]),
    ..CardRules::EMPTY
};
