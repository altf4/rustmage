// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target creature gets +3/+3 until end of turn.",
            targets: one(TargetRule::Creature),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: 3,
                toughness: 3,
            }],
        },
        SpellMode {
            text: "Target creature gets -3/-3 until end of turn.",
            targets: one(TargetRule::Creature),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: -3,
                toughness: -3,
            }],
        },
        SpellMode {
            text: "Choose both.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
                TargetSpec::one(TargetRule::Creature),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 3,
                    toughness: 3,
                },
                Instruction::SelectGroup(1),
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: -3,
                    toughness: -3,
                },
            ],
        },
    ]),
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [1, 0, 1, 0, 0, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
