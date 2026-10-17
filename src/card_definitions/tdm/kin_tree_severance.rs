// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 3,
            max_mana: 4294967295,
        })),
        &[Instruction::Exile],
    ),
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
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 0, 0, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 0, 1, 0, 1, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
