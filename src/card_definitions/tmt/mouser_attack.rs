// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create a 1/1 colorless Robot artifact creature token.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5277),
                amount: 1,
                tapped: false,
            }],
        },
        SpellMode {
            text: "Target creature gets +3/+0 and gains first strike until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 3,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords(512),
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
