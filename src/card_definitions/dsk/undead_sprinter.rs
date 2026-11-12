// Directly authored from the paired Oracle and pinned XMage sources.
const ABILITIES: Keywords = Keywords::TRAMPLE.union(Keywords::HASTE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: ABILITIES,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(ABILITIES)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::SelfGraveyardCastIfWithCounter {
                condition: StatValue::NonZombieCreatureDiedThisTurn,
                counter: CounterKind::PlusOne,
            }],
        },
    ],
    ..CardRules::EMPTY
};
