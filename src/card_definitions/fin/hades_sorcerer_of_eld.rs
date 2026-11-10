pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::VIGILANCE,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayFromGraveyardDuringOwnTurn],
    }],
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileControllerGraveyard),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
