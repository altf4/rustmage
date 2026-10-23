// XMage ShapeshifterColorlessToken; all creature types in every zone.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::CHANGELING,
    deck_limit: None,
    ..CardRules::EMPTY
};
