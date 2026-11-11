// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            }),
        ]),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FIRST_STRIKE,
            },
            Instruction::SelectGroup(1),
            Instruction::RemovePlayerCreatureKeywords {
                players: PlayerSet::Targeted,
                keywords: Keywords::FIRST_STRIKE.union(Keywords::DOUBLE_STRIKE),
            },
        ],
    ),
    ..CardRules::EMPTY
};
