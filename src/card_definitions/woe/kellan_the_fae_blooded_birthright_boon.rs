// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::DOUBLE_STRIKE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::DOUBLE_STRIKE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::CreatureGroupScaling {
                filters: &[CreatureFilter {
                    relation: 0,
                    other: true,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                }],
                value: StatValue::SourceAuraOrEquipmentCount,
                stats: Stats {
                    power: 1,
                    toughness: 0,
                },
            }],
        },
    ],
    extras: &CardExtras {
        adventure_spell: Some(Card(5556)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
