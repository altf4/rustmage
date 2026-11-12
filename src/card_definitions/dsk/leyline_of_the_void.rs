pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        opening_hand_battlefield: true,
        zone_replacement: Some(ZoneReplacementRule::ExileOpponentGraveyardCards),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
