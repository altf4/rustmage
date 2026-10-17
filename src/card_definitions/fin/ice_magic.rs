// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Blizzard — {0} — Return target creature to its owner's hand.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Blizzara — {2} — Target creature's owner puts it on their choice of the top or bottom of their library.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::OwnerLibrary],
        },
        SpellMode {
            text: "Blizzaga — {5}{U} — Target creature's owner shuffles it into their library.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::ShuffleTargetIntoLibrary],
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
                colored: [0, 1, 0, 0, 0, 0],
                generic: 5,
            },
        ]),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
