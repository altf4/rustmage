// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter target spell unless its controller pays {2}.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
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
            text: "Destroy target enchantment.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 32,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Target creature has base power and toughness 5/5 until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::SetBaseStats {
                subject: Subject::Targets,
                power: Some(5),
                toughness: Some(5),
            }],
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
