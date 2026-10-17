// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Ability, fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)] },
    StaticAbilityGroup { line: 1, first_layer: AbilityLayer::Ability, fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)] },
    StaticAbilityGroup { line: 2, first_layer: AbilityLayer::Rule, fragments: &[StaticAbilityFragment::CastTiming(0, CastFilter::Any)] },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords(Keywords::FLASH.0 | Keywords::FLYING.0),
    static_groups: GROUPS,
    extras: &CardExtras { static_ability_groups: GROUPS, cast_timing_grants: &[CastFilter::Any], ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
