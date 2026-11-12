pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room_parent: Some(Card(1030)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
