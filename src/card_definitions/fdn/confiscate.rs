// Direct typed port of Confiscate: its static ability starts in layer two.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Filtered {
        base: &object(0),
        predicate: &TargetPredicate::Permanent,
    })),
    aura: Some(AuraSpec {
        kinds: 0,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Control,
        fragments: &[StaticAbilityFragment::ControlAttached],
    }],
    ..CardRules::EMPTY
};
