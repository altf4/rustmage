pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::Controller, 3)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    extras: &CardExtras {
        disguise_cost: Some(ManaCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
