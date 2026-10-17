// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::SpellCharacteristics {
            colors: 0,
            min_mana: 0,
            max_mana: 2,
        })),
        &[Instruction::Counter],
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
        omen_parent: Some(Card(3624)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
