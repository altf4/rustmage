// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
            TargetSpec::one(TargetRule::Creature),
        ]),
        &[
            Instruction::CreateTokensFor {
                players: PlayerSet::Targeted,
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
            Instruction::SelectGroup(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FLYING,
            },
            Instruction::Untap,
        ],
    ),
    ..CardRules::EMPTY
};
