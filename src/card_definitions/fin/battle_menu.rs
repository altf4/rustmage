// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Attack — Create a 2/2 white Knight creature token.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5236),
                amount: 1,
                tapped: false,
            }],
        },
        SpellMode {
            text: "Ability — Target creature gets +0/+4 until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: 0,
                toughness: 4,
            }],
        },
        SpellMode {
            text: "Magic — Destroy target creature with power 4 or greater.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::PowerAtLeast(4),
            })),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Item — You gain 4 life.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
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
