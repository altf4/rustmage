// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledPermanentEntry,
            condition: StatValue::Fixed(1),
        }],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5564)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
