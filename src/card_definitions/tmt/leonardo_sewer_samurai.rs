// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::DOUBLE_STRIKE,
    static_groups: &[StaticAbilityGroup {
        line: 0, first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardSmallCreaturesWithFinality],
    }],
    extras: &CardExtras {
        sneak_costs: &[ManaCost { colored: [2,0,0,0,0,0], generic: 2 }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
