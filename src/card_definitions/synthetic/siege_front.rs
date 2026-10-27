pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        transforming_double_face: true,
        battle_defense: 5,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
