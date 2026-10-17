// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Draw four cards.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Draw(PlayerSet::Controller, 4)],
        },
        SpellMode {
            text: "Splatter Technique deals 4 damage to each creature and planeswalker.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamageBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 65,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords(0),
                    subtype: None,
                    max_mana: 4294967295,
                }],
                players: None,
                amount: 4,
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
