// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(2),
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 2,
                },
                destination: MilledDestination::Hand,
            }),
        ],
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
        adventure_parent: Some(Card(4559)),
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 0, 0, 1, 0],
                generic: 1,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 1,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
