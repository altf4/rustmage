// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::TRAMPLE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantDepartedAdventureUntilNextTurn],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Mosswood Dreadknight // Dread Whispers"),
        adventure_spell: Some(Card::from_catalogue_id(5557)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
