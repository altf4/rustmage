pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    deck_limit: None,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::GrantInstantSorceryStorm { ability: 0 }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::Reflexive),
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyStormSpell],
    }],
    ..CardRules::EMPTY
};
