pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    extras: &CardExtras {
        adventure_parent: Some(Card::from_catalogue_id(2340)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
