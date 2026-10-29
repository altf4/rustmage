// Full pinned Oracle and XMage rules; both faces remain gated.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Ability, fragments: &[StaticAbilityFragment::CounterKeyword(0, CounterKind::PlusOne, Keywords::VIGILANCE)] }];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        counter_keyword_grants: &[(CounterKind::PlusOne, Keywords::VIGILANCE)],
        enters_prepared: true,
        prepared_spell: Some(Card(5456)),
        cost_alternatives: &[ManaCost { colored: [1, 0, 0, 0, 0, 0], generic: 1 }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
