// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Ruinous Rampage deals 3 damage to each opponent.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 3)],
        },
        SpellMode {
            text: "Exile all artifacts with mana value 3 or less.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 16,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: 3,
                }],
                exile: true,
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
