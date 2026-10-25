// Complete printed body and conditional as-though permission, using actual cast history.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    extras: &CardExtras {
        cast_timing_grants: &[CastFilter::Any],
        cast_timing_requires_opponent_spell: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
