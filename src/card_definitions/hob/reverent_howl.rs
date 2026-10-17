// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target player draws two cards and loses 2 life.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            })),
            program: &[
                Instruction::Draw(PlayerSet::Targeted, 2),
                Instruction::LoseLife(PlayerSet::Targeted, 2),
            ],
        },
        SpellMode {
            text: "Target creature gets +2/+2 and gains lifelink until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 2,
                    toughness: 2,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::LIFELINK,
                },
            ],
        },
    ]),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
