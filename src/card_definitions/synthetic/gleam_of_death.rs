// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(6)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(12),
                count: MilledCount::All,
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
        adventure_parent: Some(Card(1764)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
