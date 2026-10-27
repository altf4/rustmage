// Complete pinned Oracle and XMage behavior; declared damage shares survive target loss.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::REACH)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Any,
            min: 1,
            max: 3,
        }),
        program: &[Instruction::DamageDivided { total: 3 }],
    }],
    ..CardRules::EMPTY
};
