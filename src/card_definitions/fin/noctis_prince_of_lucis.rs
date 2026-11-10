// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::LIFELINK,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastFromGraveyardWithLife {
            filter: CastFilter::Artifact,
            life: 3,
            entry_counter: CounterKind::Finality,
        }],
    }],
    ..CardRules::EMPTY
};
