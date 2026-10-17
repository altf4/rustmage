// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy all tapped creatures.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: Some(true),
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: 4294967295,
                }],
                exile: false,
            }],
        },
        SpellMode {
            text: "Destroy all untapped creatures.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: Some(false),
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: 4294967295,
                }],
                exile: false,
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
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
