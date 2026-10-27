// Complete pinned Oracle and XMage modal obligations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target creature gets -5/-5 until end of turn. If that creature would die this turn, exile it instead.",
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: -5,
                    toughness: -5,
                },
                Instruction::ExileIfDiesThisTurn,
            ],
        },
        SpellMode {
            text: "Creatures target player controls get -1/-1 until end of turn.",
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::GrowPlayerCreatures {
                players: PlayerSet::Targeted,
                power: -1,
                toughness: -1,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
