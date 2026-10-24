// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
            TargetSpec::one(TargetRule::Creature),
        ]),
        &[
            Instruction::SelectGroup(0),
            Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Targeted,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::SelectGroup(1),
            Instruction::May { skip: 2 },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DOUBLE_STRIKE,
            },
            Instruction::Skip(1),
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::LIFELINK,
            },
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
