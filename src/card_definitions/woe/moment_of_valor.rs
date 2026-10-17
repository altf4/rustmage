// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Untap target creature. It gets +1/+0 and gains indestructible until end of turn.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 1,
                max: 1,
            }),
            program: &[
                Instruction::Untap,
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 1,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::INDESTRUCTIBLE,
                },
            ],
        },
        SpellMode {
            text: "Destroy target creature with power 4 or greater.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::PowerAtLeast(4),
            })),
            program: &[Instruction::Destroy],
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
