// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter target spell.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
            program: &[Instruction::Counter],
        },
        SpellMode {
            text: "Tap one or two target creatures.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 1,
                max: 2,
            }),
            program: &[Instruction::Tap],
        },
        SpellMode {
            text: "Choose both",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Spell),
                TargetSpec {
                    rule: TargetRule::Creature,
                    min: 1,
                    max: 2,
                },
            ]),
            program: &[
                Instruction::Counter,
                Instruction::SelectGroup(1),
                Instruction::Tap,
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
