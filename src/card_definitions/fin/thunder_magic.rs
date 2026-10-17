// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Thunder — {0} — Thunder Magic deals 2 damage to target creature.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Damage(2)],
        },
        SpellMode {
            text: "Thundara — {3} — Thunder Magic deals 4 damage to target creature.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Damage(4)],
        },
        SpellMode {
            text: "Thundaga — {5}{R} — Thunder Magic deals 8 damage to target creature.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Damage(8)],
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
    extras: &CardExtras {
        tiered_costs: Some(&[
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 3,
            },
            ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 5,
            },
        ]),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
