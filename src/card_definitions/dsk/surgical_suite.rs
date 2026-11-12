// The physical Room owns this door's battlefield abilities.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room_parent: Some(Card::from_catalogue_id(4269)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
