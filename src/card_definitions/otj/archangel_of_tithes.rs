pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    keywords: Keywords::FLYING,
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttackControllerOrPlaneswalkerTax {
                amount: 1,
                condition: StatValue::SourceUntapped,
            }],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::BlockTaxWhileAttacking(1)],
        },
    ],
    ..CardRules::EMPTY
};
