// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::KindsOrSubtype {
                    kinds: 12,
                    subtype: "Faerie",
                },
                count: MilledCount::Range {
                    minimum: 1,
                    maximum: 1,
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
    keywords: Keywords::NONE,
    extras: &CardExtras {
        adventure_parent: Some(Card(3155)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
