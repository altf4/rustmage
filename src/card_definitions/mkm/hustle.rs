pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(2110)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
