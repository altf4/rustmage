// Direct typed Oracle behavior; full XMage source pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            }),
            TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            },
            TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::LoseLife(PlayerSet::Targeted, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::SelectGroup(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
            Instruction::SelectGroup(2),
            Instruction::Grow {
                subject: Subject::Targets,
                power: -1,
                toughness: -1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
