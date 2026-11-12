// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
        ]),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::MENACE,
            },
            Instruction::SelectGroup(1),
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Targeted,
                power: 2,
                toughness: 0,
            },
        ],
    ),
    ..CardRules::EMPTY
};
