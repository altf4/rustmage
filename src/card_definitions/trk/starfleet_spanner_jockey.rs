// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::DEFENDER,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::DEFENDER)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttackWithoutDefenderIf(
                StatValue::NoncreatureSpellsCastThisTurn,
            )],
        },
    ],
    ..CardRules::EMPTY
};
