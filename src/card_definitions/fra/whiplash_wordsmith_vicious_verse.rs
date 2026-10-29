// Direct typed rules from pinned Oracle and XMage; gated pending tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
            condition: StatValue::OpponentNoncombatDamageThisTurn,
            keywords: Keywords::FLYING.union(Keywords::HASTE),
        }],
    }],
    extras: &CardExtras {
        enters_prepared: true,
        prepared_spell: Some(Card(5449)),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 3,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
