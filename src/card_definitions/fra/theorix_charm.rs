// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter target noncreature spell unless its controller pays {2}.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::SpellKinds {
                kinds: 0,
                excluded: 1,
            })),
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0, 0, 0, 0, 0, 0],
                    generic: 2,
                },
                subject: CounterSubject::Targets,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        SpellMode {
            text: "Target creature gets -2/-2 until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: -2,
                toughness: -2,
            }],
        },
        SpellMode {
            text: "Mill three cards, then draw a card.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
                Instruction::Draw(PlayerSet::Controller, 1),
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
