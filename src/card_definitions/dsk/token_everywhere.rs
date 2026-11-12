// Its five basic land types each supply their intrinsic mana ability.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    deck_limit: None,
    ..CardRules::EMPTY
};
