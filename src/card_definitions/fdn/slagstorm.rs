// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Slagstorm deals 3 damage to each creature.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamageBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: 4294967295,
                }],
                players: None,
                amount: 3,
            }],
        },
        SpellMode {
            text: "Slagstorm deals 3 damage to each player.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamagePlayers(PlayerSet::Each, 3)],
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
