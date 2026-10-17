// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::GrowCreatures {
            filter: CreatureFilter {
                relation: -1,
                other: false,
                subtype: None,
                keyword: Keywords(0),
                attacking: true,
                tokens: None,
                artifact: false,
            },
            power: 2,
            toughness: 0,
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
        flashback_cost: Some(ManaCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
