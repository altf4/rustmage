// Forest mana is derived from the current basic land subtype.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    deck_limit: None,
    extras: &CardExtras { basic_land_subtype_mana: true, ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
