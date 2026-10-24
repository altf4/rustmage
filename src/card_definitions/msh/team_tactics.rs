// Teamwork declaration is frozen before targets; only the additional cost taps creatures.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DOUBLE_STRIKE,
            },
            Instruction::IfTeamwork { skip: 1 },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::TRAMPLE,
            },
        ],
    ),
    extras: &CardExtras {
        teamwork: Some(TeamworkSpec {
            minimum: 1,
            paid_targets: None,
            normal_modes: 0,
            paid_modes: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
