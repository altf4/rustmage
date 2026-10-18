// Ovika creates ordinary 1/1 red Phyrexian Goblins; haste is an until-cleanup effect.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    deck_limit: None,
    ..CardRules::EMPTY
};
