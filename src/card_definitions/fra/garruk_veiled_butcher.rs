// Incomplete gated draft: the static replacement is implemented; loyalty abilities are pending.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        starting_loyalty: 5,
        zone_replacement: Some(ZoneReplacementRule::ExileOpposingCreatureDeathPlain),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
