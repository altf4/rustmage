// Oracle and pinned XMage KiraNerysFirebrand: controlled creatures enter as attackers.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::TRAMPLE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledCreatureEntryTappedAttacking],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Entering,
            power: 1,
            toughness: 1,
        }],
    }],
    ..CardRules::EMPTY
};
