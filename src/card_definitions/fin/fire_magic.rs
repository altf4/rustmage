// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Fire — {0} — Fire Magic deals 1 damage to each creature.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamageBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords(0),
                    subtype: None,
                    max_mana: 4294967295,
                }],
                players: None,
                amount: 1,
            }],
        },
        SpellMode {
            text: "Fira — {2} — Fire Magic deals 2 damage to each creature.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamageBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords(0),
                    subtype: None,
                    max_mana: 4294967295,
                }],
                players: None,
                amount: 2,
            }],
        },
        SpellMode {
            text: "Firaga — {5} — Fire Magic deals 3 damage to each creature.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamageBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords(0),
                    subtype: None,
                    max_mana: 4294967295,
                }],
                players: None,
                amount: 3,
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
    extras: &CardExtras {
        tiered_costs: Some(&[
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 5,
            },
        ]),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
