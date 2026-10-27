// Candidate is gated until its attack-payment rules pass full acceptance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::VIGILANCE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::VIGILANCE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Storied],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttackControllerTax {
                amount: 1,
                condition: StatValue::HasEnduringStory,
            }],
        },
    ],
    ..CardRules::EMPTY
};
