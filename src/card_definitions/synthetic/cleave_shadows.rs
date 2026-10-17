// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::GrowCreatures {
            filter: CreatureFilter {
                relation: 1,
                other: false,
                subtype: None,
                keyword: Keywords(0),
                attacking: false,
                tokens: None,
                artifact: false,
            },
            power: -1,
            toughness: -1,
        }],
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
        adventure_parent: Some(Card(3855)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
