// Complete printed obligations from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::CounterKeyword(
            0,
            CounterKind::PlusOne,
            Keywords::MENACE,
        )],
    }],
    extras: &CardExtras {
        counter_keyword_grants: &[(CounterKind::PlusOne, Keywords::MENACE)],
        adventure_identity: Some("Great Ugly-Looking Goblin // Clap! Snap!"),
        adventure_spell: Some(Card(5410)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
