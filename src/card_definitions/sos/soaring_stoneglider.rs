// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::ExileGraveyardCards(2),
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        cost_alternatives: &[ManaCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 3,
        }],
        mana_alternative_replaces_additional: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
