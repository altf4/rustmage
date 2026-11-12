// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpellAlternativeManaCost(ManaCost {
            colored: [1, 1, 1, 1, 1, 0],
            generic: 0,
        })],
    }],
    extras: &CardExtras {
        opening_hand_battlefield: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
