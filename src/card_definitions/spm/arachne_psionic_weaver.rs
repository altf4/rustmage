pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ChosenCardTypeSpellTax(1)],
    }],
    extras: &CardExtras {
        web_slinging_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 0,
        }),
        entry_choice: Some(EntryChoiceKind::NoncreatureCardTypeAfterOpponentHand),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
