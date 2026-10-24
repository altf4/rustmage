// Normal casting chooses one mode; teamwork requires both modes and their targets.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target creature gains deathtouch until end of turn.",
            targets: one(TargetRule::Creature),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DEATHTOUCH,
            }],
        },
        SpellMode {
            text: "Target creature gets -2/-2 until end of turn.",
            targets: one(TargetRule::Creature),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: -2,
                toughness: -2,
            }],
        },
        SpellMode {
            text: "Both, in printed order.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
                TargetSpec::one(TargetRule::Creature),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::DEATHTOUCH,
                },
                Instruction::SelectGroup(1),
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: -2,
                    toughness: -2,
                },
            ],
        },
    ]),
    extras: &CardExtras {
        teamwork: Some(TeamworkSpec {
            minimum: 3,
            paid_targets: None,
            normal_modes: 0b011,
            paid_modes: 0b100,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
