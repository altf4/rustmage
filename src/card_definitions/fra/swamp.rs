// Full pinned Oracle/XMage reviewed; paired DFT scenarios exercise actual land-play, entry and mana outcomes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
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
    deck_limit: None,
    extras: &CardExtras {
        has_rules_abilities: Some(true),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
